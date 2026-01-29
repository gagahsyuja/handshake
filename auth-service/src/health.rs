use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::get;
use serde::Serialize;
use std::env;
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
    pub email_service: EmailServiceCheck,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbCheck {
    pub ok: bool,
    pub latency_ms: Option<u128>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailServiceCheck {
    pub ok: bool,
    pub configured: bool,
    pub latency_ms: Option<u128>,
    pub error: Option<String>,
}

/// Liveness probe - just checks if the service is running
#[get("/live")]
pub fn live() -> Json<LiveResponse> {
    Json(LiveResponse {
        status: HealthStatus::Ok,
        service: "auth-service",
    })
}

/// Readiness probe - checks database and email service connectivity
#[get("/ready")]
pub async fn ready(db: DbConn) -> (Status, Json<ReadyResponse>) {
    let start = Instant::now();

    // Check database connectivity
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

    let db_check = match db_result {
        Ok(()) => DbCheck {
            ok: true,
            latency_ms: Some(start.elapsed().as_millis()),
            error: None,
        },
        Err(e) => DbCheck {
            ok: false,
            latency_ms: None,
            error: Some(e),
        },
    };

    // Check email service connectivity
    let email_service_check = check_email_service().await;

    // Determine overall status
    // Email service is optional - if not configured, we're still ready
    // But if configured and failing, we're degraded (not down)
    let status = if !db_check.ok {
        HealthStatus::Down
    } else if email_service_check.configured && !email_service_check.ok {
        HealthStatus::Degraded
    } else {
        HealthStatus::Ok
    };

    let http_status = match status {
        HealthStatus::Ok => Status::Ok,
        HealthStatus::Degraded => Status::Ok, // Still return 200 for degraded
        HealthStatus::Down => Status::ServiceUnavailable,
    };

    (
        http_status,
        Json(ReadyResponse {
            status,
            service: "auth-service",
            db: db_check,
            email_service: email_service_check,
        }),
    )
}

/// Check if email service is reachable
async fn check_email_service() -> EmailServiceCheck {
    let email_service_url = match env::var("EMAIL_SERVICE_URL") {
        Ok(url) => url,
        Err(_) => {
            return EmailServiceCheck {
                ok: false,
                configured: false,
                latency_ms: None,
                error: Some("EMAIL_SERVICE_URL not configured".to_string()),
            };
        }
    };

    let start = Instant::now();

    // Try to reach email service health endpoint
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return EmailServiceCheck {
                ok: false,
                configured: true,
                latency_ms: None,
                error: Some(format!("Failed to create HTTP client: {}", e)),
            };
        }
    };

    let health_url = format!("{}/live", email_service_url);

    match client.get(&health_url).send().await {
        Ok(response) => {
            let latency = start.elapsed().as_millis();
            if response.status().is_success() {
                EmailServiceCheck {
                    ok: true,
                    configured: true,
                    latency_ms: Some(latency),
                    error: None,
                }
            } else {
                EmailServiceCheck {
                    ok: false,
                    configured: true,
                    latency_ms: Some(latency),
                    error: Some(format!("Email service returned status: {}", response.status())),
                }
            }
        }
        Err(e) => EmailServiceCheck {
            ok: false,
            configured: true,
            latency_ms: Some(start.elapsed().as_millis()),
            error: Some(format!("Failed to connect to email service: {}", e)),
        },
    }
}
