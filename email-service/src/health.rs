use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::get;
use serde::Serialize;
use std::env;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

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
    pub smtp: SmtpCheck,
}

#[derive(Debug, Clone, Serialize)]
pub struct SmtpCheck {
    pub ok: bool,
    pub host: String,
    pub port: u16,
    pub latency_ms: Option<u128>,
    pub error: Option<String>,
}

/// Liveness probe
#[get("/live")]
pub fn live() -> Json<LiveResponse> {
    Json(LiveResponse {
        status: HealthStatus::Ok,
        service: "email-service",
    })
}

/// Readiness probe
#[get("/ready")]
pub fn ready() -> (Status, Json<ReadyResponse>) {
    let (host, port) = smtp_host_port_from_env();

    let (smtp_ok, latency_ms, err) = match tcp_connect_check(&host, port, smtp_timeout()) {
        Ok(ms) => (true, Some(ms), None),
        Err(e) => (false, None, Some(e)),
    };

    let status = if smtp_ok {
        HealthStatus::Ok
    } else {
        HealthStatus::Down
    };

    let http_status = if smtp_ok {
        Status::Ok
    } else {
        Status::ServiceUnavailable
    };

    (
        http_status,
        Json(ReadyResponse {
            status,
            service: "email-service",
            smtp: SmtpCheck {
                ok: smtp_ok,
                host,
                port,
                latency_ms,
                error: err,
            },
        }),
    )
}

fn smtp_host_port_from_env() -> (String, u16) {
    let host = env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let port: u16 = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587);
    (host, port)
}

fn smtp_timeout() -> Duration {
    // Optional env override, in milliseconds.
    // Example: SMTP_HEALTH_TIMEOUT_MS=1000
    let ms: u64 = env::var("SMTP_HEALTH_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1500);
    Duration::from_millis(ms)
}

fn tcp_connect_check(host: &str, port: u16, timeout: Duration) -> Result<u128, String> {
    // Resolve host first so we can provide a clear error.
    let addr_str = format!("{}:{}", host, port);
    let mut addrs = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed for {}: {}", addr_str, e))?;

    let addr = addrs
        .next()
        .ok_or_else(|| format!("No socket addresses found for {}", addr_str))?;

    let start = Instant::now();
    TcpStream::connect_timeout(&addr, timeout)
        .map_err(|e| format!("TCP connect failed to {}: {}", addr_str, e))?;

    Ok(start.elapsed().as_millis())
}
