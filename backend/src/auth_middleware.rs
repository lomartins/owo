use crate::error::ApiError;
use crate::services::session;
use crate::state::{AppState, CurrentUser};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthenticated)?;

    let user = session::verify(&state.pool, token)
        .await?
        .ok_or(ApiError::Unauthenticated)?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

pub fn current_user(req: &Request) -> Result<&CurrentUser, ApiError> {
    req.extensions()
        .get::<CurrentUser>()
        .ok_or(ApiError::Unauthenticated)
}
