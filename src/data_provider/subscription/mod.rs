use super::persist;
use crate::interface::SubscribeBody;

use derive_more::{Display, Error};
use reqwest::Body;
use serde::Deserialize;
use serde::Serialize;
use std::cmp;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
struct Subscrition {
  uri: String,
  token: Option<String>,
  expiration: u64,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
struct PushBody<T> {
  pub id: String,
  pub token: Option<String>,
  pub expiration: u64,
  pub resource: T,
  pub resource_type: String,
}

#[derive(Debug, Serialize)]
pub struct SyncBody {
  id: String,
  token: Option<String>,
  expiration: u64,
}

#[derive(Debug, Deserialize)]
pub struct SyncResponse {
  id: String,
}

#[derive(Debug, Display, Error)]
#[display(fmt = "SubscriptionError")]
pub enum SubscritionError {
  SyncError(reqwest::Error),
  DataPersistError(persist::DataPersistError),
  JsonError(serde_json::Error),
}

impl actix_web::error::ResponseError for SubscritionError {}

type Result<T> = std::result::Result<T, SubscritionError>;

pub async fn notify<T: Serialize + Clone>(resource: &T) -> Result<()> {
  let subscriptions: HashMap<String, Subscrition> = match persist::get().await {
    Some(subscription) => subscription,
    None => return Ok(()),
  };

  for (id, subscription) in subscriptions {
    let body = PushBody {
      id,
      token: subscription.token,
      resource: resource.clone(),
      resource_type: std::any::type_name::<T>().to_owned(),
      expiration: subscription.expiration,
    };

    let body: Body = match serde_json::to_string(&body) {
      Ok(data) => data.into(),
      Err(e) => return Err(SubscritionError::JsonError(e)),
    };

    let client = reqwest::Client::new();
    let resp = client
      .get(subscription.uri.as_str())
      .body(body)
      .send()
      .await;

    match resp {
      Ok(_) => {}
      err => println!("{:?}", err),
    };
  }

  Ok(())
}

pub async fn try_sync(subscribe_body: &SubscribeBody, expiration: u64) -> Result<()> {
  let body = SyncBody {
    id: subscribe_body.id.to_owned(),
    token: subscribe_body.token.to_owned(),
    expiration,
  };

  let body: Body = match serde_json::to_string(&body) {
    Ok(data) => data.into(),
    Err(e) => return Err(SubscritionError::JsonError(e)),
  };

  reqwest::Client::new()
    .get(subscribe_body.uri.as_str())
    .body(body)
    .send()
    .await
    .map_err(|err| SubscritionError::SyncError(err))?
    .json()
    .await
    .map_err(|err| SubscritionError::SyncError(err))?;

  Ok(())
}

pub async fn subscribe(body: SubscribeBody) -> Result<()> {
  let start = SystemTime::now();
  let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_secs();
  let default_expiration = since_the_epoch + 24 * 3600;

  let expiration: u64 = match body.expiration {
    None => default_expiration,
    Some(timestamp) => timestamp,
  };

  let expiration = cmp::max(cmp::min(expiration, default_expiration), since_the_epoch);

  try_sync(&body, expiration).await?;

  let mut subscriptions: HashMap<String, Subscrition> = match persist::get().await {
    Some(subscription) => subscription,
    None => HashMap::new(),
  };

  let id = body.id;
  let uri = body.uri;
  let token = body.token;

  match subscriptions.get_mut(&id) {
    None => {
      subscriptions.insert(
        id.clone(),
        Subscrition {
          uri,
          expiration,
          token,
        },
      );
    }
    Some(subscription) => {
      subscription.uri = uri;
      subscription.token = token;
      subscription.expiration = expiration;
    }
  };

  match persist::set(&subscriptions).await {
    Ok(_) => Ok(()),
    Err(err) => Err(SubscritionError::DataPersistError(err)),
  }
}
