use std::{env, time::Duration};

use async_trait::async_trait;
use bytes::Bytes;
use oracle_rs::{Config as OracleConfig, Connection, Value};
use regex::Regex;
use sql_processor_core::{
    ScriptUnit, build_current_schema_directive, normalize_schema_name, split_sql_plus_script_units,
};
use std::sync::LazyLock;
use tokio::time::timeout;

use crate::{Config, Input, Issue, Result, Runner, Target, unavailable_result, unavailable_target};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const PARSE_BLOCK: &str = "DECLARE c INTEGER; BEGIN c := DBMS_SQL.OPEN_CURSOR; BEGIN DBMS_SQL.PARSE(c, :1, DBMS_SQL.NATIVE); DBMS_SQL.CLOSE_CURSOR(c); EXCEPTION WHEN OTHERS THEN IF DBMS_SQL.IS_OPEN(c) THEN DBMS_SQL.CLOSE_CURSOR(c); END IF; RAISE; END; END;";
static ORACLE_CODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"ORA-\d{5}").unwrap());
static ORACLE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)ORA-\d{5}.*?line\s+(\d+)\s*,\s*column\s+\d+").unwrap());

pub struct OracleRunner {
    target: Option<OracleTarget>,
    timeout: Duration,
}

struct OracleTarget {
    host: String,
    port: u16,
    service: String,
    username: String,
    password: String,
}

pub fn new_oracle_runner_from_env() -> OracleRunner {
    let timeout = env::var("SQL_PROCESSOR_VALIDATE_TIMEOUT")
        .ok()
        .and_then(|value| parse_duration(&value))
        .unwrap_or(DEFAULT_TIMEOUT);
    OracleRunner {
        target: None,
        timeout,
    }
}

pub fn new_oracle_runner_from_config(config: &Config) -> Box<dyn Runner> {
    let missing = [
        ("host", config.host.trim().is_empty()),
        ("port", config.port == 0),
        ("service", config.service.trim().is_empty()),
        ("username", config.username.trim().is_empty()),
        ("password", config.password.trim().is_empty()),
    ]
    .into_iter()
    .filter_map(|(name, missing)| missing.then_some(name))
    .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Box::new(UnavailableRunner {
            reason: format!(
                "Oracle validation server configuration is incomplete: missing {}.",
                missing.join(", ")
            ),
        });
    }
    Box::new(OracleRunner {
        target: Some(OracleTarget {
            host: config.host.trim().to_string(),
            port: config.port,
            service: config.service.trim().to_string(),
            username: config.username.trim().to_string(),
            password: config.password.clone(),
        }),
        timeout: if config.timeout_seconds > 0 {
            Duration::from_secs(config.timeout_seconds)
        } else {
            DEFAULT_TIMEOUT
        },
    })
}

struct UnavailableRunner {
    reason: String,
}

#[async_trait]
impl Runner for UnavailableRunner {
    async fn validate(&self, _input: Input) -> Result {
        unavailable_result(&self.reason)
    }
}

#[async_trait]
impl Runner for OracleRunner {
    async fn validate(&self, input: Input) -> Result {
        let Some(target) = &self.target else {
            return unavailable_result("SQL_PROCESSOR_VALIDATE_ORACLE_DSN is not configured");
        };
        let config = OracleConfig::new(
            &target.host,
            target.port,
            &target.service,
            &target.username,
            &target.password,
        )
        .connect_timeout(self.timeout);
        let connection = match timeout(self.timeout, Connection::connect_with_config(config)).await
        {
            Ok(Ok(connection)) => connection,
            Ok(Err(error)) => {
                return unavailable_result(format!("connect to Oracle validator: {error}"));
            }
            Err(_) => {
                return unavailable_result(format!(
                    "connect to Oracle validator: timed out after {:?}",
                    self.timeout
                ));
            }
        };
        let result = Result {
            targets: vec![
                self.validate_target(
                    &connection,
                    "original",
                    &input.original_sql,
                    &input.target_schema,
                )
                .await,
                self.validate_target(&connection, "processed", &input.processed_sql, "")
                    .await,
            ],
        };
        let _ = connection.close().await;
        result
    }
}

