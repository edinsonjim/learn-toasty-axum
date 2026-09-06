use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;

use crate::AppState;
use crate::error::ApiError;
use crate::models::{CreateFamily, Family, FamilyResponse};

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn create_family(
    State(state): State<AppState>,
    Json(payload): Json<CreateFamily>,
) -> Result<(StatusCode, Json<FamilyResponse>), ApiError> {
    let mut guard = state.db.lock().await;
    let db = &mut *guard;

    let family = toasty::create!(Family {
        name: payload.name,
        summary: payload.summary,
    })
    .exec(db)
    .await?;

    Ok((StatusCode::CREATED, Json(family.into())))
}

pub async fn list_families(
    State(state): State<AppState>,
) -> Result<Json<Vec<FamilyResponse>>, ApiError> {
    let mut guard = state.db.lock().await;
    let db = &mut *guard;

    let families: Vec<Family> = Family::filter(Family::fields().deleted_at().is_none())
        .exec(db)
        .await?;

    Ok(Json(families.into_iter().map(Into::into).collect()))
}
