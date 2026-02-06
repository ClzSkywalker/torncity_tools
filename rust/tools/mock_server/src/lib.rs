use axum::{
    extract::State,
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct MockResponse {
    pub status: StatusCode,
    pub body: String,
    pub content_type: String,
}

impl Default for MockResponse {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            body: "Mock response".to_string(),
            content_type: "text/plain; charset=utf-8".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct MockServerState {
    pub response: Arc<MockResponse>,
}

pub fn create_router(state: MockServerState) -> Router {
    Router::new()
        .route("/favorites", post(handle_favorites))
        .with_state(state)
}

async fn handle_favorites(
    State(state): State<MockServerState>,
    _method: Method,
    _headers: HeaderMap,
    _body: String,
) -> Response {
    println!("[Mock Server] Received request:");
    let response = state.response.as_ref();
    (
        response.status,
        [(header::CONTENT_TYPE, response.content_type.clone())],
        response.body.clone(),
    )
        .into_response()
}
