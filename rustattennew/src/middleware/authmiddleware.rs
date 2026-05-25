


use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use axum_extra::extract::cookie::CookieJar;

use mongodb::bson::oid::ObjectId;

use crate::utils::verify_token;

pub async fn auth_middleware(
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {

    // JWT secret
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Read token
    let token = match jar.get("token") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Verify token
    let claims = match verify_token(&token, &secret) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Convert string -> ObjectId
    let object_id = ObjectId::parse_str(&claims.user_id)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Insert ObjectId into extensions
    req.extensions_mut().insert(object_id);

    Ok(next.run(req).await)
}