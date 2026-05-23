use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    controller::{
        create_render_job_handler,
        get_job_handler,
    },
    middleware::auth_middleware,
    AppState,
};

pub fn job_routes() -> Router<AppState> {
    Router::new()

        // POST /api/jobs
        .route(
            "/",
            post(create_render_job_handler),
        )

        // GET /api/jobs/:job_id
        .route(
            "/{job_id}",
            get(get_job_handler),
        )

        // Auth middleware
        .route_layer(
            middleware::from_fn(auth_middleware),
        )
}