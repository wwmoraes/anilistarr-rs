use std::{future::Future, num::NonZeroU64, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll}, time::{Duration, Instant}};

use axum::response::IntoResponse;
use http::{header, status::StatusCode, Response};
use tower::Service;

use crate::Result;

// Tower Layer

#[derive(Clone)]
pub(crate) struct RateLimitDropLayer(TokenBucket);

impl RateLimitDropLayer {
  pub(crate) fn new(num: u64, burst: u64, per: Duration) -> Self {
    let rate = TokenBucket::new(num, burst, per);

    RateLimitDropLayer(rate)
  }
}

impl<S> tower::Layer<S> for RateLimitDropLayer {
  type Service = RateLimitDropService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    RateLimitDropService::new(inner, self.0.clone())
  }
}

// Tower Service

#[derive(Clone)]
pub(crate) struct RateLimitDropService<T> {
  inner: T,
  rate: TokenBucket,
}

impl<T> RateLimitDropService<T> {
  fn new(inner: T, rate: TokenBucket) -> Self {
    Self {
      inner,
      rate,
    }
  }
}

impl<S, Request, ResBody> Service<Request> for RateLimitDropService<S>
where
  S: Service<Request, Response = Response<ResBody>>,
  S::Response: 'static + Default + IntoResponse,
  S::Future: 'static + Send,
  S::Error: 'static + Send,
  ResBody: 'static + Send + Default,
{
  type Error = S::Error;
  // type Response = S::Response;
  type Response = Response<ResBody>;
  // type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<S::Response, S::Error>> + 'static>>;
  type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send + 'static>>;
  // type Future = S::Future;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, req: Request) -> Self::Future {
    let res = self.rate.try_reserve();
    res.map_or_else(
      |_err| -> Self::Future {
        let mut res = Response::new(ResBody::default());
        *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

        Box::pin(std::future::ready(Ok(res)))
      },
      |delay| -> Self::Future {
        match delay {
          None => Box::pin(self.inner.call(req)),
          Some(available) => {
            let mut res = Response::new(ResBody::default());
            *res.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            res.headers_mut().insert(header::RETRY_AFTER, header::HeaderValue::from((available - Instant::now()).as_secs()));

            Box::pin(std::future::ready(Ok(res)))
          },
        }
      })
  }
}

// token bucket

struct Bucket(Instant, f64);

impl Bucket {
  fn update(&mut self, instant: Instant, rate: f64, capacity: f64) {
    self.1 += (rate * f64::from_bits((instant - self.0).as_secs())).max(capacity);
    self.0 = instant;
  }
}

#[derive(Clone)]
pub(crate) struct TokenBucket {
  rate: f64,
  burst: f64,
  capacity: f64,
  shared: Arc<SharedBucket>,
}

struct SharedBucket {
  state: Mutex<Bucket>,
}

impl TokenBucket {
  pub(crate) fn new(num: u64, burst: u64, per: Duration) -> Self {
    Self {
      rate: f64::from_bits(num) / f64::from_bits(per.as_secs()),
      burst: f64::from_bits(burst),
      capacity: f64::from_bits(num),
      shared: Arc::new(SharedBucket {
        state: Mutex::new(Bucket(Instant::now(), f64::from_bits(num))),
      }),
    }
  }

  fn try_reserve(&self) -> Result<Option<Instant>> {
    self.try_reserve_n(NonZeroU64::MIN)
  }

  fn try_reserve_n(&self, n: NonZeroU64) -> Result<Option<Instant>> {
    let required_tokens = f64::from_bits(n.get());
    if required_tokens > self.burst {
      return Err("required tokens exceed the burst limit".into());
    }

    let mut res = self.shared.state.lock();

    let bucket = match res.as_mut() {
      Err(err) => Err(Into::<Box<dyn std::error::Error>>::into(err.to_string())),
      Ok(guard) => Ok(guard),
    }?;

    let instant = Instant::now();

    match instant < bucket.0 {
      true => (),
      false => {
        bucket.update(instant, self.rate, self.capacity);
      },
    };

    if required_tokens <= bucket.1 {
      bucket.1 -= required_tokens;
      return Ok(None);
    }

    Ok(bucket.0.checked_add(Duration::from_secs_f64((required_tokens - bucket.1) / self.rate)))
  }
}
