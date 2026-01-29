use base64::engine::general_purpose;
use base64::Engine;
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use tera::{Context, Tera};

pub struct EmailConfig {
    pub mailjet_api_key: String,
    pub mailjet_secret_key: String,
    pub from_email: String,
    pub from_name: String,
}

impl EmailConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(EmailConfig {
            mailjet_api_key: env::var("MAILJET_API_KEY")
                .map_err(|_| "MAILJET_API_KEY not set")?,
            mailjet_secret_key: env::var("MAILJET_SECRET_KEY")
                .map_err(|_| "MAILJET_SECRET_KEY not set")?,
            from_email: env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@handshake.local".to_string()),
            from_name: env::var("FROM_NAME")
                .unwrap_or_else(|_| "Handshake Marketplace".to_string()),
        })
    }
}

#[derive(Debug, Serialize)]
struct MailjetRecipient {
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct MailjetMessage {
    #[serde(rename = "From")]
    from: MailjetRecipient,
    #[serde(rename = "To")]
    to: Vec<MailjetRecipient>,
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "HTMLPart")]
    html_part: String,
}

#[derive(Debug, Serialize)]
struct MailjetRequest {
    #[serde(rename = "Messages")]
    messages: Vec<MailjetMessage>,
}

#[derive(Debug, Deserialize)]
struct MailjetResponse {
    #[serde(rename = "Messages")]
    messages: Vec<MailjetMessageResponse>,
}

#[derive(Debug, Deserialize)]
struct MailjetMessageResponse {
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "To", default)]
    to: Vec<MailjetRecipientResponse>,
}

#[derive(Debug, Deserialize)]
struct MailjetRecipientResponse {
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "MessageID")]
    message_id: i64,
}

pub async fn send_email(to_email: &str, subject: &str, body: String) -> Result<(), String> {
    let config = EmailConfig::from_env()?;

    let mailjet_request = MailjetRequest {
        messages: vec![MailjetMessage {
            from: MailjetRecipient {
                email: config.from_email,
                name: Some(config.from_name),
            },
            to: vec![MailjetRecipient {
                email: to_email.to_string(),
                name: None,
            }],
            subject: subject.to_string(),
            html_part: body,
        }],
    };

    let client = reqwest::Client::new();
    let auth = general_purpose::STANDARD.encode(format!(
        "{}:{}",
        config.mailjet_api_key, config.mailjet_secret_key
    ));

    let response = client
        .post("https://api.mailjet.com/v3.1/send")
        .header("Authorization", format!("Basic {}", auth))
        .header("Content-Type", "application/json")
        .json(&mailjet_request)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to Mailjet: {}", e))?;

    let status = response.status();
    
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        return Err(format!(
            "Mailjet API error ({}): {}",
            status, error_body
        ));
    }

    let mailjet_response: MailjetResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Mailjet response: {}", e))?;

    if let Some(message) = mailjet_response.messages.first() {
        if message.status != "success" {
            return Err(format!("Mailjet message status: {}", message.status));
        }
    } else {
        return Err("No messages in Mailjet response".to_string());
    }

    Ok(())
}

pub fn render_verification_email(name: &str, code: &str) -> Result<String, String> {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "verification",
        include_str!("../templates/verification.html"),
    )
    .map_err(|e| format!("Failed to load template: {}", e))?;

    let mut context = Context::new();
    context.insert("name", name);
    context.insert("code", code);

    tera.render("verification", &context)
        .map_err(|e| format!("Failed to render template: {}", e))
}

pub fn render_order_notification(
    name: &str,
    product_title: &str,
    order_id: i32,
    midpoint_address: &str,
) -> Result<String, String> {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "order",
        include_str!("../templates/order_notification.html"),
    )
    .map_err(|e| format!("Failed to load template: {}", e))?;

    let mut context = Context::new();
    context.insert("name", name);
    context.insert("product_title", product_title);
    context.insert("order_id", &order_id);
    context.insert("midpoint_address", midpoint_address);

    tera.render("order", &context)
        .map_err(|e| format!("Failed to render template: {}", e))
}