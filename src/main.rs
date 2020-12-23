#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::json::Json;
use std::env;
use types::wiki::{PromotionalCodes, WikiError, WikiResource};

mod types;

#[get("/promotionalCodes")]
async fn promotional_codes() -> Json<Result<PromotionalCodes, WikiError>> {
    Json(PromotionalCodes::get_wiki_resource().await)
}

#[launch]
fn rocket() -> rocket::Rocket {
    let port = env::var("PORT")
        .map_or("8080".to_owned(), |x| x)
        .parse::<u16>()
        .unwrap();

    let figment = rocket::Config::figment().merge(("port", port));
    // .merge(("address", "0.0.0.0"));
    rocket::custom(figment).mount("/", routes![promotional_codes])
}
