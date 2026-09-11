use crate::{
    lex_sql,
    utils::{is_plsql, remove_sql_comments},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptUnit {
    pub text: String,
    pub start_line: usize,
    pub statement_type: String,
}

pub fn split_sql_plus_script_units(sql: &str) -> Vec<ScriptUnit> {
    let normalized = sql.replace("\r\n", "\n");
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chunk_start_line = 1;
    for (index, line) in normalized.split('\n').enumerate() {
        let line_number = index + 1;
        if line.trim() == "/" {
            append_chunk_units(&mut result, &current, chunk_start_line);
            current.clear();
            chunk_start_line = line_number + 1;
            continue;
        }
        if current.is_empty() {
            chunk_start_line = line_number;
        }
        current.push_str(line);
        current.push('\n');
    }
    append_chunk_units(&mut result, &current, chunk_start_line);
    result
}

pub fn detect_statement_type(sql: &str) -> String {
    for token in lex_sql(sql) {
        if matches!(
            token.kind,
            crate::TokenKind::Whitespace | crate::TokenKind::Comment
        ) {
            continue;
        }
        if matches!(
            token.kind,
            crate::TokenKind::Keyword | crate::TokenKind::Identifier
        ) {
            let upper = token.value.to_uppercase();
            return if matches!(upper.as_str(), "DECLARE" | "BEGIN") {
                "PLSQL".to_string()
            } else {
                upper
            };
        }
        if token.kind == crate::TokenKind::Symbol {
            return token.value;
        }
    }
    "UNKNOWN".to_string()
}

fn append_chunk_units(result: &mut Vec<ScriptUnit>, chunk: &str, start_line: usize) {
    let trimmed = chunk.trim();
    if trimmed.is_empty() {
        return;
    }
    if !is_plsql(&remove_sql_comments(trimmed)) {
        append_semicolon_units(result, chunk, start_line);
        return;
    }
    if starts_with_plsql_chunk(chunk) {
        result.push(unit(trimmed, start_line));
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
            append_semicolon_units(result, &lines[..index].join("\n"), start_line);
            let suffix = lines[index..].join("\n").trim().to_string();
            if !suffix.is_empty() {
                result.push(unit(&suffix, start_line + index));
            }
            return;
        }
    }
    result.push(unit(trimmed, start_line));
}

fn append_semicolon_units(result: &mut Vec<ScriptUnit>, chunk: &str, start_line: usize) {
    let tokens = lex_sql(chunk);
    let mut current = String::new();
    let mut current_pos = None;
    for token in tokens {
        if current_pos.is_none()
            && !matches!(
                token.kind,
                crate::TokenKind::Whitespace | crate::TokenKind::Comment
            )
        {
            current_pos = Some(token.pos);
        }
        if token.kind == crate::TokenKind::Symbol && token.value == ";" {
            if let Some(pos) = current_pos
                && !current.trim().is_empty()
            {
                result.push(unit(
                    current.trim(),
                    start_line + chunk[..pos].matches('\n').count(),
                ));
            }
            current.clear();
            current_pos = None;
        } else {
            current.push_str(&token.value);
        }
    }
    if let Some(pos) = current_pos
        && !current.trim().is_empty()
    {
        result.push(unit(
            current.trim(),
            start_line + chunk[..pos].matches('\n').count(),
        ));
    }
}

fn unit(text: &str, start_line: usize) -> ScriptUnit {
    ScriptUnit {
        text: text.to_string(),
        start_line,
        statement_type: detect_statement_type(text),
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
    upper == "DECLARE"
        || upper == "BEGIN"
        || upper.starts_with("CREATE PROCEDURE")
        || upper.starts_with("CREATE OR REPLACE PROCEDURE")
        || upper.starts_with("CREATE FUNCTION")
        || upper.starts_with("CREATE OR REPLACE FUNCTION")
        || upper.starts_with("CREATE PACKAGE")
        || upper.starts_with("CREATE OR REPLACE PACKAGE")
        || upper.starts_with("CREATE TRIGGER")
        || upper.starts_with("CREATE OR REPLACE TRIGGER")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_units_and_tracks_lines() {
        let units = split_sql_plus_script_units(
            "-- header\nCREATE TABLE T (ID NUMBER)\n/\nSELECT 1 FROM DUAL;",
        );
        assert_eq!(units.len(), 2);
        assert_eq!(
            (units[0].statement_type.as_str(), units[0].start_line),
            ("CREATE", 2)
        );
        assert_eq!(units[1].statement_type, "SELECT");
    }
}
