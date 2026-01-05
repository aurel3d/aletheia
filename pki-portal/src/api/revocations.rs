use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

use crate::{error::ApiError, models::Revocation, AppState};

#[derive(Deserialize)]
pub struct RevocationRequest {
    pub serial: String,
    pub reason: Option<String>,
}

async fn get_revocations_impl(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query_as::<_, Revocation>(
        "select serial, reason, revoked_at from revocations order by revoked_at desc",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(rows))
}

async fn revoke_certificate_impl(
    state: web::Data<AppState>,
    req: web::Json<RevocationRequest>,
) -> Result<HttpResponse, ApiError> {
    sqlx::query(
        "insert into revocations (serial, reason) values ($1, $2) on conflict (serial) do update set reason = excluded.reason, revoked_at = now()",
    )
    .bind(&req.serial)
    .bind(&req.reason)
    .execute(&state.db)
    .await?;

    let entry = sqlx::query_as::<_, Revocation>(
        "select serial, reason, revoked_at from revocations where serial = $1",
    )
    .bind(&req.serial)
    .fetch_one(&state.db)
    .await?;

    Ok(HttpResponse::Created().json(entry))
}

#[get("")]
pub async fn get_revocations_handler(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    get_revocations_impl(state).await
}

#[post("")]
pub async fn revoke_certificate_handler(
    state: web::Data<AppState>,
    req: web::Json<RevocationRequest>,
) -> Result<HttpResponse, ApiError> {
    revoke_certificate_impl(state, req).await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, http::StatusCode, web};
    use sqlx::PgPool;
    use crate::{models::Revocation, AppState};
    use super::{get_revocations_impl, revoke_certificate_impl, RevocationRequest};

    #[sqlx::test]
    async fn revoke_and_list(pool: PgPool) {
        // First insert a certificate (required by foreign key)
        sqlx::query(
            "insert into certificates (serial, subject_id, subject_name, is_ca, public_key, status) values ($1, $2, $3, $4, $5, 'active')",
        )
        .bind("serial-1")
        .bind("subj-1")
        .bind("Test Subject")
        .bind(false)
        .bind(b"test-key")
        .execute(&pool)
        .await
        .unwrap();
        
        let state = web::Data::new(AppState { db: pool });
        
        let req = RevocationRequest {
            serial: "serial-1".into(),
            reason: Some("compromise".into()),
        };

        let resp = revoke_certificate_impl(state.clone(), web::Json(req)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let created: Revocation = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(created.serial, "serial-1");
        assert_eq!(created.reason.as_deref(), Some("compromise"));

        let resp = get_revocations_impl(state).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let list: Vec<Revocation> = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].serial, "serial-1");
    }
}
