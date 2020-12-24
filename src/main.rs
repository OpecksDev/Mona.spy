use actix_web::{get, App, HttpResponse, HttpServer, Result};

use std::env;
use types::wiki::{PromotionalCodes, WikiResource};

mod types;

#[get("/promotionalCodes")]
async fn promotional_codes() -> Result<HttpResponse> {
  Ok(HttpResponse::Ok().json(PromotionalCodes::get_wiki_resource().await?))
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
