use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct HealthResponse {
    status: String,
    db: String,
}

#[get("/health")]
pub async fn health(state: web::Data<AppState>) -> impl Responder {
    health_internal(|| async {
        sqlx::query_scalar::<_, i32>("select 1")
            .fetch_one(&state.db)
            .await
            .map(|_| true)
            .unwrap_or(false)
    })
    .await
}

async fn health_internal<F, Fut>(db_check: F) -> HttpResponse
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let db_ok = db_check().await;
    let status = if db_ok { HttpResponse::Ok } else { HttpResponse::ServiceUnavailable };
    let body = HealthResponse {
        status: "ok".to_string(),
        db: if db_ok { "ok".to_string() } else { "error".to_string() },
    };
    status().json(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn health_internal_reports_ok_when_db_ok() {
        let resp = health_internal(|| async { true }).await;
        let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        let body: HealthResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body.db, "ok");
        assert_eq!(body.status, "ok");
    }

    #[actix_rt::test]
    async fn health_internal_reports_error_when_db_fails() {
        let resp = health_internal(|| async { false }).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::SERVICE_UNAVAILABLE);
        let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        let body: HealthResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body.db, "error");
    }
}
