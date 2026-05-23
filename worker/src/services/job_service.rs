use chrono::Utc;

use mongodb::bson::{
    doc,
    to_bson,
};

use crate::{
    models::{
        RenderJob,
        RenderResult,
    },
    AppState,
};

// ======================================================
// Get Job
// ======================================================

pub async fn get_render_job_by_id(
    state: &AppState,
    job_id: &str,
) -> mongodb::error::Result<Option<RenderJob>> {

    let collection =
        state.db.collection::<RenderJob>("render_jobs");

    collection
        .find_one(
            doc! {
                "job_id": job_id
            },
            None,
        )
        .await
}

// ======================================================
// Mark GeneratingCode
// ======================================================

pub async fn mark_job_generating(
    state: &AppState,
    job_id: &str,
) -> mongodb::error::Result<()> {

    let collection =
        state.db.collection::<RenderJob>("render_jobs");

    let now = Utc::now();

    collection
        .update_one(
            doc! {
                "job_id": job_id
            },
            doc! {
                "$set": {
                    "status": "GeneratingCode",
                    "started_at": to_bson(&now).unwrap(),
                    "updated_at": to_bson(&now).unwrap(),
                }
            },
            None,
        )
        .await?;

    Ok(())
}

// ======================================================
// Mark Completed
// ======================================================

pub async fn mark_job_completed(
    state: &AppState,
    job_id: &str,
    result: RenderResult,
) -> mongodb::error::Result<()> {

    let collection =
        state.db.collection::<RenderJob>("render_jobs");

    let now = Utc::now();

    collection
        .update_one(
            doc! {
                "job_id": job_id
            },
            doc! {
                "$set": {
                    "status": "Completed",
                    "result": to_bson(&result).unwrap(),
                    "completed_at": to_bson(&now).unwrap(),
                    "updated_at": to_bson(&now).unwrap(),
                }
            },
            None,
        )
        .await?;

    Ok(())
}

// ======================================================
// Mark Failed
// ======================================================

pub async fn mark_job_failed(
    state: &AppState,
    job_id: &str,
    error_message: &str,
) -> mongodb::error::Result<()> {

    let collection =
        state.db.collection::<RenderJob>("render_jobs");

    let now = Utc::now();

    collection
        .update_one(
            doc! {
                "job_id": job_id
            },
            doc! {
                "$set": {
                    "status": "Failed",
                    "error_message": error_message,
                    "completed_at": to_bson(&now).unwrap(),
                    "updated_at": to_bson(&now).unwrap(),
                }
            },
            None,
        )
        .await?;

    Ok(())
}