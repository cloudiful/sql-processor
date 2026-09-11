use crate::{
    CoreError, TokenKind, build_current_schema_directive,
    idempotent::try_make_idempotent,
    lex_sql,
    schema::normalize_schema_name,
    strings::rewrite_sql_plus_risky_string_literals,
    utils::{Terminator, analyze_sql_plus_script_tail, script_unit_requires_slash},
};

pub fn process_sql_text(
    mut content: String,
    target_schema: &str,
) -> Result<(String, Vec<String>), CoreError> {
    if content.trim().is_empty() {
        return Ok((content, Vec::new()));
    }
    let mut messages = Vec::new();
    let (wrapped, modified) = try_make_idempotent(&content);
    if modified {
        messages.push("Processed for idempotency".to_string());
        content = wrapped;
    }

    let mut has_dml = false;
    let mut has_ddl = false;
    let mut has_commit = false;
    for token in lex_sql(&content) {
        if token.kind != TokenKind::Keyword {
            continue;
        }
        match token.value.to_uppercase().as_str() {
            "INSERT" | "UPDATE" | "DELETE" | "MERGE" => has_dml = true,
            "CREATE" | "DROP" | "ALTER" | "TRUNCATE" => {
                if token.value.eq_ignore_ascii_case("ALTER")
                    && current_schema_at(&lex_sql(&content), token.pos).is_some()
                {
                    continue;
                }
                has_ddl = true;
            }
            "COMMIT" => has_commit = true,
            _ => {}
        }
    }

    let mut injected_terminator = false;
    let tail = analyze_sql_plus_script_tail(&content);
    let last_needs_slash = tail
        .as_ref()
        .is_some_and(|tail| tail.has_executable && script_unit_requires_slash(&tail.last_unit));
    if last_needs_slash
        && tail
            .as_ref()
            .is_some_and(|tail| tail.terminator != Terminator::Slash)
    {
        messages.push("Missing slash (PL/SQL detected)".to_string());
        let position = tail
            .as_ref()
            .map(|tail| tail.insertion_pos)
            .unwrap_or(content.len());
        content.insert_str(position, "\n/\n");
        injected_terminator = true;
    }

    let mut auto_commit = String::new();
    if !has_commit && (has_dml || (has_ddl && !last_needs_slash)) {
        messages.push("Missing commit command".to_string());
        if tail.as_ref().is_some_and(|tail| {
            tail.has_executable
                && !script_unit_requires_slash(&tail.last_unit)
                && tail.terminator == Terminator::None
        }) {
            let position = tail
                .as_ref()
                .map(|tail| tail.insertion_pos)
                .unwrap_or(content.len());
            content.insert(position, ';');
            injected_terminator = true;
        }
        if !content.ends_with('\n') {
            auto_commit.push('\n');
        }
        auto_commit.push_str("-- [AUTO-GENERATED] Commit added for DML\nCOMMIT;\n");
    }

    let before_rewrite = format!("{content}{auto_commit}");
    let (mut final_content, strings_modified) =
        rewrite_sql_plus_risky_string_literals(&before_rewrite);
    if strings_modified {
        messages.push("Processed for SQL*Plus-safe string concatenation".to_string());
    }

    let mut session_added = false;
    if !target_schema.trim().is_empty() {
        let normalized = normalize_schema_name(target_schema)
            .ok_or_else(|| CoreError::InvalidSchema(target_schema.trim().to_string()))?;
        if !has_leading_current_schema(&final_content, &normalized) {
            let directive = build_current_schema_directive(&normalized)?;
            final_content = if final_content.trim().is_empty() {
                format!("{directive}\n")
            } else {
                format!("{directive}\n\n{final_content}")
            };
            messages.push(format!(
                "Added CURRENT_SCHEMA session directive (Target: {normalized})"
            ));
            session_added = true;
        }
    }

    if modified
        || injected_terminator
        || !auto_commit.is_empty()
        || strings_modified
        || session_added
    {
        final_content = format!(
            "-- [AUTO-GENERATED] This file has been automatically processed\n{final_content}"
        );
    }
    if !final_content.ends_with('\n') {
        final_content.push('\n');
    }
    Ok((final_content, messages))
}

fn current_schema_at(tokens: &[crate::Token], start_pos: usize) -> Option<String> {
    let start = tokens.iter().position(|token| token.pos == start_pos)?;
    let meaningful: Vec<&crate::Token> = tokens[start..]
        .iter()
        .filter(|token| !matches!(token.kind, TokenKind::Whitespace | TokenKind::Comment))
        .take(6)
        .collect();
    if meaningful.len() < 6 {
        return None;
    }
    if !meaningful[0].value.eq_ignore_ascii_case("ALTER")
        || !meaningful[1].value.eq_ignore_ascii_case("SESSION")
        || !meaningful[2].value.eq_ignore_ascii_case("SET")
        || !meaningful[3].value.eq_ignore_ascii_case("CURRENT_SCHEMA")
        || meaningful[4].value != "="
        || meaningful[5].kind != TokenKind::Identifier
    {
        return None;
    }
    Some(meaningful[5].value.to_uppercase())
}

fn has_leading_current_schema(content: &str, target: &str) -> bool {
    let tokens = lex_sql(content);
    let meaningful: Vec<&crate::Token> = tokens
        .iter()
        .filter(|token| !matches!(token.kind, TokenKind::Whitespace | TokenKind::Comment))
        .take(6)
        .collect();
    meaningful.len() == 6
        && meaningful[0].value.eq_ignore_ascii_case("ALTER")
        && meaningful[1].value.eq_ignore_ascii_case("SESSION")
        && meaningful[2].value.eq_ignore_ascii_case("SET")
        && meaningful[3].value.eq_ignore_ascii_case("CURRENT_SCHEMA")
        && meaningful[4].value == "="
        && meaningful[5].value.eq_ignore_ascii_case(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_commit_and_schema_in_stable_order() {
        let (output, messages) =
            process_sql_text("insert into orders (id) values (1)".into(), "EBANK").unwrap();
        assert!(output.starts_with("-- [AUTO-GENERATED]"));
        assert!(output.contains("ALTER SESSION SET CURRENT_SCHEMA = EBANK;"));
        assert!(output.contains("COMMIT;"));
        assert!(
            messages
                .iter()
                .any(|message| message == "Missing commit command")
        );
    }

    #[test]
    fn adds_slash_to_plsql() {
        let (output, _) = process_sql_text("BEGIN\n NULL;\nEND;".into(), "").unwrap();
        assert!(output.contains("END;\n/"));
    }
}
