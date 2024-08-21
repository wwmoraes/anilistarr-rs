use crate::{entities::SourceID, usecases::Tracker, Result};
use governor::DefaultDirectRateLimiter;
use graphql_client::reqwest::post_graphql;
use reqwest::Client;
use nonzero_ext::*;

#[derive(Debug)]
pub struct Anilist {
  url: String,
  client: Client,
  limiter: DefaultDirectRateLimiter,
}

impl Anilist {
  #[tracing::instrument(skip(client))]
  pub fn new(url: String, client: Client) -> Self {
    Self {
      client,
      url,
      limiter: governor::RateLimiter::direct(governor::Quota::per_minute(nonzero!(90u32)))
    }
  }
}

impl Tracker for Anilist {
  #[tracing::instrument(skip(self), ret, err)]
  fn get_user_id(&self, name: &str) -> Result<String> {
    self.limiter.check().map_err(|err| crate::usecases::Errors::Unknown(err.to_string()))?;

    let variables = queries::get_user_by_name::Variables {
      name: name.to_owned(),
    };

    let data = crate::resolve(async {
      post_graphql::<queries::GetUserByName, _>(&self.client, &self.url, variables)
        .await.map(|res| res.data)
    })?.ok_or(crate::usecases::Errors::UserNoMedia("FAIL".to_owned()))?;

    let user = data.user.ok_or("no user data")?;

    Ok(user.id.to_string())
  }

  #[tracing::instrument(skip(self), ret, err)]
  fn get_media_list_ids(&self, user_id: &str) -> Result<Vec<SourceID>> {
    self.limiter.check().map_err(|err| crate::usecases::Errors::Unknown(err.to_string()))?;

    let variables = queries::get_watching::Variables {
      user_id: user_id.parse()?,
      page: 1,
      per_page: 50,
    };

    let data = crate::resolve(async {
      post_graphql::<queries::GetWatching, _>(&self.client, &self.url, variables)
        .await.map(|res| res.data)
    })?.ok_or(crate::usecases::Errors::UserNoMedia("FAIL".to_owned()))?;

    let result: Vec<SourceID> = data.page
      .and_then(|v|v.media_list)
      .map(|v| v
        .into_iter()
        .filter_map(|v| v
          .and_then(|v| v.media)
          .map(|v| v.id.to_string()))
        .collect()
      ).unwrap_or_default();

    Ok(result)
  }
}

mod queries {
  use graphql_client::GraphQLQuery;

  #[derive(GraphQLQuery)]
  #[graphql(
      schema_path = "graphql/anilist/schema.gql",
      query_path = "graphql/anilist/GetUserByName.gql",
      response_derives = "Debug",
  )]
  pub(crate) struct GetUserByName;

  #[derive(GraphQLQuery)]
  #[graphql(
      schema_path = "graphql/anilist/schema.gql",
      query_path = "graphql/anilist/GetWatching.gql",
      response_derives = "Debug",
  )]
  pub(crate) struct GetWatching;
}
