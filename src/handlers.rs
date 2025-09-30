use crate::{
    models::{LatestResponse, StoredRequest},
    storage::Storage,
    utils::internal_err,
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    Json,
};
use std::{
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true }))
}

pub async fn get_latest(
    State(storage): State<Storage>,
) -> Result<Json<LatestResponse>, (StatusCode, String)> {
    let items = storage.latest(50).map_err(internal_err)?;
    Ok(Json(LatestResponse {
        count: items.len(),
        items,
    }))
}

pub async fn get_one(
    State(storage): State<Storage>,
    Path(id): Path<u64>,
) -> Result<Json<StoredRequest>, (StatusCode, String)> {
    let items = storage.latest(1_000).map_err(internal_err)?;
    items
        .into_iter()
        .find(|r| r.id == id)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "not found".into()))
}

pub async fn post_webhook(
    State(storage): State<Storage>,
    method: Method,
    uri: axum::http::Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let id = storage.next_id().fetch_add(1, Ordering::SeqCst) + 1;

    // std timestamp (no chrono dep)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(internal_err)?
        .as_millis() as i64;

    let headers_vec = headers
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect::<Vec<_>>();

    let record = StoredRequest {
        id,
        ts_ms: now,
        method: method.to_string(),
        path: uri.to_string(),
        headers: headers_vec,
        body: body.to_vec(),
    };

    storage.insert(&record).map_err(internal_err)?;
    Ok((StatusCode::OK, "ok"))
}
