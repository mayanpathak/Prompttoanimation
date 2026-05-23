mod test_payloads;
mod test_users;

use colored::*;
use futures::future::join_all;
use reqwest::{cookie::Jar, Client};
use serde_json::{json, Value};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{task, time::sleep};
use tracing::{debug, error, info, warn};

use tracing_subscriber::{
    fmt,
    EnvFilter,
    prelude::*,
};

use test_payloads::generate_test_payloads;
use test_users::generate_test_users;

const BASE_URL: &str = "http://localhost:5000";
const TOTAL_USERS: usize = 50;
const MAX_POLL_ATTEMPTS: usize = 60;

#[derive(Clone, Debug)]
struct TestUser {
    username: String,
    email: String,
    password: String,
    cookie_jar: Arc<Jar>,
}

#[derive(Debug)]
struct JobResult {
    user_index: usize,
    email: String,
    job_id: String,
    success: bool,
    duration_ms: u128,
    video_path: Option<String>,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    init_tracing();

    banner();

    let started = Instant::now();

    let payloads = generate_test_payloads();
    let users_data = generate_test_users(TOTAL_USERS);

    info!(
        "{} {}",
        "Loaded users:".bright_blue(),
        TOTAL_USERS.to_string().bright_white()
    );

    info!(
        "{} {}",
        "Loaded payloads:".bright_blue(),
        payloads.len().to_string().bright_white()
    );

    let mut tasks = vec![];

    for (i, user_data) in users_data.into_iter().enumerate() {
        let payload = payloads[i % payloads.len()].clone();

        let user = TestUser {
            username: user_data["username"]
                .as_str()
                .unwrap()
                .to_string(),

            email: user_data["email"]
                .as_str()
                .unwrap()
                .to_string(),

            password: user_data["password"]
                .as_str()
                .unwrap()
                .to_string(),

            cookie_jar: Arc::new(Jar::default()),
        };

        tasks.push(task::spawn(run_user_flow(
            i + 1,
            user,
            payload["prompt"].as_str().unwrap().to_string(),
        )));
    }

    let results = join_all(tasks).await;

    let mut success = 0usize;
    let mut failed = 0usize;

    println!();

    println!(
        "{}",
        "══════════════════════════════════════════════════════════════"
            .bright_black()
    );

    println!(
        "{}",
        "                    FINAL TEST RESULTS"
            .bright_white()
            .bold()
    );

    println!(
        "{}",
        "══════════════════════════════════════════════════════════════"
            .bright_black()
    );

    for result in results {
        match result {
            Ok(Ok(job)) => {
                success += 1;

                println!(
                    "{} {} {} {} {} {}",
                    "✅".green(),
                    format!("[USER #{:02}]", job.user_index)
                        .bright_cyan()
                        .bold(),
                    job.email.bright_white(),
                    "→".bright_black(),
                    "SUCCESS".green().bold(),
                    format!("({} ms)", job.duration_ms)
                        .bright_black()
                );

                println!(
                    "   {} {}",
                    "JOB".bright_blue(),
                    job.job_id.bright_white()
                );

                if let Some(path) = job.video_path {
                    println!(
                        "   {} {}",
                        "VIDEO".bright_magenta(),
                        path.bright_white()
                    );
                }
            }

            Ok(Err(err)) => {
                failed += 1;

                println!(
                    "{} {}",
                    "❌ JOB FAILED".red().bold(),
                    err.bright_red()
                );
            }

            Err(join_err) => {
                failed += 1;

                println!(
                    "{} {}",
                    "❌ TASK PANIC".red().bold(),
                    join_err.to_string().bright_red()
                );
            }
        }

        println!(
            "{}",
            "──────────────────────────────────────────────────────────────"
                .bright_black()
        );
    }

