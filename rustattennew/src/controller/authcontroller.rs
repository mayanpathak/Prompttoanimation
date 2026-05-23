use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use mongodb::bson::{doc, oid::ObjectId, to_bson};
use mongodb::Collection;
use serde_json::json;

use crate::models::User;
use crate::utils::{
    cookie::{build_auth_cookie, clear_auth_cookie},
    hash::{hash_password, verify_password},
    jwt::generate_token,
};
use crate::schema::{LoginForm, SignupForm};
use crate::AppState;

// ─── HELPERS ──────────────────────────────────────────────────────────────────

macro_rules! internal_err {
    () => {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": "Internal server error" })),
        )
            .into_response()
    };
}

// ─── SIGNUP ──────────────────────────────────────────────────────────────────

pub async fn signup(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<SignupForm>,
) -> impl IntoResponse {
    let collection: Collection<User> = state.db.collection("users");

    match collection.find_one(doc! { "email": &body.email }, None).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({ "success": false, "message": "User already exists" })),
            )
                .into_response();
        }
        Err(_) => return internal_err!(),
        _ => {}
    }

    let hashed = match hash_password(&body.password) {
        Ok(h) => h,
        Err(_) => return internal_err!(),
    };

    let now = chrono::Utc::now();
    let new_user = User {
        id: None,
        username: body.username,
        email: body.email.clone(),
        password: hashed,
        is_verified: true,
        last_login: now,
        reset_password_token: None,
        reset_password_expires_at: None,
        created_at: now,
        updated_at: now,
    };

    let inserted_id = match collection.insert_one(&new_user, None).await {
        Ok(r) => match r.inserted_id.as_object_id() {
            Some(oid) => oid.to_hex(),
            None => return internal_err!(),
        },
        Err(_) => return internal_err!(),
    };

    let token = match generate_token(inserted_id, &state.jwt_secret) {
        Ok(t) => t,
        Err(_) => return internal_err!(),
    };

    (
        StatusCode::CREATED,
        jar.add(build_auth_cookie(token)),
        Json(json!({
            "success": true,
            "message": "User created successfully",
            "user": {
                "email": new_user.email,
                "username": new_user.username,
            }
        })),
    )
        .into_response()
}

// ─── LOGIN ────────────────────────────────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginForm>,
) -> impl IntoResponse {
    let collection: Collection<User> = state.db.collection("users");

    let user = match collection.find_one(doc! { "email": &body.email }, None).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "success": false, "message": "Invalid credentials" })),
            )
                .into_response();
        }
        Err(_) => return internal_err!(),
    };

    match verify_password(&body.password, &user.password) {
        Ok(true) => {}
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "success": false, "message": "Invalid credentials" })),
            )
                .into_response();
        }
    }

    let user_id = match user.id {
        Some(oid) => oid,
        None => return internal_err!(),
    };

    let now = chrono::Utc::now();
    let now_bson = match to_bson(&now) {
        Ok(b) => b,
        Err(_) => return internal_err!(),
    };

    let _ = collection
        .update_one(
            doc! { "_id": user_id },
            doc! { "$set": { "last_login": &now_bson, "updated_at": &now_bson } },
            None,
        )
        .await;

    let token = match generate_token(user_id.to_hex(), &state.jwt_secret) {
        Ok(t) => t,
        Err(_) => return internal_err!(),
    };

    (
        StatusCode::OK,
        jar.add(build_auth_cookie(token)),
        Json(json!({
            "success": true,
            "message": "Logged in successfully",
            "user": {
                "email": user.email,
                "username": user.username,
            }
        })),
    )
        .into_response()
}

// ─── LOGOUT ───────────────────────────────────────────────────────────────────

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    (
        StatusCode::OK,
        jar.add(clear_auth_cookie()),
        Json(json!({ "success": true, "message": "Logged out successfully" })),
    )
        .into_response()
}

// ─── FORGOT PASSWORD ──────────────────────────────────────────────────────────

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let email = match body.get("email").and_then(|v| v.as_str()) {
        Some(e) => e.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Email is required" })),
            )
                .into_response();
        }
    };

    let collection: Collection<User> = state.db.collection("users");

    let user = match collection.find_one(doc! { "email": &email }, None).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "message": "User not found" })),
            )
                .into_response();
        }
        Err(_) => return internal_err!(),
    };

    let user_id = match user.id {
        Some(oid) => oid,
        None => return internal_err!(),
    };

    let reset_token: String = (0..20).map(|_| format!("{:02x}", rand::random::<u8>())).collect();

    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    let expires_bson = match to_bson(&expires_at) {
        Ok(b) => b,
        Err(_) => return internal_err!(),
    };
    let updated_bson = match to_bson(&chrono::Utc::now()) {
        Ok(b) => b,
        Err(_) => return internal_err!(),
    };

    let _ = collection
        .update_one(
            doc! { "_id": user_id },
            doc! { "$set": {
                "reset_password_token": &reset_token,
                "reset_password_expires_at": expires_bson,
                "updated_at": updated_bson,
            }},
            None,
        )
        .await;

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Password reset link sent to your email",
            "debug_token": reset_token  // remove in production
        })),
    )
        .into_response()
}

// ─── RESET PASSWORD ───────────────────────────────────────────────────────────

pub async fn reset_password(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let password = match body.get("password").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Password is required" })),
            )
                .into_response();
        }
    };

    let collection: Collection<User> = state.db.collection("users");

    let now = chrono::Utc::now();
    let now_bson = match to_bson(&now) {
        Ok(b) => b,
        Err(_) => return internal_err!(),
    };

    let user = match collection
        .find_one(
            doc! {
                "reset_password_token": &token,
                "reset_password_expires_at": { "$gt": &now_bson }
            },
            None,
        )
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Invalid or expired reset token" })),
            )
                .into_response();
        }
        Err(_) => return internal_err!(),
    };

    let user_id = match user.id {
        Some(oid) => oid,
        None => return internal_err!(),
    };

    let hashed = match hash_password(&password) {
        Ok(h) => h,
        Err(_) => return internal_err!(),
    };

    let _ = collection
        .update_one(
            doc! { "_id": user_id },
            doc! {
                "$set": {
                    "password": hashed,
                    "updated_at": &now_bson,
                },
                "$unset": {
                    "reset_password_token": "",
                    "reset_password_expires_at": "",
                }
            },
            None,
        )
        .await;

    (
        StatusCode::OK,
        Json(json!({ "success": true, "message": "Password reset successful" })),
    )
        .into_response()
}

// ─── CHECK AUTH ───────────────────────────────────────────────────────────────

pub async fn check_auth(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let token = match jar.get("token") {
        Some(c) => c.value().to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "success": false, "message": "Not authenticated" })),
            )
                .into_response();
        }
    };

    let claims = match crate::utils::jwt::verify_token(&token, &state.jwt_secret) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "success": false, "message": "Invalid or expired token" })),
            )
                .into_response();
        }
    };

    let oid = match ObjectId::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "message": "Invalid user ID" })),
            )
                .into_response();
        }
    };

    let collection: Collection<User> = state.db.collection("users");

    match collection.find_one(doc! { "_id": oid }, None).await {
        Ok(Some(u)) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "user": {
                    "email": u.email,
                    "username": u.username,
                    "last_login": u.last_login,
                    "created_at": u.created_at,
                }
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "message": "User not found" })),
        )
            .into_response(),
        Err(_) => internal_err!(),
    }
} 