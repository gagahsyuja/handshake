pub mod health;
pub mod routes;
pub mod smtp;

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
            "POST, GET, OPTIONS",
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
        .mount("/", routes![health::live, health::ready])
        .mount(
            "/",
            routes![
                routes::send_verification,
                routes::send_order_notification,
                routes::send_custom_email,
            ],
        )
        .launch()
        .await?;

    Ok(())
}
