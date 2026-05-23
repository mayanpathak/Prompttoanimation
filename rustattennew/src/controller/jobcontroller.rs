use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
    Json,
};

use mongodb::bson::oid::ObjectId;

use serde::Serialize;

use uuid::Uuid;

use validator::Validate;

use crate::{
    schema::CreateRenderJobForm,
    models::JobStatus,
    services::{
        create_render_job_service,
        get_render_job_service,
        AppError,
    },
    AppState,
};

// ======================================================
// Response Types
// ======================================================

#[derive(Debug, Serialize)]
pub struct CreateJobResponse {
    pub success: bool,
    pub message: String,
    pub data: CreateJobData,
}

#[derive(Debug, Serialize)]
pub struct CreateJobData {
    pub job_id: String,
    pub status: JobStatus,
}

#[derive(Debug, Serialize)]
pub struct GetJobResponse {
    pub success: bool,
    pub data: GetJobData,
}

#[derive(Debug, Serialize)]
pub struct GetJobData {
    pub job_id: String,
    pub status: JobStatus,
    pub video_path: Option<String>,
}

// ======================================================
// AppError -> StatusCode
// ======================================================

fn map_app_error(err: AppError) -> StatusCode {
    match err {
        AppError::Validation(_) => StatusCode::BAD_REQUEST,

        AppError::NotFound(_) => StatusCode::NOT_FOUND,

        AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,

        AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,

        AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ======================================================
// POST /api/jobs
// ======================================================

pub async fn create_render_job_handler(
    State(state): State<AppState>,

    Extension(user_id): Extension<ObjectId>,

    Json(payload): Json<CreateRenderJobForm>,
) -> Result<Json<CreateJobResponse>, StatusCode> {

    // =========================================
    // Validate request body
    // =========================================

    payload
        .validate()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // =========================================
    // Call service layer
    // =========================================

    let response = create_render_job_service(
        &state,
        user_id,
        payload,
    )
    .await
    .map_err(map_app_error)?;

    // =========================================
    // Response
    // =========================================

    Ok(Json(CreateJobResponse {
        success: true,

        message: "Render job created successfully".to_string(),

        data: CreateJobData {
            job_id: response.job_id,
            status: response.status,
        },
    }))
}

// ======================================================
// GET /api/jobs/:job_id
// ======================================================

pub async fn get_job_handler(
    State(state): State<AppState>,

    Extension(user_id): Extension<ObjectId>,

    Path(job_id): Path<Uuid>,
) -> Result<Json<GetJobResponse>, StatusCode> {

    // =========================================
    // Call service layer
    // =========================================

    let response = get_render_job_service(
        &state,
        user_id,
        job_id.to_string(),
    )
    .await
    .map_err(map_app_error)?;

    // =========================================
    // Extract video URL
    // =========================================

    let video_path = response
    .job
    .result
    .as_ref()
    .map(|result| result.video_path.clone());
    // =========================================
    // Response
    // =========================================

    Ok(Json(GetJobResponse {
        success: true,

        data: GetJobData {
            job_id: response.job.job_id,
            status: response.job.status,
            video_path,
        },
    }))
}