    let total = success + failed;
    let success_rate = if total > 0 {
        (success as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!();

    println!(
        "{}",
        "══════════════════════════════════════════════════════════════"
            .bright_black()
    );

    println!(
        "{} {}",
        "✅ SUCCESSFUL JOBS :".green().bold(),
        success.to_string().bright_white()
    );

    println!(
        "{} {}",
        "❌ FAILED JOBS     :".red().bold(),
        failed.to_string().bright_white()
    );

    println!(
        "{} {:.2}%",
        "📈 SUCCESS RATE    :".bright_blue().bold(),
        success_rate
    );

    println!(
        "{} {} ms",
        "⏱ TOTAL TIME      :".yellow().bold(),
        started.elapsed().as_millis()
    );

    println!(
        "{}",
        "══════════════════════════════════════════════════════════════"
            .bright_black()
    );
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
        "══════════════════════════════════════════════════════════════"
            .bright_black()
    );

    println!(
        "{}",
        "        🚀 WORKER QUEUE STRESS TEST SUITE"
            .bright_white()
            .bold()
    );

    println!(
        "{}",
        "══════════════════════════════════════════════════════════════"
            .bright_black()
    );

    println!(
        "{} {}",
        "BASE URL :".bright_blue().bold(),
        BASE_URL.bright_white()
    );

    println!(
        "{} {}",
        "USERS    :".bright_blue().bold(),
        TOTAL_USERS.to_string().bright_white()
    );

    println!(
        "{} {}",
        "MAX POLL :".bright_blue().bold(),
        MAX_POLL_ATTEMPTS.to_string().bright_white()
    );

    println!(
        "{}",
        "══════════════════════════════════════════════════════════════"
            .bright_black()
    );

    println!();
}

async fn run_user_flow(
    index: usize,
    user: TestUser,
    prompt: String,
) -> Result<JobResult, String> {
    let started = Instant::now();

    let client = build_client(user.cookie_jar.clone())?;

    info!(
        "{} {} {}",
        "USER".bright_cyan().bold(),
        format!("#{:02}", index).bright_white(),
        user.email.bright_white()
    );

    signup_or_login(&client, &user).await?;

    let job_id = submit_render_job(&client, &prompt).await?;

    info!(
        "{} {} {}",
        "JOB SUBMITTED".green().bold(),
        "→".bright_black(),
        job_id.bright_white()
    );

    let video_path =
        poll_job_status(&client, &job_id, index).await?;

    let duration = started.elapsed().as_millis();

    Ok(JobResult {
        user_index: index,
        email: user.email,
        job_id,
        success: true,
        duration_ms: duration,
        video_path: Some(video_path),
        error: None,
    })
}

fn build_client(cookie_jar: Arc<Jar>) -> Result<Client, String> {
    Client::builder()
        .cookie_provider(cookie_jar)
        .pool_idle_timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(20)
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())
}

async fn signup_or_login(
    client: &Client,
    user: &TestUser,
) -> Result<(), String> {
    let signup_response = client
        .post(format!("{}/api/auth/signup", BASE_URL))
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
        .post(format!("{}/api/auth/login", BASE_URL))
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

        Err("Login failed".into())
    }
}

async fn submit_render_job(
    client: &Client,
    prompt: &str,
) -> Result<String, String> {
    let response = client
        .post(format!("{}/api/jobs", BASE_URL))
        .json(&json!({
            "prompt": prompt
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
            "Job submission failed: {}",
            status
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
    user_index: usize,
) -> Result<String, String> {
    for attempt in 1..=MAX_POLL_ATTEMPTS {
        let response = client
            .get(format!("{}/api/jobs/{}", BASE_URL, job_id))
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

        let colored_status = match status {
            "Completed" => status.green().bold(),
            "Failed" => status.red().bold(),
            "GeneratingCode" => status.yellow().bold(),
            "Pending" => status.bright_blue().bold(),
            _ => status.normal(),
        };

        debug!(
            "{} {} {} {} {}",
            format!("[USER #{:02}]", user_index)
                .bright_cyan(),
            "POLL".bright_black(),
            format!("#{}", attempt).bright_white(),
            "STATUS".bright_black(),
            colored_status
        );

        match status {
            "Completed" => {
                let path = body["data"]["video_path"]
                    .as_str()
                    .unwrap_or("missing");

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
                sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Err("Polling timeout".into())
}



// hiiiiiiiiiii






