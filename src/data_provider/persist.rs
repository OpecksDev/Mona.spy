use derive_more::{Display, Error};
use redis::{AsyncCommands, RedisError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error as JsonError;

#[derive(Debug, Display, Error)]
#[display(fmt = "DataPersistError")]
pub enum DataPersistError {
  RedisError(RedisError),
  JsonError(JsonError),
}

type Result<T> = std::result::Result<T, DataPersistError>;

pub async fn get<T: DeserializeOwned>() -> Option<T> {
  let client = redis::Client::open(env!("REDIS_URL")).ok()?;
  let mut con = client.get_async_connection().await.ok()?;

  let json_data: String = con.get(std::any::type_name::<T>()).await.ok()?;
  let data: T = serde_json::from_str(json_data.as_str()).ok()?;
  Some(data)
}

pub async fn set<T: Serialize>(data: &T) -> Result<()> {
  let client = redis::Client::open(env!("REDIS_URL"))?;
  let mut con = client.get_async_connection().await?;

  let json_data = serde_json::to_string(&data)?;

  con
    .set(std::any::type_name::<T>(), json_data.as_str())
    .await?;
  Ok(())
}

impl From<RedisError> for DataPersistError {
  fn from(e: RedisError) -> DataPersistError {
    DataPersistError::RedisError(e)
  }
}

impl From<JsonError> for DataPersistError {
  fn from(e: JsonError) -> DataPersistError {
    DataPersistError::JsonError(e)
  }
}
