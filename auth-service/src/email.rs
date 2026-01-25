use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;

use std::env;

pub fn send_verification_email(
    to_email: &str,
    to_name: &str,
    verification_code: &str,
) -> Result<(),
        String> {
    let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587);
    let smtp_username = env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME not set")?;
    let smtp_password = env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD not set")?;
    let smtp_from = env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@handshake.local".to_string());

    let email_body = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background-color: #4F46E5; color: white; padding: 20px; text-align: center; }}
        .content {{ background-color: #f4f4f4; padding: 30px; }}
        .code {{ font-size: 32px; font-weight: bold; color: #4F46E5; text-align: center; letter-spacing: 5px; padding: 20px; background: white; border-radius: 5px; }}
        .footer {{ text-align: center; padding: 20px; font-size: 12px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Handshake Marketplace</h1>
        </div>
        <div class="content">
            <h2>Welcome, {}!</h2>
            <p>Thank you for registering with Handshake Marketplace. To complete your registration, please verify your email address using the code below:</p>
            <div class="code">{}</div>
            <p>This code will expire in 15 minutes.</p>
            <p>If you didn't create an account with Handshake, please ignore this email.</p>
        </div>
        <div class="footer">
            <p>© 2026 Handshake Marketplace. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
        "#,
        to_name, verification_code
    );

    let email = Message::builder()
        .from(smtp_from.parse().map_err(|_| "Invalid FROM email")?)
        .to(to_email.parse().map_err(|_| "Invalid TO email")?)
        .subject("Verify your Handshake account")
        .header(ContentType::TEXT_HTML)
        .body(email_body)
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .port(smtp_port)
        .credentials(creds)
        .build();

    mailer
        .send(&email)
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}

pub fn generate_otp() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1000000))
}
