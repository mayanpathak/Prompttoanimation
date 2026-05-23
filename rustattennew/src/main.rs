use axum::{
    routing::get,
    response::IntoResponse,
    http::{StatusCode, Method, HeaderValue, header},
    Router,
};

use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};

use dotenvy::dotenv;

use std::{
    env,
    net::SocketAddr,
};

use tokio::net::TcpListener;

use deadpool_redis::{
    redis::AsyncCommands,
    Pool,
};

mod db;
mod routes;
mod controller;
mod middleware;
mod utils;
mod models;
mod schema;
mod services;
mod config;

// use routes::auth_routes;
use routes::{auth_routes, job_routes};


use crate::config::redis::create_redis_pool;
#[derive(Clone)]
pub struct AppState {
    mongo_client: mongodb::Client,
    db: mongodb::Database,

    // Redis
    redis_pool: Pool,

    // Email
    email_host: String,
    email_port: u16,
    email_user: String,
    email_pass: String,
    sender_name: String,

    // JWT
    jwt_secret: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // =========================
    // Environment Variables
    // =========================

    let port = env::var("PORT")
        .unwrap_or_else(|_| "5000".to_string());

    let env_mode = env::var("NODE_ENV")
        .unwrap_or_else(|_| "development".to_string());

    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid address");

    // =========================
    // MongoDB Connection
    // =========================

    let (mongo_client, db) = db::connect_db().await;

    println!("✅ MongoDB connected successfully");

    // =========================
    // Redis Connection
    // =========================

    let redis_pool = create_redis_pool().await;

    match redis_pool.get().await {
        Ok(mut conn) => {
let pong: Result<String, _> =
    deadpool_redis::redis::cmd("PING")
        .query_async(&mut conn)
        .await;
            match pong {
                Ok(response) => {
                    println!("✅ Redis connected successfully: {}", response);
                }

                Err(err) => {
                    eprintln!("❌ Redis ping failed: {:?}", err);
                    std::process::exit(1);
                }
            }
        }

        Err(err) => {
            eprintln!("❌ Failed to connect to Redis: {:?}", err);
            std::process::exit(1);
        }
    }

    // =========================
    // App State
    // =========================

    let state = AppState {
        mongo_client,
        db,

        redis_pool,

        email_host: env::var("EMAIL_HOST")
            .unwrap_or_default(),

        email_port: env::var("EMAIL_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587),

        email_user: env::var("EMAIL_USER")
            .unwrap_or_default(),

        email_pass: env::var("EMAIL_PASS")
            .unwrap_or_default(),

        sender_name: env::var("SENDER_NAME")
            .unwrap_or_default(),

        jwt_secret: env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set"),
    };

    // =========================
    // CORS
    // =========================

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:5173"
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allow_credentials(true);

    // =========================
    // API Routes
    // =========================

    let api_routes = Router::new()
        .nest("/api/auth", auth_routes())
        .nest("/api/jobs", job_routes())
        .with_state(state);

    // =========================
    // App Router
    // =========================

    let app = if env_mode == "production" {
        Router::new()
            .merge(api_routes)
            .nest_service("/", ServeDir::new("frontend/dist"))
            .fallback(get(index_handler))
            .layer(cors)
    } else {
        Router::new()
            .merge(api_routes)
            .layer(cors)
    };

    println!("🚀 Server running on http://{}", addr);

    // =========================
    // TCP Listener
    // =========================

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn index_handler() -> impl IntoResponse {
    match tokio::fs::read_to_string("frontend/dist/index.html").await {
        Ok(contents) => (StatusCode::OK, contents),

        Err(_) => (
            StatusCode::NOT_FOUND,
            "index.html not found".to_string(),
        ),
    }
}

// hiiiiii