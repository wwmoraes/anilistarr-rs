use anilistarr_rs::{adapters::{mappers, trackers}, drivers::{self, api, providers}, usecases::{Cache, Mapper, Store, TrackerMediaLister}, Result};
use axum::{routing, Router};
use std::path::Path;
use tower::ServiceBuilder;
use http::{header::{self, HeaderName}, HeaderValue};
use tower_http::set_header::SetResponseHeaderLayer;
use signal_hook::{consts::SIGINT, iterator::Signals};
use opentelemetry_sdk::resource::Resource;

use opentelemetry::KeyValue;

mod limiter;

const DATA_PATH: &str = "tmp/lmdb";
const REFRESH_ON_BOOT: bool = false;
const USE_REDIS: bool = false;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result {
  tracing_subscriber::fmt::init();

  if let Err(err) = instrumentation::init(Resource::new(vec![
    KeyValue::new("service.name", "anilistarr"),
    KeyValue::new("service.namespace", "github.com/wwmoraes/anilistarr"),
    KeyValue::new("service.version", "0.0.0"),
    // KeyValue::new("host.id", "?"),
  ])) {
    tracing::error!("failed to initialize instrumentation: {}", err);
  };

  tracing::info!("opening store");

  let store = init_store()?;
  let cache = if USE_REDIS {
    init_redis_cache("redis://127.0.0.1/")
  } else {
    init_cache()
  }?;

  let provider = Box::new(providers::anilist::Fribbs("https://github.com/Fribb/anime-lists/raw/master/anime-list-full.json".to_owned()));

  let mapper = Box::new(mappers::Persistent::new(provider, store));

  let http_client = reqwest::Client::new();
  if REFRESH_ON_BOOT {
    tracing::info!("refreshing data");
    mapper.refresh(&http_client)?;
  }

  let tracker = Box::new(drivers::trackers::Anilist::new(String::from("https://graphql.anilist.co"), http_client));

  let cached_tracker = Box::new(trackers::CachedTracker::new(cache, tracker));

  // let mut tracker = Box::new(trackers::Memory::default());
  // tracker.user_ids.insert("test".into(), 1234);
  // tracker.media_lists.insert(1234, vec!["1".into()]);

  // let mut mapper = Box::new(mappers::Memory::default());
  // mapper.mapping.insert("1".into(), "91".into());
  // mapper.mapping.insert("2".into(), "92".into());

  let state = api::state![TrackerMediaLister::new(cached_tracker, mapper)];

  let app = Router::new()
    // TODO change to /anilist to support multiple sources
    .nest("/user", Router::new()
      .route("/:name/id", routing::get(api::handlers::get_user_id))
      .route("/:name/media", routing::get(api::handlers::get_user_media))
      .with_state(state)
    ).layer(ServiceBuilder::new()
      .layer(tower_http::compression::CompressionLayer::new().gzip(true))
      .layer(SetResponseHeaderLayer::overriding(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin")
      ))
      .layer(SetResponseHeaderLayer::overriding(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY")
      ))
      .layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff")
      ))
      .layer(limiter::RateLimitDropLayer::new(1000, 20, std::time::Duration::from_secs(60)))
    );

  let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;

  tracing::info!("HTTP server listening on {}", listener.local_addr()?);
  axum::serve(listener, app).with_graceful_shutdown(interrupt()).await?;

  Ok(())
}

type ServiceStore = dyn Store + Send + Sync;
type ServiceCache = dyn Cache + Send + Sync;

fn init_store() -> Result<Box<ServiceStore>> {
  let path = Path::new(DATA_PATH).join("store");

  std::fs::create_dir_all(path.as_path())?;

  let store = Box::new(drivers::persistence::LMDB::open(path.as_path())?);

  Ok(store)
}

fn init_cache() -> Result<Box<ServiceCache>> {
  let path = Path::new(DATA_PATH).join("store");

  std::fs::create_dir_all(path.as_path())?;

  let cache = Box::new(drivers::persistence::LMDB::open(path.as_path())?);

  Ok(cache)
}

fn init_redis_cache<T: redis::IntoConnectionInfo>(params: T) -> Result<Box<ServiceCache>> {
  let client = Box::new(redis::Client::open(params)?);

  Ok(client)
}

async fn interrupt() {
  let mut signals = Signals::new([SIGINT]).expect("unable to register signal handler");
  let handle = signals.handle();

  match signals.forever().next() {
    Some(SIGINT) => tracing::info!("interrupted, stopping gracefully"),
    Some(signal) => tracing::info!("received signal {}, stopping", signal),
    None => tracing::info!("handler closed, stopping"),
  }

  handle.close();
}

mod instrumentation {
  use opentelemetry_otlp::{ExportConfig, HttpExporterBuilder, WithExportConfig};
  use opentelemetry_sdk::{logs::LoggerProvider, metrics::{PeriodicReader, SdkMeterProvider}, propagation::{BaggagePropagator, TraceContextPropagator}, runtime::RuntimeChannel, trace::TracerProvider, Resource};
  use opentelemetry::{propagation::composite::TextMapCompositePropagator, trace::TracerProvider as _};
  use std::time::Duration;
  use tracing_subscriber::layer::SubscriberExt;

