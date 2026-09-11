//! HTTP transport, OpenAPI document generation, and static frontend serving.

use std::{path::PathBuf, sync::Arc};

use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use sql_processor_core::process_sql_text;
use sql_processor_validation::{
    Config as ValidationConfig, Input as ValidationInput, Result as ValidationResult, Runner,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(Debug, Clone)]
pub struct ApiState {
    pub openapi: Arc<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessRequest {
    pub sql: String,
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub validation_config: Option<ValidationConfig>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProcessResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_sql: Option<String>,
    pub messages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        (
            status,
            Json(ProcessResponse {
                processed_sql: None,
                messages: Vec::new(),
                validation: None,
                error: Some(message),
            }),
        )
            .into_response()
    }
}

pub fn build_app(static_dir: PathBuf) -> Router {
    let (router, document) = build_api_router().split_for_parts();
    let state = ApiState {
        openapi: Arc::new(serde_json::to_value(document).expect("OpenAPI serializes")),
    };
    let index = static_dir.join("index.html");
    let static_service = ServeDir::new(static_dir).fallback(ServeFile::new(index));
    router
        .route("/openapi.json", get(openapi))
        .with_state(state)
        .layer(RequestBodyLimitLayer::new(4 * 1024 * 1024))
        .fallback_service(static_service)
}

pub fn openapi_json() -> Result<String, serde_json::Error> {
    ApiDoc::openapi().to_pretty_json()
}

pub fn build_api_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(health))
        .routes(routes!(process))
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses((status = 200, description = "Service health", body = HealthResponse))
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    post,
    path = "/api/process",
    request_body = ProcessRequest,
    responses(
        (status = 200, description = "Processed SQL", body = ProcessResponse),
        (status = 400, description = "Invalid request", body = ProcessResponse),
        (status = 500, description = "Processing failure", body = ProcessResponse)
    )
)]
async fn process(
    State(_state): State<ApiState>,
    payload: Result<Json<ProcessRequest>, JsonRejection>,
) -> Result<Json<ProcessResponse>, ApiError> {
    let Json(request) =
        payload.map_err(|_| ApiError::BadRequest("invalid JSON payload".to_string()))?;
    if request.sql.trim().is_empty() {
        return Err(ApiError::BadRequest("sql cannot be empty".to_string()));
    }
    let original_sql = request.sql.clone();
    let (processed_sql, messages) = process_sql_text(request.sql, &request.schema)
        .map_err(|error| ApiError::Internal(format!("error processing SQL text: {error}")))?;
    let runner: Box<dyn Runner> = match request.validation_config.as_ref() {
        Some(config) if config.enabled => {
            sql_processor_validation::new_oracle_runner_from_config(config)
        }
        _ => Box::new(sql_processor_validation::new_oracle_runner_from_env()),
    };
    let validation = runner
        .validate(ValidationInput {
            original_sql,
            processed_sql: processed_sql.clone(),
            target_schema: request.schema.trim().to_string(),
        })
        .await;
    Ok(Json(ProcessResponse {
        processed_sql: Some(processed_sql),
        messages,
        validation: Some(validation),
        error: None,
    }))
}

async fn openapi(State(state): State<ApiState>) -> Json<serde_json::Value> {
    Json((*state.openapi).clone())
}

#[derive(OpenApi)]
#[openapi(
    info(title = "SQL Processor API", description = "Oracle SQL transformation and parse-only validation API", version = env!("CARGO_PKG_VERSION")),
    paths(health, process),
    components(schemas(ProcessRequest, ProcessResponse, HealthResponse, ValidationConfig, sql_processor_validation::Result, sql_processor_validation::Target, sql_processor_validation::Issue))
)]
struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn exposes_health_and_openapi() {
        let app = build_app(PathBuf::from("missing-static-dir"));
        let response = app
            .clone()
            .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let response = app
            .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn openapi_contains_process_contract() {
        let document = serde_json::from_str::<serde_json::Value>(&openapi_json().unwrap()).unwrap();
        assert!(document["paths"]["/api/process"]["post"].is_object());
        assert!(document["components"]["schemas"]["ProcessRequest"].is_object());
    }

    #[tokio::test]
    async fn process_preserves_json_contract() {
        let app = build_app(PathBuf::from("missing-static-dir"));
        let request = Request::post("/api/process")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"sql":"SELECT 1 FROM DUAL","schema":"EBANK"}"#,
            ))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let document = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
        assert!(document["processedSql"].as_str().is_some());
        assert!(document["messages"].is_array());
        assert!(document["validation"]["targets"].is_array());
        assert!(document.get("error").is_none());
    }
}
