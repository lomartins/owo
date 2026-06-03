use crate::api;
use crate::state::AppState;
use axum::Router;
use std::path::{Path, PathBuf};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn router(state: AppState) -> Router {
    let mut app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api::ApiDoc::openapi()))
        .merge(api::router(state));

    // Uploaded user assets (profile photos). Created lazily by the upload handler.
    let uploads_dir =
        PathBuf::from(std::env::var("OWO_UPLOADS_DIR").unwrap_or_else(|_| "uploads".to_string()));
    if !uploads_dir.exists() {
        let _ = std::fs::create_dir_all(&uploads_dir);
    }
    app = app.nest_service("/uploads", ServeDir::new(&uploads_dir));
    tracing::info!("serving uploads from {}", uploads_dir.display());

    if let Some(dist) = web_dist_path() {
        let index = dist.join("index.html");
        let serve = ServeDir::new(&dist).not_found_service(ServeFile::new(&index));
        app = app.fallback_service(serve);
        tracing::info!("serving SPA from {}", dist.display());
    } else {
        tracing::warn!("no web SPA found (set OWO_WEB_DIST or build ../web; backend will run API-only)");
    }

    app.layer(TraceLayer::new_for_http()).layer(CorsLayer::permissive())
}

fn web_dist_path() -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = match std::env::var("OWO_WEB_DIST") {
        Ok(p) => vec![PathBuf::from(p)],
        Err(_) => vec![
            PathBuf::from("web/dist"),
            PathBuf::from("../web/dist"),
        ],
    };
    candidates
        .into_iter()
        .find(|p| Path::new(&p).join("index.html").exists())
}
