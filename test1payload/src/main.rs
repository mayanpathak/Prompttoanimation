
use colored::*;
use reqwest::{cookie::Jar, Client};
use serde_json::{json, Value};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tracing::{error, info, warn};

use tracing_subscriber::{
    fmt,
    prelude::*,
    EnvFilter,
};

const BASE_URL: &str = "http://localhost:5000";
const MAX_POLL_ATTEMPTS: usize = 60;

// Hardcoded test user
const USERNAME: &str = "testuser123";
const EMAIL: &str = "testuser1233@gmail.com";
const PASSWORD: &str = "StrongPassword123!";

// Hardcoded payload
const PROMPT: &str =
    "Create a 30-second Manim animation explaining a neural network visually.

Start with floating dots representing neurons. Lines connect layer by layer while signals pulse through the network.

Display a simple equation like:

y=σ(Wx+b)
w
b

Animate inputs entering the network, weights glowing during propagation, and outputs forming recognizable patterns like digits or shapes.

Final scene zooms out to reveal the network forming the shape of a human brain.

Closing text:
“Learning Through Connections”

Style:

Futuristic dark theme
Electric blue signal pulses
Smooth node animations
Minimalist design";

#[derive(Clone, Debug)]
struct TestUser {
    username: String,
    email: String,
    password: String,
    cookie_jar: Arc<Jar>,
}

#[tokio::main]
async fn main() {
    init_tracing();

    banner();

    let started = Instant::now();

    let user = TestUser {
        username: USERNAME.to_string(),
        email: EMAIL.to_string(),
        password: PASSWORD.to_string(),
        cookie_jar: Arc::new(Jar::default()),
    };

    info!(
        "{} {}",
        "TEST USER".bright_blue().bold(),
        user.email.bright_white()
    );

    info!(
        "{} {}",
        "PROMPT".bright_blue().bold(),
        PROMPT.bright_white()
    );

    match run_single_flow(user).await {
        Ok(video_path) => {
            println!();

            println!(
                "{}",
                "════════════════════════════════════════════════════"
                    .bright_black()
            );

            println!(
                "{}",
                "                TEST SUCCESS"
                    .green()
                    .bold()
            );

            println!(
                "{}",
                "════════════════════════════════════════════════════"
                    .bright_black()
            );

            println!(
                "{} {}",
                "VIDEO PATH :".bright_magenta().bold(),
                video_path.bright_white()
            );

            println!(
                "{} {} ms",
                "TOTAL TIME :".yellow().bold(),
                started.elapsed().as_millis()
            );
        }

        Err(err) => {
            println!();

            println!(
                "{}",
                "════════════════════════════════════════════════════"
                    .bright_black()
            );

            println!(
                "{}",
                "                 TEST FAILED"
                    .red()
                    .bold()
            );

            println!(
                "{}",
                "════════════════════════════════════════════════════"
                    .bright_black()
            );

            println!(
                "{} {}",
                "ERROR :".red().bold(),
                err.bright_red()
            );
        }
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .without_time()
                .compact(),
        )
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

fn banner() {
    println!();

    println!(
        "{}",
        "════════════════════════════════════════════════════"
            .bright_black()
    );

    println!(
        "{}",
        "          🚀 SINGLE PAYLOAD TEST"
            .bright_white()
            .bold()
    );

    println!(
        "{}",
        "════════════════════════════════════════════════════"
            .bright_black()
    );

    println!(
        "{} {}",
        "BASE URL :".bright_blue().bold(),
        BASE_URL.bright_white()
    );

    println!(
        "{} {}",
        "MAX POLL :".bright_blue().bold(),
        MAX_POLL_ATTEMPTS.to_string().bright_white()
    );

    println!(
        "{}",
        "════════════════════════════════════════════════════"
            .bright_black()
    );

    println!();
}

async fn run_single_flow(
    user: TestUser,
) -> Result<String, String> {
    let client = build_client(user.cookie_jar.clone())?;

    signup_or_login(&client, &user).await?;

    let job_id =
        submit_render_job(&client).await?;

    info!(
        "{} {}",
        "JOB ID".green().bold(),
        job_id.bright_white()
    );

    let video_path =
        poll_job_status(&client, &job_id).await?;

    Ok(video_path)
}

fn build_client(
    cookie_jar: Arc<Jar>,
) -> Result<Client, String> {
    Client::builder()
        .cookie_provider(cookie_jar)
        .pool_idle_timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())
}

async fn signup_or_login(
    client: &Client,
    user: &TestUser,
) -> Result<(), String> {
    let signup_response = client
        .post(format!(
            "{}/api/auth/signup",
            BASE_URL
        ))
        .json(&json!({
            "username": user.username,
            "email": user.email,
            "password": user.password,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if signup_response.status().is_success() {
        info!(
            "{} {}",
            "SIGNUP".green().bold(),
            user.email.bright_white()
        );
    } else {
        warn!(
            "{} {}",
            "SIGNUP SKIPPED".yellow().bold(),
            user.email.bright_white()
        );
    }

    let login_response = client
        .post(format!(
            "{}/api/auth/login",
            BASE_URL
        ))
        .json(&json!({
            "email": user.email,
            "password": user.password,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = login_response.status();

    let body: Value = login_response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if status.is_success()
        && body["success"].as_bool().unwrap_or(false)
    {
        info!(
            "{} {}",
            "LOGIN".bright_green().bold(),
            user.email.bright_white()
        );

        Ok(())
    } else {
        error!(
            "{} {}",
            "LOGIN FAILED".red().bold(),
            user.email.bright_white()
        );

        Err(format!(
            "Login failed: {:?}",
            body
        ))
    }
}

async fn submit_render_job(
    client: &Client,
) -> Result<String, String> {
    info!(
        "{} {}",
        "SUBMITTING JOB".bright_blue().bold(),
        PROMPT.bright_white()
    );

    let response = client
        .post(format!("{}/api/jobs", BASE_URL))
        .json(&json!({
            "prompt": PROMPT
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();

    let body: Value = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!(
            "Job submission failed: {:?}",
            body
        ));
    }

    body["data"]["job_id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or("Missing job_id".into())
}

async fn poll_job_status(
    client: &Client,
    job_id: &str,
) -> Result<String, String> {
    for attempt in 1..=MAX_POLL_ATTEMPTS {
        info!(
            "{} {}",
            "POLL ATTEMPT".bright_blue().bold(),
            attempt.to_string().bright_white()
        );

        let response = client
            .get(format!(
                "{}/api/jobs/{}",
                BASE_URL,
                job_id
            ))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let status = body["data"]["status"]
            .as_str()
            .unwrap_or("Unknown");

        match status {
            "Completed" => {
                let path = body["data"]["video_path"]
                    .as_str()
                    .unwrap_or("missing");

                info!(
                    "{} {}",
                    "JOB COMPLETED".green().bold(),
                    path.bright_white()
                );

                return Ok(path.to_string());
            }

            "Failed" => {
                return Err(
                    body["error_message"]
                        .as_str()
                        .unwrap_or("Unknown failure")
                        .to_string(),
                );
            }

            _ => {
                info!(
                    "{} {}",
                    "CURRENT STATUS".yellow().bold(),
                    status.bright_white()
                );

                sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Err("Polling timeout".into())
}




//  hiiii