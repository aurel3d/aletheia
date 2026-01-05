use actix_web::{get, post, web, HttpResponse};
use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, models::Certificate, AppState};

#[derive(Deserialize)]
pub struct CertificateRequest {
    pub issuer_id: Option<Uuid>,
    pub subject_id: String,
    pub subject_name: String,
    pub public_key_b64: String,
    pub is_ca: bool,
}

async fn issue_certificate_impl(
    state: web::Data<AppState>,
    req: web::Json<CertificateRequest>,
) -> Result<HttpResponse, ApiError> {
    let serial = Uuid::new_v4().to_string();
    let public_key = b64
        .decode(&req.public_key_b64)
        .map_err(|e| ApiError::Invalid(format!("invalid public key b64: {e}")))?;

    sqlx::query(
        "insert into certificates (serial, issuer_id, subject_id, subject_name, is_ca, public_key, status) values ($1, $2, $3, $4, $5, $6, 'active')",
    )
    .bind(&serial)
    .bind(req.issuer_id)
    .bind(&req.subject_id)
    .bind(&req.subject_name)
    .bind(req.is_ca)
    .bind(&public_key)
    .execute(&state.db)
    .await?;

    let created = sqlx::query_as::<_, Certificate>(
        "select serial, issuer_id, subject_id, subject_name, is_ca, public_key, status, created_at from certificates where serial = $1",
    )
    .bind(&serial)
    .fetch_one(&state.db)
    .await?;

    Ok(HttpResponse::Created().json(created))
}

async fn get_certificate_impl(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let serial = path.into_inner();
    let cert = sqlx::query_as::<_, Certificate>(
        "select serial, issuer_id, subject_id, subject_name, is_ca, public_key, status, created_at from certificates where serial = $1",
    )
    .bind(&serial)
    .fetch_optional(&state.db)
    .await?;

    match cert {
        Some(c) => Ok(HttpResponse::Ok().json(c)),
        None => Err(ApiError::NotFound),
    }
}

#[post("")]
pub async fn issue_certificate_handler(
    state: web::Data<AppState>,
    req: web::Json<CertificateRequest>,
) -> Result<HttpResponse, ApiError> {
    issue_certificate_impl(state, req).await
}

#[get("/{serial}")]
pub async fn get_certificate_handler(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    get_certificate_impl(state, path).await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, http::StatusCode, web};
    use base64::Engine;
    use sqlx::PgPool;
    use crate::{error::ApiError, models::Certificate, AppState};
    use super::{get_certificate_impl, issue_certificate_impl, CertificateRequest};

    #[sqlx::test]
    async fn issue_and_get_certificate_round_trip(pool: PgPool) {
        let state = web::Data::new(AppState { db: pool.clone() });
        let req = CertificateRequest {
            issuer_id: None,
            subject_id: "subj-1".into(),
            subject_name: "Test Subject".into(),
            public_key_b64: base64::engine::general_purpose::STANDARD.encode(b"foo-key"),
            is_ca: false,
        };

        let resp = issue_certificate_impl(state.clone(), web::Json(req)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = to_bytes(resp.into_body()).await.unwrap();
        let created: Certificate = serde_json::from_slice(&body).unwrap();

        let resp = get_certificate_impl(state, web::Path::from(created.serial.clone()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let fetched: Certificate = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(fetched.serial, created.serial);
        assert_eq!(fetched.subject_id, "subj-1");
        assert_eq!(fetched.subject_name, "Test Subject");
        assert_eq!(fetched.is_ca, false);
    }

    #[sqlx::test]
    async fn issue_certificate_invalid_b64_rejected(_pool: PgPool) {
        let state = web::Data::new(AppState { db: _pool });
        let bad_req = CertificateRequest {
            issuer_id: None,
            subject_id: "subj-bad".into(),
            subject_name: "Bad".into(),
            public_key_b64: "@@notb64".into(),
            is_ca: false,
        };

        let result = issue_certificate_impl(state, web::Json(bad_req)).await;
        match result {
            Err(ApiError::Invalid(_)) => {}
            other => panic!("expected invalid error, got {other:?}"),
        }
    }
}
