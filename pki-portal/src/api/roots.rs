use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, models::Root, AppState};

#[derive(Deserialize)]
pub struct CreateRootRequest {
    pub name: String,
}

#[get("")]
pub async fn list_roots(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query_as::<_, Root>(
        "select id, name, fingerprint, status, created_at from roots order by created_at desc",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(rows))
}

#[post("")]
pub async fn create_root(
    state: web::Data<AppState>,
    req: web::Json<CreateRootRequest>,
) -> Result<HttpResponse, ApiError> {
    // Placeholder: in real implementation, key material lives in KMS/HSM; fingerprint comes from stored public key.
    let id = Uuid::new_v4();
    let fingerprint = format!("fp-{}", id);

    sqlx::query("insert into roots (id, name, fingerprint, status) values ($1, $2, $3, 'active')")
    .bind(id)
    .bind(&req.name)
    .bind(&fingerprint)
    .execute(&state.db)
    .await?;

    let created = sqlx::query_as::<_, Root>(
        "select id, name, fingerprint, status, created_at from roots where id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(HttpResponse::Created().json(created))
}

#[get("/{id}")]
pub async fn get_root(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let root = sqlx::query_as::<_, Root>(
        "select id, name, fingerprint, status, created_at from roots where id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    match root {
        Some(r) => Ok(HttpResponse::Ok().json(r)),
        None => Err(ApiError::NotFound),
    }
}

#[post("/{id}/rotate")]
pub async fn rotate_root(
    _state: web::Data<AppState>,
    _path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    Err(ApiError::NotImplemented)
}