impl OracleRunner {
    async fn validate_target(
        &self,
        connection: &Connection,
        name: &str,
        sql: &str,
        schema: &str,
    ) -> Target {
        if !schema.trim().is_empty() {
            let Some(normalized) = normalize_schema_name(schema) else {
                return unavailable_target(
                    name,
                    &format!("invalid target schema for validation: {}", schema.trim()),
                );
            };
            let Ok(directive) = build_current_schema_directive(&normalized) else {
                return unavailable_target(name, "failed to build CURRENT_SCHEMA directive");
            };
            match timeout(self.timeout, connection.execute(&directive, &[])).await {
                Ok(Ok(_)) => {}
                Ok(Err(error)) => {
                    return unavailable_target(
                        name,
                        &format!("set CURRENT_SCHEMA to {normalized}: {error}"),
                    );
                }
                Err(error) => {
                    return unavailable_target(
                        name,
                        &format!("set CURRENT_SCHEMA to {normalized}: timed out: {error}"),
                    );
                }
            }
        }

        let units = split_sql_plus_script_units(sql);
        if units.is_empty() {
            return Target {
                target: name.to_string(),
                status: "passed".to_string(),
                summary: vec!["No executable statements to validate.".to_string()],
                issues: Vec::new(),
            };
        }
        let mut validated = 0;
        let mut skipped = 0;
        let mut failed = 0;
        let mut issues = Vec::new();
        for (index, unit) in units.iter().enumerate() {
            if should_skip(&unit.statement_type) {
                skipped += 1;
                issues.push(Issue { status: "skipped".to_string(), statement_type: unit.statement_type.clone(), unit_index: index + 1, line: unit.start_line, code: None, message: "DDL skipped in safe mode; Oracle parse-only validation is not run for this statement type.".to_string(), snippet: Some(snippet(&unit.text)) });
                continue;
            }
            let parameter = if unit.text.len() > 32_767 {
                Value::Lob(oracle_rs::types::LobValue::inline(Bytes::from(
                    unit.text.clone(),
                )))
            } else {
                Value::String(unit.text.clone())
            };
            match timeout(self.timeout, connection.execute(PARSE_BLOCK, &[parameter])).await {
                Ok(Ok(_)) => validated += 1,
                Ok(Err(error)) => {
                    failed += 1;
                    issues.push(failed_issue(index + 1, unit, &error.to_string()));
                }
                Err(error) => {
                    failed += 1;
                    issues.push(failed_issue(
                        index + 1,
                        unit,
                        &format!("Oracle parse timeout: {error}"),
                    ));
                }
            }
        }
        let status = if failed > 0 {
            "failed"
        } else if skipped > 0 {
            "partial"
        } else {
            "passed"
        };
        let mut summary = vec![format!(
            "Validated {validated} of {} execution units.",
            units.len()
        )];
        if skipped > 0 {
            summary.push(format!(
                "{skipped} execution unit(s) were skipped in safe mode."
            ));
        }
        if failed > 0 {
            summary.push(format!("{failed} Oracle parse error(s) found."));
        }
        if failed == 0 && skipped == 0 {
            summary.push("Oracle parse-only validation passed.".to_string());
        }
        Target {
            target: name.to_string(),
            status: status.to_string(),
            summary,
            issues,
        }
    }
}

fn parse_duration(value: &str) -> Option<Duration> {
    let value = value.trim();
    if let Some(seconds) = value.strip_suffix('s') {
        seconds
            .parse::<u64>()
            .ok()
            .filter(|seconds| *seconds > 0)
            .map(Duration::from_secs)
    } else {
        value
            .parse::<u64>()
            .ok()
            .filter(|seconds| *seconds > 0)
            .map(Duration::from_secs)
    }
}

fn should_skip(statement_type: &str) -> bool {
    matches!(
        statement_type.to_uppercase().as_str(),
        "CREATE" | "ALTER" | "DROP" | "TRUNCATE" | "COMMENT" | "GRANT" | "REVOKE" | "RENAME"
    )
}

fn failed_issue(index: usize, unit: &ScriptUnit, message: &str) -> Issue {
    let code = ORACLE_CODE
        .find(message)
        .map(|match_| match_.as_str().to_string());
    let line = ORACLE_LINE
        .captures(message)
        .and_then(|captures| captures.get(1))
        .and_then(|line| line.as_str().parse::<usize>().ok())
        .map_or(unit.start_line, |line| {
            unit.start_line.saturating_add(line.saturating_sub(1))
        });
    Issue {
        status: "failed".to_string(),
        statement_type: unit.statement_type.clone(),
        unit_index: index,
        line,
        code,
        message: message.to_string(),
        snippet: Some(snippet(&unit.text)),
    }
}

fn snippet(sql: &str) -> String {
    let mut value = sql.trim().replace("\r\n", "\n").replace('\n', " ");
    value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.len() > 160 {
        value.truncate(157);
        value.push_str("...");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_oracle_code_and_relative_error_line() {
        let unit = ScriptUnit {
            text: "SELECT\n  missing_column\nFROM orders".to_string(),
            start_line: 12,
            statement_type: "SELECT".to_string(),
        };
        let issue = failed_issue(2, &unit, "ORA-06550: line 2, column 3: invalid identifier");
        assert_eq!(issue.code.as_deref(), Some("ORA-06550"));
        assert_eq!(issue.line, 13);
    }
}
