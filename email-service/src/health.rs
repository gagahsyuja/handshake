use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::get;
use serde::Serialize;
use std::env;
use std::time::Instant;

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
    pub mailjet: MailjetCheck,
}

#[derive(Debug, Clone, Serialize)]
pub struct MailjetCheck {
    pub ok: bool,
    pub credentials_configured: bool,
    pub latency_ms: Option<u128>,
    pub error: Option<String>,
}

/// Liveness probe - just checks if the service is running
#[get("/live")]
pub fn live() -> Json<LiveResponse> {
    Json(LiveResponse {
        status: HealthStatus::Ok,
        service: "email-service",
    })
}

/// Readiness probe - checks if Mailjet is configured and reachable
#[get("/ready")]
pub async fn ready() -> (Status, Json<ReadyResponse>) {
    // Check if Mailjet credentials are configured
    let api_key = env::var("MAILJET_API_KEY");
    let secret_key = env::var("MAILJET_SECRET_KEY");
    let from_email = env::var("FROM_EMAIL");

    let credentials_configured = api_key.is_ok() && secret_key.is_ok() && from_email.is_ok();

    if !credentials_configured {
        return (
            Status::ServiceUnavailable,
            Json(ReadyResponse {
                status: HealthStatus::Down,
                service: "email-service",
                mailjet: MailjetCheck {
                    ok: false,
                    credentials_configured: false,
                    latency_ms: None,
                    error: Some("Mailjet credentials not configured (MAILJET_API_KEY, MAILJET_SECRET_KEY, FROM_EMAIL required)".to_string()),
                },
            }),
        );
    }

    // Check Mailjet API connectivity
    let start = Instant::now();
    let mailjet_check = check_mailjet_api().await;

    match mailjet_check {
        Ok(latency) => (
            Status::Ok,
            Json(ReadyResponse {
                status: HealthStatus::Ok,
                service: "email-service",
                mailjet: MailjetCheck {
                    ok: true,
                    credentials_configured: true,
                    latency_ms: Some(latency),
                    error: None,
                },
            }),
        ),
        Err(e) => (
            Status::ServiceUnavailable,
            Json(ReadyResponse {
                status: HealthStatus::Down,
                service: "email-service",
                mailjet: MailjetCheck {
                    ok: false,
                    credentials_configured: true,
                    latency_ms: Some(start.elapsed().as_millis()),
                    error: Some(e),
                },
            }),
        ),
    }
}

/// Check Mailjet API connectivity by calling the API version endpoint
async fn check_mailjet_api() -> Result<u128, String> {
    use base64::engine::general_purpose;
    use base64::Engine;

    let api_key = env::var("MAILJET_API_KEY")
        .map_err(|_| "MAILJET_API_KEY not set".to_string())?;
    let secret_key = env::var("MAILJET_SECRET_KEY")
        .map_err(|_| "MAILJET_SECRET_KEY not set".to_string())?;

    let auth = general_purpose::STANDARD.encode(format!("{}:{}", api_key, secret_key));

    let start = Instant::now();

    // Call Mailjet API to check connectivity
    // Using a lightweight endpoint that doesn't send email
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get("https://api.mailjet.com/v3/REST/contact")
        .header("Authorization", format!("Basic {}", auth))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Mailjet API: {}", e))?;

    let latency = start.elapsed().as_millis();

    // Check if authentication was successful
    let status = response.status();
    if status.is_success() || status.as_u16() == 200 {
        Ok(latency)
    } else if status.as_u16() == 401 {
        Err("Mailjet API authentication failed - check credentials".to_string())
    } else {
        Err(format!("Mailjet API returned status: {}", status))
    }
}