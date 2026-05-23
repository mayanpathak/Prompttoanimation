
use std::sync::Arc;
use std::time::Duration;

use deadpool_redis::redis::AsyncCommands;
use tokio::sync::Semaphore;

use crate::{
    processor::process_render_job,
    AppState,
};

const RENDER_QUEUE: &str = "render_queue";

const MAX_CONCURRENT_JOBS: usize = 5;

pub async fn start_worker(
    state: Arc<AppState>,
) {

    println!("Worker started.");
    println!("Waiting for jobs...");

    // =====================================
    // Concurrency limiter
    // =====================================

    let semaphore = Arc::new(
        Semaphore::new(MAX_CONCURRENT_JOBS)
    );

    loop {

        // =====================================
        // Get Redis connection
        // =====================================

        let mut redis_conn = match state
            .redis_pool
            .get()
            .await
        {
            Ok(conn) => conn,

            Err(err) => {

                eprintln!(
                    "Failed to get Redis connection: {}",
                    err
                );

                tokio::time::sleep(
                    Duration::from_secs(2)
                )
                .await;

                continue;
            }
        };

        // =====================================
        // Fetch next job
        // =====================================

        let result: Result<
            Option<(String, String)>,
            _
        > = redis_conn
            .brpop(RENDER_QUEUE, 0.0)
            .await;

        match result {

            Ok(Some((_queue, job_id))) => {

                println!(
                    "Received job: {}",
                    job_id
                );

                // =====================================
                // Wait for concurrency slot
                // =====================================

                let available_before =
                    semaphore.available_permits();

                println!(
                    "[QUEUE] Waiting for slot | available permits: {}",
                    available_before
                );

                // =====================================
                // Acquire concurrency permit
                // =====================================

                let permit = semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .unwrap();

                let available_after =
                    semaphore.available_permits();

                println!(
                    "[WORKER] Slot acquired | active jobs: {} | available permits: {}",
                    MAX_CONCURRENT_JOBS - available_after,
                    available_after
                );

                // =====================================
                // Clone shared state
                // =====================================

                let state_clone = state.clone();

                // IMPORTANT:
                // Clone semaphore before moving into task
                let semaphore_clone =
                    semaphore.clone();

                // =====================================
                // Spawn concurrent task
                // =====================================

                tokio::spawn(async move {

                    println!(
                        "START job {} at {:?}",
                        job_id,
                        std::time::SystemTime::now()
                    );

                    let result =
                        process_render_job(
                            &state_clone,
                            &job_id,
                        )
                        .await;

                    match result {

                        Ok(_) => {

                            println!(
                                "Successfully processed job: {}",
                                job_id
                            );
                        }

                        Err(err) => {

                            eprintln!(
                                "Failed processing job {}: {:?}",
                                job_id,
                                err
                            );
                        }
                    }

                    println!(
                        "END job {} at {:?}",
                        job_id,
                        std::time::SystemTime::now()
                    );

                    // =====================================
                    // Release semaphore slot
                    // =====================================

                    drop(permit);

                    let available_after_release =
                        semaphore_clone
                            .available_permits();

                    println!(
                        "[WORKER] Job finished | active jobs: {} | available permits: {}",
                        MAX_CONCURRENT_JOBS
                            - available_after_release,
                        available_after_release
                    );
                });
            }

            Ok(None) => {

                eprintln!(
                    "BRPOP returned no data."
                );
            }

            Err(err) => {

                eprintln!(
                    "Redis BRPOP error: {}",
                    err
                );

                tokio::time::sleep(
                    Duration::from_secs(1)
                )
                .await;
            }
        }
    }
}