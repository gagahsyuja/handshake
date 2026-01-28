pub mod auth;
pub mod db;
pub mod geolocation;
pub mod health;
pub mod models;
pub mod nominatim;
pub mod routes;
pub mod schema;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::figment::value::{Map, Value};
use rocket::http::Header;
use rocket::routes;
use rocket::{Request, Response};

use rocket_cors::CorsOptions;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PUT, DELETE, OPTIONS",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "*, Authorization",
        ));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (e.g. in .env)");

    let mut db: Map<String, Value> = Map::new();
    db.insert("url".to_string(), database_url.into());

    let mut databases: Map<String, Value> = Map::new();
    databases.insert("order_db".to_string(), db.into());

    let figment = rocket::Config::figment().merge(("databases", databases));

    let cors = CorsOptions::default().to_cors().unwrap();

    let _rocket = rocket::custom(figment)
        // .attach(CORS)
        .attach(cors)
        .attach(db::DbConn::fairing())
        .mount("/", routes![health::live, health::ready])
        .mount(
            "/orders",
            routes![routes::create_order, routes::get_order, routes::my_orders,],
        )
        .mount(
            "/geocode",
            routes![routes::geocode_address, routes::reverse_geocode,],
        )
        .mount("/locations", routes![routes::upsert_my_location,])
        .launch()
        .await?;

    Ok(())
}
