use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{header, HeaderValue, Method};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use utoipa::IntoParams;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::errors::DneyesError;
use crate::telemetry::clickhouse::{ClickhouseContext, DnsQueryFilter};
use crate::telemetry::models::DnsResolutionEvent;

/// Run the REST API server exposing DNS resolution history.
pub async fn run(config: &AppConfig, clickhouse: ClickhouseContext) -> Result<(), DneyesError> {
    let state = ApiState {
        clickhouse: Arc::new(clickhouse),
        page_size: config.api.page_size,
    };

    let openapi = ApiDoc::openapi();
    let openapi_route = openapi.clone();

    let cors = build_cors_layer(&config.api.cors.allow_origins);

    let app = Router::new()
        .route("/api/v1/dns/resolutions", get(list_dns_resolutions))
        .route("/healthz", get(healthz))
        .route(
            "/openapi.json",
            get(move || async { Json(openapi_route.clone()) }),
        )
        .merge(SwaggerUi::new("/docs").url("/openapi.json", openapi))
        .with_state(state)
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.api.host, config.api.port)
        .parse()
        .map_err(|e| DneyesError::Config(format!("Invalid API bind address: {}", e)))?;

    tracing::info!("Starting API server", %addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| DneyesError::Http(e.to_string()))
}

fn build_cors_layer(origins: &[String]) -> CorsLayer {
    if origins.is_empty() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET])
            .allow_headers([header::CONTENT_TYPE])
    } else {
        let allowed: Vec<HeaderValue> = origins
            .iter()
            .filter_map(|origin| HeaderValue::from_str(origin).ok())
            .collect();
        CorsLayer::new()
            .allow_origin(allowed)
            .allow_methods([Method::GET])
            .allow_headers([header::CONTENT_TYPE])
    }
}

#[derive(Clone)]
struct ApiState {
    clickhouse: Arc<ClickhouseContext>,
    page_size: usize,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
struct DnsQueryParams {
    /// Fully qualified domain name to filter for.
    #[serde(default)]
    domain: Option<String>,
    /// Resolver country code to filter for.
    #[serde(default)]
    country: Option<String>,
    /// Filter by success state (use `success` or `failure`).
    #[serde(default)]
    status: Option<String>,
    /// Start of the time range in RFC3339 format.
    #[serde(default)]
    from: Option<String>,
    /// End of the time range in RFC3339 format.
    #[serde(default)]
    to: Option<String>,
    /// Maximum number of entries to return.
    #[serde(default)]
    limit: Option<usize>,
}

/// API error wrapper converting internal errors into HTTP responses.
#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    Internal(DneyesError),
}

impl From<DneyesError> for ApiError {
    fn from(value: DneyesError) -> Self {
        ApiError::Internal(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(message) => {
                (axum::http::StatusCode::BAD_REQUEST, message).into_response()
            }
            ApiError::Internal(err) => {
                tracing::error!("Internal API error: {err}");
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
                    .into_response()
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/dns/resolutions",
    params(DnsQueryParams),
    responses((status = 200, description = "DNS resolution history", body = [DnsResolutionEvent]))
)]
async fn list_dns_resolutions(
    State(state): State<ApiState>,
    Query(params): Query<DnsQueryParams>,
) -> Result<Json<Vec<DnsResolutionEvent>>, ApiError> {
    let limit = params.limit.unwrap_or(state.page_size);
    let status = match params.status.as_deref() {
        Some("success") => Some(true),
        Some("failure") => Some(false),
        Some(other) => {
            return Err(ApiError::BadRequest(format!(
                "Invalid status filter: {other}. Use 'success' or 'failure'."
            )))
        }
        None => None,
    };

    let from = parse_timestamp(params.from)?;
    let to = parse_timestamp(params.to)?;

    let filter = DnsQueryFilter {
        domain: params.domain.clone(),
        country: params.country.clone(),
        success: status,
        from,
        to,
    };

    let events = state
        .clickhouse
        .query_dns(&filter, limit)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(events))
}

fn parse_timestamp(ts: Option<String>) -> Result<Option<DateTime<Utc>>, ApiError> {
    ts.map(|value| {
        DateTime::parse_from_rfc3339(&value)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| {
                ApiError::BadRequest(format!(
                    "Invalid timestamp: {value}. Use RFC3339 format (e.g. 2024-05-23T10:00:00Z)."
                ))
            })
    })
    .transpose()
}

#[utoipa::path(get, path = "/healthz", responses((status = 200, body = String)))]
async fn healthz(State(state): State<ApiState>) -> Result<String, ApiError> {
    state.clickhouse.ping().await.map_err(ApiError::from)?;
    Ok("ok".to_string())
}

#[derive(OpenApi)]
#[openapi(
    paths(list_dns_resolutions, healthz),
    components(schemas(DnsResolutionEvent, DnsQueryParams)),
    tags((name = "dns", description = "DNS resolution history"))
)]
struct ApiDoc;
