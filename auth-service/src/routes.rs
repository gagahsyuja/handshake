use chrono::{Duration, Utc};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use serde::{Deserialize, Serialize};

use crate::auth::{create_jwt, AuthenticatedUser};
use crate::db::DbConn;
use crate::email::{generate_otp, send_verification_email};
use crate::models::{EmailVerification, NewEmailVerification, NewUser, User};
use crate::schema::{email_verifications, users};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResendOtpRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub email_verified: bool,
}

#[post("/register", data = "<request>")]
pub async fn register(
    db: DbConn,
    request: Json<RegisterRequest>,
) -> Result<Json<MessageResponse>, Status> {
    // Check if user already exists
    let email = request.email.clone();
    let existing_user = db
        .run(move |conn| {
            users::table
                .filter(users::email.eq(&email))
                .first::<User>(conn)
                .optional()
        })
        .await
        .map_err(|_| Status::InternalServerError)?;

    if existing_user.is_some() {
        return Err(Status::Conflict);
    }

    // Hash password
    let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
        .map_err(|_| Status::InternalServerError)?;

    // Create user
    let new_user = NewUser {
        email: request.email.clone(),
        password_hash,
        name: request.name.clone(),
    };

    let user: User = db
        .run(move |conn| {
            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result(conn)
        })
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Generate OTP
    let otp_code = generate_otp();
    let user_id = user.id;
    let otp_code_clone = otp_code.clone();

    let new_verification = NewEmailVerification {
        user_id,
        code: otp_code_clone,
        expires_at: (Utc::now() + Duration::minutes(15)).naive_utc(),
    };

    db.run(move |conn| {
        diesel::insert_into(email_verifications::table)
            .values(&new_verification)
            .execute(conn)
    })
    .await
    .map_err(|_| Status::InternalServerError)?;

    // Send verification email
    send_verification_email(&user.email, &user.name, &otp_code)
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(MessageResponse {
        message: "Registration successful. Please check your email for verification code."
            .to_string(),
    }))
}

#[post("/verify-email", data = "<request>")]
pub async fn verify_email(
    db: DbConn,
    request: Json<VerifyEmailRequest>,
) -> Result<Json<MessageResponse>, Status> {
    let email = request.email.clone();
    let code = request.code.clone();

    // Get user
    let user: User = db
        .run(move |conn| users::table.filter(users::email.eq(&email)).first(conn))
        .await
        .map_err(|_| Status::NotFound)?;

    if user.email_verified {
        return Err(Status::BadRequest);
    }

    let user_id = user.id;

    // Check verification code
    let verification: EmailVerification = db
        .run(move |conn| {
            email_verifications::table
                .filter(email_verifications::user_id.eq(user_id))
                .filter(email_verifications::code.eq(&code))
                .order(email_verifications::created_at.desc())
                .first(conn)
        })
        .await
        .map_err(|_| Status::Unauthorized)?;

    // Check if code is expired
    if verification.expires_at < Utc::now().naive_utc() {
        return Err(Status::Gone);
    }

    // Update user as verified
    let user_id = user.id;
    db.run(move |conn| {
        diesel::update(users::table.find(user_id))
            .set(users::email_verified.eq(true))
            .execute(conn)
    })
    .await
    .map_err(|_| Status::InternalServerError)?;

    // Delete used verification codes
    let user_id = user.id;
    db.run(move |conn| {
        diesel::delete(email_verifications::table.filter(email_verifications::user_id.eq(user_id)))
            .execute(conn)
    })
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(MessageResponse {
        message: "Email verified successfully. You can now login.".to_string(),
    }))
}

#[post("/login", data = "<request>")]
pub async fn login(db: DbConn, request: Json<LoginRequest>) -> Result<Json<AuthResponse>, Status> {
    let email = request.email.clone();
    let password = request.password.clone();

    let user: User = db
        .run(move |conn| users::table.filter(users::email.eq(&email)).first(conn))
        .await
        .map_err(|_| Status::Unauthorized)?;

    // Check if email is verified
    if !user.email_verified {
        return Err(Status::Forbidden);
    }

    // Verify password
    let valid =
        bcrypt::verify(&password, &user.password_hash).map_err(|_| Status::InternalServerError)?;

    if !valid {
        return Err(Status::Unauthorized);
    }

    // Create JWT
    let token = create_jwt(user.id, user.email.clone()).map_err(|_| Status::InternalServerError)?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            email_verified: user.email_verified,
        },
    }))
}

#[post("/resend-otp", data = "<request>")]
pub async fn resend_otp(
    db: DbConn,
    request: Json<ResendOtpRequest>,
) -> Result<Json<MessageResponse>, Status> {
    let email = request.email.clone();

    let user: User = db
        .run(move |conn| users::table.filter(users::email.eq(&email)).first(conn))
        .await
        .map_err(|_| Status::NotFound)?;

    if user.email_verified {
        return Err(Status::BadRequest);
    }

    // Delete old verification codes
    let user_id = user.id;
    db.run(move |conn| {
        diesel::delete(email_verifications::table.filter(email_verifications::user_id.eq(user_id)))
            .execute(conn)
    })
    .await
    .map_err(|_| Status::InternalServerError)?;

    // Generate new OTP
    let otp_code = generate_otp();
    let user_id = user.id;
    let otp_code_clone = otp_code.clone();

    let new_verification = NewEmailVerification {
        user_id,
        code: otp_code_clone,
        expires_at: (Utc::now() + Duration::minutes(15)).naive_utc(),
    };

    db.run(move |conn| {
        diesel::insert_into(email_verifications::table)
            .values(&new_verification)
            .execute(conn)
    })
    .await
    .map_err(|_| Status::InternalServerError)?;

    // Send verification email
    send_verification_email(&user.email, &user.name, &otp_code)
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(MessageResponse {
        message: "Verification code resent. Please check your email.".to_string(),
    }))
}

#[get("/me")]
pub async fn me(db: DbConn, auth: AuthenticatedUser) -> Result<Json<UserResponse>, Status> {
    let user_id = auth.user_id;

    let user: User = db
        .run(move |conn| users::table.find(user_id).first(conn))
        .await
        .map_err(|_| Status::NotFound)?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        email_verified: user.email_verified,
    }))
}
