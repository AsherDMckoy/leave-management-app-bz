use super::IntoResponse;
pub async fn fallback(uri: axum::http::Uri) -> impl IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("Error 404. No route {}", uri),
    )
}
