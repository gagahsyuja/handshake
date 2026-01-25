pub mod auth;
pub mod db;
pub mod models;
pub mod routes;
pub mod schema;

use rocket::fairing::{Fairing, Info, Kind};
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

    let cors = CorsOptions::default().to_cors().unwrap();

    let _rocket = rocket::build()
        // .attach(CORS)
        .attach(cors)
        .attach(db::DbConn::fairing())
        .mount(
            "/products",
            routes![
                routes::products::list_products,
                routes::products::get_product,
                routes::products::create_product,
                routes::products::update_product,
                routes::products::delete_product,
            ],
        )
        .mount(
            "/categories",
            routes![
                routes::categories::list_categories,
                routes::categories::get_category_products,
            ],
        )
        .launch()
        .await?;

    Ok(())
}
