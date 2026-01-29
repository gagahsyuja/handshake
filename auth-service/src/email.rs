use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
pub struct VerificationEmailRequest {
    pub to_email: String,
    pub to_name: String,
    pub verification_code: String,
}

#[derive(Debug, Deserialize)]
struct EmailServiceResponse {
    success: bool,
    message: String,
}

pub async fn send_verification_email(
    to_email: &str,
    to_name: &str,
    verification_code: &str,
) -> Result<(), String> {
    let email_service_url = env::var("EMAIL_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8004".to_string());

    let request = VerificationEmailRequest {
        to_email: to_email.to_string(),
        to_name: to_name.to_string(),
        verification_code: verification_code.to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/send-verification", email_service_url))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to email service: {}", e))?;

    let status = response.status();
    
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        return Err(format!(
            "Email service returned error ({}): {}",
            status, error_body
        ));
    }

    let email_response: EmailServiceResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse email service response: {}", e))?;

    if !email_response.success {
        return Err(format!("Email service failed: {}", email_response.message));
    }

    Ok(())
}

pub fn generate_otp() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1000000))
}