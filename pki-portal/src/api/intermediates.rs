use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use crate::{error::ApiError, models::Intermediate, AppState};

#[derive(Deserialize)]
pub struct CreateIntermediateRequest {
    pub parent_id: Uuid,
    pub name: String,
    pub path_len: Option<i32>,
}

#[get("")]
pub async fn list_intermediates(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query_as::<_, Intermediate>(
        "select id, parent_id, name, fingerprint, path_len, status, created_at from intermediates order by created_at desc",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(rows))
}

#[post("")]
pub async fn create_intermediate(
    state: web::Data<AppState>,
    req: web::Json<CreateIntermediateRequest>,
) -> Result<HttpResponse, ApiError> {
    let id = Uuid::new_v4();
    let fingerprint = format!("fp-{}", id);

    sqlx::query(
        "insert into intermediates (id, parent_id, name, fingerprint, path_len, status) values ($1, $2, $3, $4, $5, 'active')",
    )
    .bind(id)
    .bind(req.parent_id)
    .bind(&req.name)
    .bind(&fingerprint)
    .bind(req.path_len)
    .execute(&state.db)
    .await?;

    let created = sqlx::query_as::<_, Intermediate>(
        "select id, parent_id, name, fingerprint, path_len, status, created_at from intermediates where id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(HttpResponse::Created().json(created))
}

#[get("/{id}")]
pub async fn get_intermediate(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let item = sqlx::query_as::<_, Intermediate>(
        "select id, parent_id, name, fingerprint, path_len, status, created_at from intermediates where id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    match item {
        Some(row) => Ok(HttpResponse::Ok().json(row)),
        None => Err(ApiError::NotFound),
    }
}
