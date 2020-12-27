mod check_update;
mod data_provider;
mod interface;

use actix_web::{error, get, post, web, App, HttpResponse, HttpServer};
use check_update::call_every_hour;
use data_provider::subscription;
use data_provider::subscription::{PushBody, PushResponse};
use data_provider::wiki::promotional_codes::PromotionalCodes;
use data_provider::wiki::update_wiki_resource;
use interface::SubscribeBody;
use serde_json::Value;
use std::env;

#[get("/promotional_codes")]
async fn promotional_codes() -> actix_web::Result<HttpResponse> {
  let new_resource = update_wiki_resource::<PromotionalCodes>().await?;
  Ok(HttpResponse::Ok().json(new_resource))
}

#[post("/subscribe")]
async fn subscribe(
  body: web::Json<SubscribeBody>,
) -> Result<HttpResponse, subscription::SubscritionError> {
  println!("{:?}", body);
  match subscription::subscribe(body.into_inner()).await {
    Ok(_) => Ok(HttpResponse::Ok().body("Subscribed!")),
    Err(err) => Err(err),
  }
}

#[post("/subscribe_test")]
async fn subscribe_test(body: web::Json<PushBody<Value>>) -> actix_web::Result<HttpResponse> {
  match &body.resource_type {
    None => {
      println!("Sync {:?}", body);
    }
    Some(resource_type) => match resource_type.as_str() {
      "mona_spy::data_provider::wiki::promotional_codes::PromotionalCodes" => {
        let resource = body
          .resource
          .to_owned()
          .ok_or(error::ErrorBadRequest("Empty Resource"))?;
        let resource: PromotionalCodes = serde_json::from_value(resource)
          .map_err(|_| error::ErrorBadRequest("Bad Resource Format"))?;
        println!("Received Update Request for {:?}", resource);
      }
      _ => return Err(error::ErrorBadRequest("Invalid Resource Type:")),
    },
  };

  Ok(HttpResponse::Ok().json(PushResponse {
    id: body.id.to_owned(),
  }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  #[cfg(debug_assertions)]
  let ip = "127.0.0.1";

  #[cfg(not(debug_assertions))]
  let ip = "0.0.0.0";

  let port = env::var("PORT").map_or("8080".to_owned(), |x| x);
  let addr = ip.to_owned() + ":" + port.as_str();
  let addr_copy = addr.clone();

  actix_rt::spawn(async move {
    call_every_hour(addr_copy, vec!["/promotional_codes"]).await;
  });

  println!("Running Server on {}", addr);

  HttpServer::new(|| {
    let app = App::new().service(promotional_codes).service(subscribe);
    #[cfg(debug_assertions)] // Debug APIs
    let app = app.service(subscribe_test);
    app
  })
  .bind(addr)?
  .run()
  .await
}
