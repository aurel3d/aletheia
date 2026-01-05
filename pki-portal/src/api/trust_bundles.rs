use actix_web::{get, post, web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{error::ApiError, models::TrustBundleMeta, AppState};

async fn get_latest_bundle_impl(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let item = sqlx::query_as::<_, TrustBundleMeta>(
        "select version, issued_at, url, signer_fingerprint, status, payload, signature from trust_bundles order by issued_at desc limit 1",
    )
    .fetch_optional(&state.db)
    .await?;

    match item {
        Some(bundle) => Ok(HttpResponse::Ok().json(bundle)),
        None => Err(ApiError::NotFound),
    }
}

async fn get_bundle_by_version_impl(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let version = path.into_inner();
    let item = sqlx::query_as::<_, TrustBundleMeta>(
        "select version, issued_at, url, signer_fingerprint, status, payload, signature from trust_bundles where version = $1",
    )
    .bind(&version)
    .fetch_optional(&state.db)
    .await?;

    match item {
        Some(bundle) => Ok(HttpResponse::Ok().json(bundle)),
        None => Err(ApiError::NotFound),
    }
}

#[derive(Deserialize)]
pub struct PublishBundleRequest {
    pub url: String,
    pub signer_fingerprint: String,
}

async fn publish_bundle_impl(
    state: web::Data<AppState>,
    _req: web::Json<PublishBundleRequest>,
) -> Result<HttpResponse, ApiError> {
    // Assemble payload from current roots and intermediates.
    let roots: Vec<(Uuid, String, String)> = sqlx::query_as(
        "select id, name, fingerprint from roots where status = 'active'",
    )
    .fetch_all(&state.db)
    .await?;

    let intermediates: Vec<(Uuid, String, String)> = sqlx::query_as(
        "select id, name, fingerprint from intermediates where status = 'active'",
    )
    .fetch_all(&state.db)
    .await?;

    let issued_at = Utc::now();
    let version = issued_at.timestamp_millis().to_string();

    let payload = serde_json::json!({
        "version": version,
        "issued_at": issued_at.timestamp(),
        "roots": roots.iter().map(|(id, name, fp)| serde_json::json!({
            "id": id,
            "name": name,
            "fingerprint": fp,
        })).collect::<Vec<_>>(),
        "intermediates": intermediates.iter().map(|(id, name, fp)| serde_json::json!({
            "id": id,
            "name": name,
            "fingerprint": fp,
        })).collect::<Vec<_>>(),
    });

    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|e| ApiError::Invalid(format!("serialize payload: {e}")))?;
    let mut hasher = Sha256::new();
    hasher.update(&payload_bytes);
    let signature = format!("{:x}", hasher.finalize());

    sqlx::query(
        "insert into trust_bundles (version, issued_at, url, signer_fingerprint, status, payload, signature) values ($1, $2, $3, $4, 'active', $5, $6)",
    )
    .bind(&version)
    .bind(issued_at)
    .bind(&_req.url)
    .bind(&_req.signer_fingerprint)
    .bind(&payload)
    .bind(&signature)
    .execute(&state.db)
    .await?;

    let created = sqlx::query_as::<_, TrustBundleMeta>(
        "select version, issued_at, url, signer_fingerprint, status, payload, signature from trust_bundles where version = $1",
    )
    .bind(&version)
    .fetch_one(&state.db)
    .await?;

    Ok(HttpResponse::Created().json(created))
}

#[get("/latest")]
pub async fn get_latest_bundle_handler(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    get_latest_bundle_impl(state).await
}

#[get("/{version}")]
pub async fn get_bundle_by_version_handler(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    get_bundle_by_version_impl(state, path).await
}

#[post("")]
pub async fn publish_bundle_handler(
    state: web::Data<AppState>,
    _req: web::Json<PublishBundleRequest>,
) -> Result<HttpResponse, ApiError> {
    publish_bundle_impl(state, _req).await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, http::StatusCode, web};
    use sqlx::PgPool;
    use uuid::Uuid;
    use crate::{models::TrustBundleMeta, AppState};
    use super::{get_bundle_by_version_impl, get_latest_bundle_impl, publish_bundle_impl, PublishBundleRequest};

    #[sqlx::test]
    async fn latest_and_specific_bundle(pool: PgPool) {
        sqlx::query(
            "insert into trust_bundles (version, url, signer_fingerprint, status, payload, signature) values ($1, $2, $3, 'active', '{}'::jsonb, 'sig')",
        )
        .bind("v1")
        .bind("https://example.com/bundles/v1.json")
        .bind("fp1")
        .execute(&pool)
        .await
        .unwrap();

        let state = web::Data::new(AppState { db: pool.clone() });

        let resp = get_latest_bundle_impl(state.clone()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let latest: TrustBundleMeta = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(latest.version, "v1");

        let resp = get_bundle_by_version_impl(state, web::Path::from("v1".to_string()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let fetched: TrustBundleMeta = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(fetched.version, "v1");
        assert_eq!(fetched.url, "https://example.com/bundles/v1.json");
    }

    #[sqlx::test]
    async fn publish_creates_bundle(pool: PgPool) {
        // seed data
        sqlx::query("insert into roots (id, name, fingerprint, status) values ($1, $2, $3, 'active')")
            .bind(Uuid::new_v4())
            .bind("root1")
            .bind("fp-root1")
            .execute(&pool)
            .await
            .unwrap();

        let state = web::Data::new(AppState { db: pool });
        let req = PublishBundleRequest {
            url: "https://example.com/bundles/v2.json".into(),
            signer_fingerprint: "fp-signer".into(),
        };

        let resp = publish_bundle_impl(state.clone(), web::Json(req)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let created: TrustBundleMeta = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(created.url, "https://example.com/bundles/v2.json");
        assert!(!created.signature.is_empty());
        assert!(created.payload.get("roots").is_some());

        let fetched_resp = get_latest_bundle_impl(state).await.unwrap();
        let fetched: TrustBundleMeta = serde_json::from_slice(&to_bytes(fetched_resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(fetched.version, created.version);
    }
}
