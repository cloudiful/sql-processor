use regex::Regex;
use std::sync::LazyLock;

use crate::{Token, TokenKind, lex_sql, script::ScriptUnit};

static MULTI_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());
static SINGLE_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"--.*").unwrap());
static CREATE_PLSQL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^CREATE\s+(OR\s+REPLACE\s+)?(PROCEDURE|FUNCTION|PACKAGE|TRIGGER|TYPE|BODY|LIBRARY)\b",
    )
    .unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Terminator {
    None,
    Semicolon,
    Slash,
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptTail {
    pub has_executable: bool,
    pub last_unit: ScriptUnit,
    pub insertion_pos: usize,
    pub terminator: Terminator,
}

pub(crate) fn remove_sql_comments(content: &str) -> String {
    SINGLE_COMMENT
        .replace_all(&MULTI_COMMENT.replace_all(content, ""), "")
        .to_string()
}

pub(crate) fn is_plsql(content: &str) -> bool {
    let upper = content.trim().to_uppercase();
    if upper.ends_with("END;") || upper.ends_with("END") {
        return true;
    }
    let stripped = content.trim_start();
    let upper_stripped = stripped.to_uppercase();
    upper_stripped.starts_with("BEGIN")
        || upper_stripped.starts_with("DECLARE")
        || CREATE_PLSQL.is_match(stripped)
}

pub(crate) fn split_sql_plus_statements(sql: &str) -> Vec<String> {
    let normalized = sql.replace("\r\n", "\n");
    let mut result = Vec::new();
    let mut current = String::new();
    for line in normalized.split('\n') {
        if line.trim() == "/" {
            append_sql_plus_chunk(&mut result, &current);
            current.clear();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    append_sql_plus_chunk(&mut result, &current);
    result
}

fn append_sql_plus_chunk(result: &mut Vec<String>, chunk: &str) {
    let trimmed = chunk.trim();
    if trimmed.is_empty() {
        return;
    }
    let cleaned = remove_sql_comments(trimmed);
    if !is_plsql(&cleaned) {
        append_semicolon_statements(result, chunk);
        return;
    }
    if starts_with_plsql_chunk(chunk) {
        result.push(trimmed.to_string());
        return;
    }
    let normalized = chunk.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    for (index, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        if is_plsql_start_line(line) {
            append_semicolon_statements(result, &lines[..index].join("\n"));
            let suffix = lines[index..].join("\n").trim().to_string();
            if !suffix.is_empty() {
                result.push(suffix);
            }
            return;
        }
    }
    result.push(trimmed.to_string());
}

fn append_semicolon_statements(result: &mut Vec<String>, chunk: &str) {
    let tokens = lex_sql(chunk);
    let mut current = String::new();
    for token in tokens {
        if token.kind == TokenKind::Symbol && token.value == ";" {
            if !current.trim().is_empty() {
                result.push(current.trim().to_string());
            }
            current.clear();
        } else {
            current.push_str(&token.value);
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
}

fn starts_with_plsql_chunk(chunk: &str) -> bool {
    chunk
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("--"))
        .is_some_and(is_plsql_start_line)
}

fn is_plsql_start_line(line: &str) -> bool {
    let upper = line.trim().to_uppercase();
    upper == "DECLARE" || upper == "BEGIN" || CREATE_PLSQL.is_match(&upper)
}

pub(crate) fn analyze_sql_plus_script_tail(sql: &str) -> Option<ScriptTail> {
    let units = crate::script::split_sql_plus_script_units(sql);
    let last_unit = units.last()?.clone();
    let tokens = lex_sql(sql);
    let last = tokens
        .iter()
        .rev()
        .find(|token| !matches!(token.kind, TokenKind::Whitespace | TokenKind::Comment))?;
    let mut terminator = Terminator::None;
    if last.kind == TokenKind::Symbol && last.value == ";" {
        terminator = Terminator::Semicolon;
    } else if last.kind == TokenKind::Symbol && last.value == "/" && is_standalone_slash(sql, last)
    {
        terminator = Terminator::Slash;
    }
    Some(ScriptTail {
        has_executable: true,
        last_unit,
        insertion_pos: last.pos + last.value.len(),
        terminator,
    })
}

fn is_standalone_slash(sql: &str, token: &Token) -> bool {
    let line_start = sql[..token.pos].rfind('\n').map_or(0, |pos| pos + 1);
    let line_end = sql[token.pos..]
        .find('\n')
        .map_or(sql.len(), |offset| token.pos + offset);
    let line = &sql[line_start..line_end];
    let relative = token.pos - line_start;
    line[..relative].trim().is_empty() && line[relative + token.value.len()..].trim().is_empty()
}

pub(crate) fn script_unit_requires_slash(unit: &ScriptUnit) -> bool {
    !unit.text.trim().is_empty() && is_plsql(&remove_sql_comments(&unit.text))
}