  pub(crate) fn init(resource: Resource) -> crate::Result {
    let runtime = opentelemetry_sdk::runtime::Tokio;
    let resource = Resource::default().merge(&resource);

    opentelemetry::global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
      Box::new(TraceContextPropagator::new()),
      Box::new(BaggagePropagator::new()),
    ]));

    tracing::debug!("creating tracer provider");
    let tracer_provider = init_otlp_trace(runtime.clone(), resource.clone())
      .unwrap_or(init_stdout_trace(runtime.clone(), resource.clone()));

    let meter_provider = init_otlp_meter(runtime.clone(), resource.clone())
      .unwrap_or(init_stdout_meter(runtime.clone(), resource.clone()));

    let logger_provider = init_otlp_logger(runtime.clone(), resource.clone())
      .unwrap_or(init_stdout_logger(runtime.clone(), resource.clone()));

    tracing_subscriber::registry()
      .with(tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("")))
      .with(tracing_opentelemetry::MetricsLayer::new(meter_provider.clone()))
      .with(opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider))
      .with(tracing_subscriber::fmt::Layer::default().with_level(true).compact());

    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    Ok(())
  }

  fn traces_exporter() -> crate::Result<HttpExporterBuilder> {
    std::env::var(opentelemetry_otlp::OTEL_EXPORTER_OTLP_TRACES_ENDPOINT)
      .or(std::env::var(opentelemetry_otlp::OTEL_EXPORTER_OTLP_ENDPOINT))
      .map_err(|err| err.into())
      .map(|endpoint| opentelemetry_otlp::new_exporter().http().with_export_config(ExportConfig {
        endpoint,
        ..Default::default()
      }))
  }

  fn logger_exporter() -> crate::Result<HttpExporterBuilder> {
    std::env::var(opentelemetry_otlp::OTEL_EXPORTER_OTLP_LOGS_ENDPOINT)
      .or(std::env::var(opentelemetry_otlp::OTEL_EXPORTER_OTLP_ENDPOINT))
      .map_err(|err| err.into())
      .map(|endpoint| opentelemetry_otlp::new_exporter().http().with_export_config(ExportConfig {
        endpoint,
        ..Default::default()
      }))
  }

  fn meter_exporter() -> crate::Result<HttpExporterBuilder> {
    std::env::var(opentelemetry_otlp::OTEL_EXPORTER_OTLP_METRICS_ENDPOINT)
      .or(std::env::var(opentelemetry_otlp::OTEL_EXPORTER_OTLP_ENDPOINT))
      .map_err(|err| err.into())
      .map(|endpoint| opentelemetry_otlp::new_exporter().http().with_export_config(ExportConfig {
        endpoint,
        ..Default::default()
      }))
  }

  fn init_otlp_trace<Runtime: RuntimeChannel>(runtime: Runtime, resource: Resource) -> crate::Result<TracerProvider> {
    use opentelemetry_sdk::trace::Config;

    let exporter = traces_exporter()?;
    let provider = opentelemetry_otlp::new_pipeline()
      .tracing()
      .with_exporter(exporter)
      .with_trace_config(Config::default().with_resource(resource.clone()))
      .install_batch(runtime.clone())?;

    Ok(provider)
  }

  fn init_otlp_meter<Runtime: RuntimeChannel>(runtime: Runtime, resource: Resource) -> crate::Result<SdkMeterProvider> {
    let exporter = meter_exporter()?;
    let provider = opentelemetry_otlp::new_pipeline()
      .metrics(runtime)
      .with_exporter(exporter)
      .with_resource(resource)
      .with_period(Duration::from_secs(60))
      .with_timeout(Duration::from_secs(30))
      .build()?;

    Ok(provider)
  }

  fn init_otlp_logger<Runtime: RuntimeChannel>(runtime: Runtime, resource: Resource) -> crate::Result<LoggerProvider> {
    let exporter = logger_exporter()?;
    let provider = opentelemetry_otlp::new_pipeline()
      .logging()
      .with_exporter(exporter)
      .with_resource(resource)
      .install_batch(runtime)?;

    Ok(provider)
  }

  fn init_stdout_trace<Runtime: RuntimeChannel>(runtime: Runtime, resource: Resource) -> TracerProvider {
    use opentelemetry_sdk::trace::Config;

    let exporter = opentelemetry_stdout::SpanExporter::default();

    TracerProvider::builder()
      .with_batch_exporter(exporter, runtime)
      .with_config(Config::default().with_resource(resource))
      .build()
  }

  fn init_stdout_meter<Runtime: RuntimeChannel>(runtime: Runtime, resource: Resource) -> SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime)
      .with_interval(Duration::from_secs(60))
      .with_timeout(Duration::from_secs(30))
      .build();

    SdkMeterProvider::builder()
      .with_reader(reader)
      .with_resource(resource)
      .build()
  }

  fn init_stdout_logger<Runtime: RuntimeChannel>(runtime: Runtime, resource: Resource) -> LoggerProvider {
    let exporter = opentelemetry_stdout::LogExporter::default();

    LoggerProvider::builder()
      .with_batch_exporter(exporter, runtime)
      .with_resource(resource)
      .build()
  }
}
