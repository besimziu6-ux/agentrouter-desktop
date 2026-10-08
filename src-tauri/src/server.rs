use std::path::PathBuf;

use axum::{
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use serde_json::json;
use tower_http::services::ServeDir;

pub fn build_router(dist_dir: PathBuf) -> Router {
    let index_file = dist_dir.join("index.html");
    let static_dir = dist_dir.join("static");

    Router::new()
        .route("/api/health", get(health))
        .route(
            "/",
            get(move || async move {
                match tokio::fs::read(&index_file).await {
                    Ok(bytes) => (
                        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                        bytes,
                    )
                        .into_response(),
                    Err(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "missing dist/index.html",
                    )
                        .into_response(),
                }
            }),
        )
        .route("/api/*path", any(api_not_found))
        .nest_service("/static", ServeDir::new(static_dir))
        .fallback(fallback)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}))
}

async fn api_not_found(uri: Uri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": "not found", "path": uri.path().to_string()})),
    )
}

async fn fallback(uri: Uri) -> impl IntoResponse {
    let path = uri.path().to_string();
    if path.starts_with("/api/") || path == "/api" {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "not found", "path": path})),
        )
            .into_response();
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    async fn spawn_test_server() -> SocketAddr {
        let tmp = std::env::temp_dir().join(format!("ar-desktop-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(tmp.join("static")).unwrap();
        std::fs::write(tmp.join("index.html"), "<html>test</html>").unwrap();
        let router = build_router(tmp);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        addr
    }

    #[tokio::test]
    async fn health_returns_status_ok() {
        let addr = spawn_test_server().await;
        let url = format!("http://{addr}/api/health");
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("ok"));
        assert!(body.get("version").is_some());
    }

    #[tokio::test]
    async fn root_serves_index_html() {
        let addr = spawn_test_server().await;
        let url = format!("http://{addr}/");
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), 200);
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(ct.contains("text/html"), "unexpected content-type: {ct}");
        let text = resp.text().await.unwrap();
        assert!(text.contains("test"));
    }

    #[tokio::test]
    async fn unknown_api_returns_json_404() {
        let addr = spawn_test_server().await;
        let url = format!("http://{addr}/api/nope");
        let resp = reqwest::get(&url).await.unwrap();
        assert_eq!(resp.status(), 404);
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("not found")
        );
    }
}
