mod data_provider;
mod interface;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use data_provider::subscription;
use data_provider::subscription::{PushBody, PushResponse};
use data_provider::wiki::promotional_codes::PromotionalCodes;
use data_provider::wiki::update_wiki_resource;
use interface::SubscribeBody;
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
async fn subscribe_test(body: web::Json<PushBody<String>>) -> actix_web::Result<HttpResponse> {
  match &body.resource_type {
    None => {
      println!("Sync {:?}", body);
    }
    Some(_) => {
      println!("Update {:?}", body);
    }
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
