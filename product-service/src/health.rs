use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::get;
use serde::Serialize;
use std::time::Instant;

use crate::db::DbConn;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Ok,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveResponse {
    pub status: HealthStatus,
    pub service: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadyResponse {
    pub status: HealthStatus,
    pub service: &'static str,
    pub db: DbCheck,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbCheck {
    pub ok: bool,
    pub latency_ms: Option<u128>,
    pub error: Option<String>,
}



/// Liveness probe
#[get("/live")]
pub fn live() -> Json<LiveResponse> {
    Json(LiveResponse {
        status: HealthStatus::Ok,
        service: "product-service",
    })
}

/// Readiness probe
#[get("/ready")]
pub async fn ready(db: DbConn) -> (Status, Json<ReadyResponse>) {
    let start = Instant::now();

    // Diesel is sync; DbConn::run executes this on a blocking thread via rocket_sync_db_pools.
    let db_result: Result<(), String> = db
        .run(|conn| {
            use diesel::sql_query;
            use diesel::RunQueryDsl;

            sql_query("SELECT 1")
                .execute(conn)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .await;

    match db_result {
        Ok(()) => (
            Status::Ok,
            Json(ReadyResponse {
                status: HealthStatus::Ok,
                service: "product-service",
                db: DbCheck {
                    ok: true,
                    latency_ms: Some(start.elapsed().as_millis()),
                    error: None,
                },
            }),
        ),
        Err(e) => (
            Status::ServiceUnavailable,
            Json(ReadyResponse {
                status: HealthStatus::Down,
                service: "product-service",
                db: DbCheck {
                    ok: false,
                    latency_ms: None,
                    error: Some(e),
                },
            }),
        ),
    }
}
