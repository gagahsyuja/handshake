use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use tera::{Context, Tera};

pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
}

impl EmailConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(EmailConfig {
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_username: env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME not set")?,
            smtp_password: env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD not set")?,
            smtp_from: env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@handshake.local".to_string()),
        })
    }
}

pub fn send_email(to_email: &str, subject: &str, body: String) -> Result<(), String> {
    let config = EmailConfig::from_env()?;

    let email = Message::builder()
        .from(config.smtp_from.parse().map_err(|_| "Invalid FROM email")?)
        .to(to_email.parse().map_err(|_| "Invalid TO email")?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body)
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(config.smtp_username, config.smtp_password);

    let mailer = SmtpTransport::starttls_relay(&config.smtp_host)
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    mailer
        .send(&email)
        .map_err(|e| format!("Failed to send email: {}", e))?;

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
