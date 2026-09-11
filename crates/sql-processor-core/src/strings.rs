use crate::{TokenKind, lex_sql, lexer::matching_quote_delimiter};

pub(crate) fn rewrite_sql_plus_risky_string_literals(sql: &str) -> (String, bool) {
    let mut output = String::new();
    let mut modified = false;
    for token in lex_sql(sql) {
        if token.kind != TokenKind::String {
            output.push_str(&token.value);
            continue;
        }
        let Some(content) = decode_oracle_literal(&token.value) else {
            output.push_str(&token.value);
            continue;
        };
        if should_rewrite(&content) {
            output.push_str(&build_concat_expression(&content));
            modified = true;
        } else {
            output.push_str(&token.value);
        }
    }
    (output, modified)
}

fn should_rewrite(content: &str) -> bool {
    if content.contains('&') {
        return true;
    }
    if !content.contains('\r') && !content.contains('\n') {
        return false;
    }
    content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.is_empty() || trimmed.starts_with('@') || trimmed == "/" || trimmed.ends_with(';')
    })
}

fn decode_oracle_literal(raw: &str) -> Option<String> {
    if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
        let mut result = String::new();
        let inner = &raw[1..raw.len() - 1];
        let mut chars = inner.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\'' && chars.peek() == Some(&'\'') {
                chars.next();
            }
            result.push(ch);
        }
        return Some(result);
    }
    if raw.len() < 4
        || !matches!(raw.as_bytes()[0], b'q' | b'Q')
        || !raw[1..].starts_with('\'')
        || !raw.ends_with('\'')
    {
        return None;
    }
    let open = raw[2..].chars().next()?;
    let close = matching_quote_delimiter(open);
    let content_start = 2 + open.len_utf8();
    let content_end = raw.len() - 1 - close.len_utf8();
    if content_end < content_start || !raw[content_end..raw.len() - 1].starts_with(close) {
        return None;
    }
    Some(raw[content_start..content_end].to_string())
}

fn build_concat_expression(content: &str) -> String {
    if content.contains('\r') || content.contains('\n') {
        build_multiline(content)
    } else {
        let parts = concat_parts(content);
        if parts.is_empty() {
            "('')".to_string()
        } else {
            format!("({})", parts.join(" || "))
        }
    }
}

fn build_multiline(content: &str) -> String {
    let mut rendered = Vec::new();
    let mut line_parts = Vec::new();
    let mut chunk = String::new();
    let mut chars = content.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '&' => {
                flush_chunk(&mut chunk, &mut line_parts);
                line_parts.push("chr(38)".to_string());
            }
            '\r' => {
                flush_chunk(&mut chunk, &mut line_parts);
                line_parts.push("chr(13)".to_string());
                if chars.peek() == Some(&'\n') {
                    chars.next();
                    line_parts.push("chr(10)".to_string());
                }
                flush_line(&mut line_parts, &mut rendered, chars.peek().is_some());
            }
            '\n' => {
                flush_chunk(&mut chunk, &mut line_parts);
                line_parts.push("chr(10)".to_string());
                flush_line(&mut line_parts, &mut rendered, chars.peek().is_some());
            }
            other => chunk.push(other),
        }
    }
    flush_chunk(&mut chunk, &mut line_parts);
    flush_line(&mut line_parts, &mut rendered, false);
    if rendered.is_empty() {
        "('')".to_string()
    } else {
        format!("({})", rendered.join("\n"))
    }
}

fn flush_chunk(chunk: &mut String, parts: &mut Vec<String>) {
    if !chunk.is_empty() {
        parts.push(quote_plain_chunk(chunk));
        chunk.clear();
    }
}

fn flush_line(parts: &mut Vec<String>, rendered: &mut Vec<String>, has_more: bool) {
    if parts.is_empty() {
        return;
    }
    let mut line = parts.join(" || ");
    if has_more {
        line.push_str(" ||");
    }
    rendered.push(line);
    parts.clear();
}

fn concat_parts(content: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut chunk = String::new();
    for ch in content.chars() {
        match ch {
            '&' => {
                flush_chunk(&mut chunk, &mut parts);
                parts.push("chr(38)".to_string());
            }
            '\n' => {
                flush_chunk(&mut chunk, &mut parts);
                parts.push("chr(10)".to_string());
            }
            '\r' => {
                flush_chunk(&mut chunk, &mut parts);
                parts.push("chr(13)".to_string());
            }
            other => chunk.push(other),
        }
    }
    flush_chunk(&mut chunk, &mut parts);
    parts
}

fn quote_plain_chunk(chunk: &str) -> String {
    for (open, close) in [
        ("~", "~"),
        ("!", "!"),
        ("#", "#"),
        ("%", "%"),
        ("^", "^"),
        ("|", "|"),
        ("[", "]"),
        ("{", "}"),
        ("(", ")"),
        ("<", ">"),
    ]
    .iter()
    {
        if !chunk.contains(&format!("{close}'")) {
            return format!("q'{open}{chunk}{close}'");
        }
    }
    format!("'{}'", chunk.replace('\'', "''"))
}
