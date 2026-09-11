use regex::Regex;
use std::sync::LazyLock;

use crate::CoreError;

static SCHEMA_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z][A-Za-z0-9_$#]*$").unwrap());

pub fn normalize_schema_name(schema: &str) -> Option<String> {
    let trimmed = schema.trim();
    if trimmed.is_empty() || !SCHEMA_NAME.is_match(trimmed) {
        return None;
    }
    Some(trimmed.to_uppercase())
}

pub fn build_current_schema_directive(schema: &str) -> Result<String, CoreError> {
    let normalized = normalize_schema_name(schema)
        .ok_or_else(|| CoreError::InvalidSchema(schema.trim().to_string()))?;
    Ok(format!("ALTER SESSION SET CURRENT_SCHEMA = {normalized};"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_valid_schema() {
        assert_eq!(normalize_schema_name(" ebank "), Some("EBANK".to_string()));
        assert!(normalize_schema_name("bad-schema").is_none());
    }
}
