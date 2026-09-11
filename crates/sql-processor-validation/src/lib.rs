mod oracle;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use oracle::{new_oracle_runner_from_config, new_oracle_runner_from_env};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub profile_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub host: String,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub service: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub username: String,
    #[schema(write_only = true)]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub password: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub timeout_seconds: u64,
}

fn is_zero(value: &u64) -> bool {
    *value == 0
}

fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

#[derive(Debug, Clone)]
pub struct Input {
    pub original_sql: String,
    pub processed_sql: String,
    pub target_schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub target: String,
    pub status: String,
    pub summary: Vec<String>,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub status: String,
    pub statement_type: String,
    pub unit_index: usize,
    pub line: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[async_trait]
pub trait Runner: Send + Sync {
    async fn validate(&self, input: Input) -> Result;
}

pub(crate) fn unavailable_result(reason: impl Into<String>) -> Result {
    let reason = reason.into();
    Result {
        targets: vec![
            unavailable_target("original", &reason),
            unavailable_target("processed", &reason),
        ],
    }
}

pub(crate) fn unavailable_target(target: &str, reason: &str) -> Target {
    Target {
        target: target.to_string(),
        status: "unavailable".to_string(),
        summary: vec![
            "Oracle validation unavailable.".to_string(),
            reason.to_string(),
        ],
        issues: Vec::new(),
    }
}
