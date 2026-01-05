use actix_web::{get, put, web, HttpResponse};
use serde::Deserialize;

use crate::{error::ApiError, models::Policy, AppState};

async fn get_policy_impl(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let row = sqlx::query_as::<_, Policy>(
        "select subject_id_pattern, allow_ca_issue, updated_at from policy where id = 1",
    )
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Err(ApiError::NotFound),
    }
}

#[derive(Deserialize)]
pub struct UpdatePolicyRequest {
    pub subject_id_pattern: Option<String>,
    pub allow_ca_issue: bool,
}

async fn update_policy_impl(
    state: web::Data<AppState>,
    req: web::Json<UpdatePolicyRequest>,
) -> Result<HttpResponse, ApiError> {
    let updated = sqlx::query_as::<_, Policy>(
        "insert into policy (id, subject_id_pattern, allow_ca_issue) values (1, $1, $2)
         on conflict (id) do update set subject_id_pattern = excluded.subject_id_pattern, allow_ca_issue = excluded.allow_ca_issue, updated_at = now()
         returning subject_id_pattern, allow_ca_issue, updated_at",
    )
    .bind(&req.subject_id_pattern)
    .bind(req.allow_ca_issue)
    .fetch_one(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(updated))
}

#[get("")]
pub async fn get_policy_handler(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    get_policy_impl(state).await
}

#[put("")]
pub async fn update_policy_handler(
    state: web::Data<AppState>,
    req: web::Json<UpdatePolicyRequest>,
) -> Result<HttpResponse, ApiError> {
    update_policy_impl(state, req).await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, http::StatusCode, web};
    use sqlx::PgPool;
    use crate::{models::Policy, AppState};
    use super::{get_policy_impl, update_policy_impl, UpdatePolicyRequest};

    #[sqlx::test]
    async fn policy_round_trip(pool: PgPool) {
        let state = web::Data::new(AppState { db: pool });

        // Update policy (upsert) - creates if not exists
        let req = UpdatePolicyRequest {
            subject_id_pattern: Some("^subj-.*$".into()),
            allow_ca_issue: true,
        };
        let resp = update_policy_impl(state.clone(), web::Json(req)).await.unwrap();
        let updated: Policy = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(updated.allow_ca_issue, true);
        assert_eq!(updated.subject_id_pattern.as_deref(), Some("^subj-.*$"));

        // Now get should succeed
        let resp = get_policy_impl(state).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let fetched: Policy = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert_eq!(fetched.allow_ca_issue, true);
    }
}
