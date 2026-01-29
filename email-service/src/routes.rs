use rocket::http::Status;
use rocket::post;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::smtp::{render_order_notification, render_verification_email, send_email};

#[derive(Debug, Deserialize)]
pub struct VerificationEmailRequest {
    pub to_email: String,
    pub to_name: String,
    pub verification_code: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderNotificationRequest {
    pub to_email: String,
    pub to_name: String,
    pub product_title: String,
    pub order_id: i32,
    pub midpoint_address: String,
}

#[derive(Debug, Deserialize)]
pub struct CustomEmailRequest {
    pub to_email: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct EmailResponse {
    pub success: bool,
    pub message: String,
}

#[post("/send-verification", data = "<request>")]
pub async fn send_verification(
    request: Json<VerificationEmailRequest>,
) -> Result<Json<EmailResponse>, Status> {
    let body = render_verification_email(&request.to_name, &request.verification_code)
        .map_err(|_| Status::InternalServerError)?;

    send_email(&request.to_email, "Verify your Handshake account", body)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(EmailResponse {
        success: true,
        message: "Verification email sent successfully".to_string(),
    }))
}

#[post("/send-order-notification", data = "<request>")]
pub async fn send_order_notification(
    request: Json<OrderNotificationRequest>,
) -> Result<Json<EmailResponse>, Status> {
    let body = render_order_notification(
        &request.to_name,
        &request.product_title,
        request.order_id,
        &request.midpoint_address,
    )
    .map_err(|_| Status::InternalServerError)?;

    send_email(
        &request.to_email,
        &format!("Order Confirmation - {}", request.product_title),
        body,
    )
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(EmailResponse {
        success: true,
        message: "Order notification sent successfully".to_string(),
    }))
}

#[post("/send-custom", data = "<request>")]
pub async fn send_custom_email(request: Json<CustomEmailRequest>) -> Result<Json<EmailResponse>, Status> {
    send_email(&request.to_email, &request.subject, request.body.clone())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(EmailResponse {
        success: true,
        message: "Email sent successfully".to_string(),
    }))
}