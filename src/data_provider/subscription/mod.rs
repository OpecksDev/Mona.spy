use super::persist;
use crate::interface::SubscribeBody;

use actix_web::http::StatusCode;
use derive_more::{Display, Error};
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

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct PushBody<T> {
  pub id: String,
  pub token: Option<String>,
  pub expiration: u64,
  pub resource: Option<T>,
  pub resource_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PushResponse {
  pub id: String,
}

#[derive(Debug, Display, Error)]
pub enum SubscritionError {
  #[display(fmt = "Error: The provided URL didn't respond the request with the provided ID")]
  SyncError(reqwest::Error),
  #[display(fmt = "Error: The provided URL didn't respond the request with the provided ID")]
  DifferentIdSyncError,
  #[display(fmt = "Error: The Subscription wasn't saved")]
  DataPersistError(persist::DataPersistError),
}

impl actix_web::error::ResponseError for SubscritionError {
  fn status_code(&self) -> StatusCode {
    StatusCode::BAD_REQUEST
  }
}

type Result<T> = std::result::Result<T, SubscritionError>;

pub async fn notify<T: Serialize + Clone>(resource: &T) -> Result<()>
where
  T: std::fmt::Debug,
{
  let subscriptions: HashMap<String, Subscrition> = match persist::get().await {
    Some(subscription) => subscription,
    None => return Ok(()),
  };

  for (id, subscription) in subscriptions {
    let body = PushBody {
      id,
      token: subscription.token,
      resource: Some(resource.clone()),
      resource_type: Some(std::any::type_name::<T>().to_owned()),
      expiration: subscription.expiration,
    };

    let client = reqwest::Client::new();
    let resp = client
      .post(subscription.uri.as_str())
      .json(&body)
      .send()
      .await;

    match resp {
      Ok(resp) => match resp.status() {
        reqwest::StatusCode::OK => {}
        code => println!(
          ">>> PushNotificationError for {} <<<\nStatus Code {}\n{:?}\n>>> End <<<",
          &subscription.uri, &code, &resp
        ),
      },
      err => println!(
        ">>> PushNotificationError for {} <<<\n{:?}\n>>> End <<<",
        &subscription.uri, err
      ),
    };
  }

  Ok(())
}

pub async fn try_sync(subscribe_body: &SubscribeBody, expiration: u64) -> Result<()> {
  let body = PushBody::<()> {
    id: subscribe_body.id.to_owned(),
    token: subscribe_body.token.to_owned(),
    expiration,
    resource: None,
    resource_type: None,
  };

  let resp: PushResponse = reqwest::Client::new()
    .post(subscribe_body.uri.as_str())
    .json(&body)
    .send()
    .await
    .map_err(|err| SubscritionError::SyncError(err))?
    .json()
    .await
    .map_err(|err| SubscritionError::SyncError(err))?;

  if resp.id != subscribe_body.id {
    return Err(SubscritionError::DifferentIdSyncError);
  }

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
