mod data_provider;
mod interface;

use actix_web::{get, web, App, HttpResponse, HttpServer};
use data_provider::subscription;
use data_provider::wiki::promotional_codes::PromotionalCodes;
use data_provider::wiki::update_wiki_resource;
use interface::SubscribeBody;
use std::env;

#[get("/promotionalCodes")]
async fn promotional_codes() -> actix_web::Result<HttpResponse> {
  let new_resource = update_wiki_resource::<PromotionalCodes>().await?;
  Ok(HttpResponse::Ok().json(new_resource))
}

#[get("/subscribe")]
async fn subscribe(
  body: web::Json<SubscribeBody>,
) -> Result<HttpResponse, subscription::SubscritionError> {
  match subscription::subscribe(body.into_inner()).await {
    Ok(_) => Ok(HttpResponse::Ok().body("Subscribed!")),
    Err(err) => Err(err),
  }
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

  HttpServer::new(|| App::new().service(promotional_codes).service(subscribe))
    .bind(addr)?
    .run()
    .await
}
