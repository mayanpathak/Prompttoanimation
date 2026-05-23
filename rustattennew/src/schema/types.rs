use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginForm {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SignupForm {
    #[validate(length(min = 3))]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRenderJobForm {
    #[validate(length(min = 10, max = 2000))]
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
}