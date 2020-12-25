mod data_provider;

use actix_web::{get, App, HttpResponse, HttpServer, Result};
use data_provider::wiki::{PromotionalCodes, update_wiki_resource};
use std::env;

#[get("/promotionalCodes")]
async fn promotional_codes() -> Result<HttpResponse> {
  let new_resource = update_wiki_resource::<PromotionalCodes>().await?;
  Ok(HttpResponse::Ok().json(new_resource))
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

  HttpServer::new(|| App::new().service(promotional_codes))
    .bind(addr)?
    .run()
    .await
}
