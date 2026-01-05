use actix_web::{get, web, HttpResponse};

use crate::{error::ApiError, models::AuditEvent, AppState};

async fn list_events_impl(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let rows = sqlx::query_as::<_, AuditEvent>(
        "select id, event_type, actor, scope, payload, occurred_at from audit_logs order by occurred_at desc limit 100",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(rows))
}

#[get("/logs")]
pub async fn list_events_handler(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    list_events_impl(state).await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, http::StatusCode, web};
    use sqlx::PgPool;
    use uuid::Uuid;
    use crate::{models::AuditEvent, AppState};
    use super::list_events_impl;

    #[sqlx::test]
    async fn list_events_returns_inserted(pool: PgPool) {
        sqlx::query(
            "insert into audit_logs (id, event_type, actor, scope, payload) values ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind("test_event")
        .bind(Some("alice".to_string()))
        .bind(Some("pki.write".to_string()))
        .bind(serde_json::json!({"resource": "cert", "action": "issue"}))
        .execute(&pool)
        .await
        .unwrap();

        let state = web::Data::new(AppState { db: pool });
        let resp = list_events_impl(state).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let events: Vec<AuditEvent> = serde_json::from_slice(&to_bytes(resp.into_body()).await.unwrap()).unwrap();
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, "test_event");
    }
}
