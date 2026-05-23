
mod config;
mod errors;
mod models;
mod mongo;
mod processor;
mod redis;
mod worker;
mod configgemini;
mod concurrent;
mod cleanup;
mod renderer;
mod utils;
mod generator;
mod filesystem;
mod docker;



mod services {
    pub mod job_service;
}



use crate::configgemini::GeminiConfig;
use deadpool_redis::Pool;
use mongodb::Database;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis_pool: Pool,
    pub gemini_config: GeminiConfig,
}

#[tokio::main]
async fn main() {

    // =====================================
    // Load environment variables
    // =====================================

    dotenvy::dotenv().ok();

     let gemini_config = GeminiConfig::from_env();

    println!("Gemini model: {}", gemini_config.model);

    

    println!("Starting worker runtime...");

    // =====================================
    // Connect MongoDB
    // =====================================

    let (_client, db) = mongo::connect_db().await;

    println!("MongoDB connected.");

    // =====================================
    // Connect Redis
    // =====================================

    let redis_pool = redis::create_redis_pool().await;

    println!("Redis connected.");

    // =====================================
    // Shared app state
    // =====================================

    let state = AppState {
        db,
        redis_pool,
        gemini_config,
    };

    let state = Arc::new(state);

    // =====================================
    // Start worker runtime
    // =====================================

    worker::start_worker(state).await;
}