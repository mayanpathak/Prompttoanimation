
use axum::{
    routing::{get, post},
    Router,
};

use crate::controller::authcontroller::{
    signup,
    login,
    logout,
    forgot_password,
    reset_password,
    check_auth,
};
use crate::AppState;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        // .route("/verify-email", post(verify_email))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password/{token}", post(reset_password))
        .route("/check-auth", get(check_auth))
}