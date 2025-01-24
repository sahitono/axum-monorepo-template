use crate::infrastructure::config::Config;
use crate::infrastructure::state::AppState;
use crate::route::auth::AuthRoute;
use crate::route::user::UserRoute;
use axum::error_handling::HandleErrorLayer;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::{BoxError, Json, Router};
use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use axum::extract::DefaultBodyLimit;
use tower::ServiceBuilder;
use tower::{buffer::BufferLayer, limit::RateLimitLayer};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod auth;
mod project;
mod project_image;
pub mod user;

lazy_static! {
    static ref HTTP_TIMEOUT: u64 = 30;
}

pub struct AppRoute;
impl AppRoute {
    pub fn init(db: Arc<DatabaseConnection>, config: Arc<Config>) -> Router {
        let state = AppState::init( db, config );

        let routes = Router::new()
            .nest("/auth", AuthRoute::init(&state))
            .nest("/users", UserRoute::init(&state))

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::DELETE,
                Method::PUT,
                Method::PATCH,
            ])
            .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

        Router::new()
            .nest("/api", routes)
            .layer(cors)
            .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(HandleErrorLayer::new(Self::handle_timeout_error))
                    .layer(BufferLayer::new(1024))
                    .layer(RateLimitLayer::new(5, Duration::from_secs(1))),
            )
            .with_state(state)
            .fallback(Self::handle_404)
    }

    #[allow(clippy::unused_async)]
    async fn handle_404() -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
            "errors":{
            "message": vec!(String::from("The requested resource does not exist on this server!")),}
            })),
        )
    }

    #[allow(clippy::unused_async)]
    async fn handle_timeout_error(err: BoxError) -> (StatusCode, Json<serde_json::Value>) {
        if err.is::<tower::timeout::error::Elapsed>() {
            (
                StatusCode::REQUEST_TIMEOUT,
                Json(json!({
                    "error":
                        format!(
                            "request took longer than the configured {} second timeout",
                            *HTTP_TIMEOUT
                        )
                })),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("unhandled internal error: {}", err)
                })),
            )
        }
    }
}
