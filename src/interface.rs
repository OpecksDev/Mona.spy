use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct SubscribeBody {
  pub id: String,              // Your channel ID.
  pub uri: String,             // Ex: "https://mydomain.com/notifications". Your receiving URL.
  pub token: Option<String>, // Ex: "target=myApp-myCalendarChannelDest". (Optional) Your channel token.
  pub expiration: Option<u64>, // Ex: 1426325213000 // (Optional) Your requested channel expiration time.
}
