pub mod auth;
pub mod db;
pub mod email;
pub mod health;
pub mod models;
pub mod routes;
pub mod schema;

use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::figment::value::{Map, Value};
use rocket::http::Header;
use rocket::routes;
use rocket::{Request, Response};
use rocket_cors::{AllowedOrigins, CorsOptions};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn run_migrations(connection: &mut PgConnection) {
    match connection.run_pending_migrations(MIGRATIONS) {
        Ok(migrations) => {
            if migrations.is_empty() {
                println!("No pending migrations to run.");
            } else {
                println!("Successfully ran {} migrations:", migrations.len());
                for migration in migrations {
                    println!("  - {}", migration);
                }
            }
        }
        Err(e) => {
            eprintln!("Error running migrations: {}", e);
            std::process::exit(1);
        }
    }
}

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

    let app_url =
        std::env::var("APP_URL").expect("APP_URL must be set (e.g. in .env)");

    // Run database migrations on startup
    println!("Running database migrations...");
    let mut connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|e| {
            eprintln!("Error connecting to database for migrations: {}", e);
            std::process::exit(1);
        });
    run_migrations(&mut connection);

    let mut db: Map<String, Value> = Map::new();
    db.insert("url".to_string(), database_url.into());

    let mut databases: Map<String, Value> = Map::new();
    databases.insert("auth_db".to_string(), db.into());

    let figment = rocket::Config::figment().merge(("databases", databases));

    let allowed_origins = AllowedOrigins::some_exact(
        &[
            app_url,
            "http://localhost:3000".to_string()
        ]
    );

    let cors = CorsOptions::default()
        .allowed_origins(allowed_origins)
        .to_cors()
        .unwrap();

    let _rocket = rocket::custom(figment)
        // .attach(CORS)
        .attach(cors)
        .attach(db::DbConn::fairing())
        .mount("/", routes![health::live, health::ready])
        .mount(
            "/",
            routes![
                routes::register,
                routes::verify_email,
                routes::login,
                routes::resend_otp,
                routes::me,
            ],
        )
        .launch()
        .await?;

    Ok(())
}
