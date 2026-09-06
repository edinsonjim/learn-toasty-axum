use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde_json::json;

use crate::AppState;
use crate::error::ApiError;
use crate::models::{CreateFamily, Family, FamilyResponse, UpdateFamily};

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

pub async fn get_family(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<FamilyResponse>, ApiError> {
    let mut guard = state.db.lock().await;
    let db = &mut *guard;

    let family: Vec<Family> = Family::filter(
        Family::fields()
            .id()
            .eq(id)
            .and(Family::fields().deleted_at().is_none()),
    )
    .exec(db)
    .await?;

    match family.into_iter().next() {
        Some(family) => Ok(Json(family.into())),
        None => Err(ApiError::NotFound),
    }
}

pub async fn update_family(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateFamily>,
) -> Result<Json<FamilyResponse>, ApiError> {
    let mut guard = state.db.lock().await;
    let db = &mut *guard;

    let family: Vec<Family> = Family::filter(
        Family::fields()
            .id()
            .eq(id)
            .and(Family::fields().deleted_at().is_none()),
    )
    .exec(db)
    .await?;

    let mut family = match family.into_iter().next() {
        Some(family) => family,
        None => return Err(ApiError::NotFound),
    };

    toasty::update!(family {
        name: payload.name,
        summary: payload.summary,
    })
    .exec(db)
    .await?;

    Ok(Json(family.into()))
}

pub async fn delete_family(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, ApiError> {
    let mut guard = state.db.lock().await;
    let db = &mut *guard;

    let family: Vec<Family> = Family::filter(Family::fields().id().eq(id))
        .exec(db)
        .await?;

    let mut family = match family.into_iter().next() {
        Some(family) => family,
        None => return Err(ApiError::NotFound),
    };

    if family.deleted_at.is_none() {
        toasty::update!(family {
            deleted_at: Some(jiff::Timestamp::now()),
        })
        .exec(db)
        .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}
