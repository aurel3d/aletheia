use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Root {
    pub id: Uuid,
    pub name: String,
    pub fingerprint: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Intermediate {
    pub id: Uuid,
    pub parent_id: Uuid,
    pub name: String,
    pub fingerprint: String,
    pub path_len: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Certificate {
    pub serial: String,
    pub issuer_id: Option<Uuid>,
    pub subject_id: String,
    pub subject_name: String,
    pub is_ca: bool,
    pub public_key: Vec<u8>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Revocation {
    pub serial: String,
    pub reason: Option<String>,
    pub revoked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrustBundleMeta {
    pub version: String,
    pub issued_at: DateTime<Utc>,
    pub url: String,
    pub signer_fingerprint: String,
    pub status: String,
    pub payload: serde_json::Value,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Policy {
    pub subject_id_pattern: Option<String>,
    pub allow_ca_issue: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditEvent {
    pub id: Uuid,
    pub event_type: String,
    pub actor: Option<String>,
    pub scope: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub occurred_at: DateTime<Utc>,
}
