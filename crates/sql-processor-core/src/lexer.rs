use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Error,
    Eof,
    Keyword,
    Identifier,
    String,
    Number,
    Symbol,
    Comment,
    Whitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub pos: usize,
}

static KEYWORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "SELECT",
        "FROM",
        "WHERE",
        "INSERT",
        "INTO",
        "UPDATE",
        "DELETE",
        "CREATE",
        "DROP",
        "ALTER",
        "TABLE",
        "INDEX",
        "VIEW",
        "SEQUENCE",
        "SYNONYM",
        "JOIN",
        "LEFT",
        "RIGHT",
        "INNER",
        "OUTER",
        "ON",
        "AND",
        "OR",
        "AS",
        "GROUP",
        "BY",
        "ORDER",
        "HAVING",
        "DISTINCT",
        "UNION",
        "VALUES",
        "SET",
        "TRUNCATE",
        "GRANT",
        "REVOKE",
        "COMMIT",
        "ROLLBACK",
        "TRIGGER",
        "PROCEDURE",
        "FUNCTION",
        "PACKAGE",
        "BODY",
        "CONSTRAINT",
        "PRIMARY",
        "KEY",
        "FOREIGN",
        "REFERENCES",
        "CHECK",
        "UNIQUE",
        "DEFAULT",
        "NULL",
        "NOT",
        "MERGE",
    ]
});

pub fn lex_sql(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < input.len() {
        let ch = input[cursor..]
            .chars()
            .next()
            .expect("cursor is a char boundary");
        let size = ch.len_utf8();

        if ch.is_whitespace() {
            let start = cursor;
            cursor += size;
            while cursor < input.len() {
                let next = input[cursor..].chars().next().expect("char boundary");
                if !next.is_whitespace() {
                    break;
                }
                cursor += next.len_utf8();
            }
            tokens.push(token(TokenKind::Whitespace, input, start, cursor));
            continue;
        }

        if input[cursor..].starts_with("--") {
            let start = cursor;
            cursor += 2;
            while cursor < input.len() && input.as_bytes()[cursor] != b'\n' {
                let next = input[cursor..].chars().next().expect("char boundary");
                cursor += next.len_utf8();
            }
            tokens.push(token(TokenKind::Comment, input, start, cursor));
            continue;
        }

        if input[cursor..].starts_with("/*") {
            let start = cursor;
            cursor += 2;
            while cursor < input.len() {
                if input[cursor..].starts_with("*/") {
                    cursor += 2;
                    break;
                }
                let next = input[cursor..].chars().next().expect("char boundary");
                cursor += next.len_utf8();
            }
            tokens.push(token(TokenKind::Comment, input, start, cursor));
            continue;
        }

        if matches!(ch, 'q' | 'Q')
            && input[cursor + size..].starts_with("'")
            && let Some(end) = scan_alternative_quote(input, cursor)
        {
            tokens.push(token(TokenKind::String, input, cursor, end));
            cursor = end;
            continue;
        }

        if ch == '\'' {
            let start = cursor;
            cursor += 1;
            while cursor < input.len() {
                if input.as_bytes()[cursor] == b'\'' {
                    if input[cursor + 1..].starts_with("'") {
                        cursor += 2;
                        continue;
                    }
                    cursor += 1;
                    break;
                }
                let next = input[cursor..].chars().next().expect("char boundary");
                cursor += next.len_utf8();
            }
            tokens.push(token(TokenKind::String, input, start, cursor));
            continue;
        }

        if ch == '"' {
            let start = cursor;
            cursor += 1;
            while cursor < input.len() {
                let next = input[cursor..].chars().next().expect("char boundary");
                cursor += next.len_utf8();
                if next == '"' {
                    break;
                }
            }
            tokens.push(token(TokenKind::Identifier, input, start, cursor));
            continue;
        }

        if is_ident_start(ch) {
            let start = cursor;
            cursor += size;
            while cursor < input.len() {
                let next = input[cursor..].chars().next().expect("char boundary");
                if !is_ident_part(next) {
                    break;
                }
                cursor += next.len_utf8();
            }
            let value = &input[start..cursor];
            let kind = if is_keyword(value) {
                TokenKind::Keyword
            } else {
                TokenKind::Identifier
            };
            tokens.push(Token {
                kind,
                value: value.to_string(),
                pos: start,
            });
            continue;
        }

        if ch.is_ascii_digit() {
            let start = cursor;
            cursor += size;
            while cursor < input.len() {
                let next = input[cursor..].chars().next().expect("char boundary");
                if !next.is_ascii_digit() && next != '.' {
                    break;
                }
                cursor += next.len_utf8();
            }
            tokens.push(token(TokenKind::Number, input, start, cursor));
            continue;
        }

        tokens.push(token(TokenKind::Symbol, input, cursor, cursor + size));
        cursor += size;
    }
    tokens
}

fn token(kind: TokenKind, input: &str, start: usize, end: usize) -> Token {
    Token {
        kind,
        value: input[start..end].to_string(),
        pos: start,
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}
fn is_ident_part(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '$' | '#')
}
fn is_keyword(value: &str) -> bool {
    KEYWORDS
        .iter()
        .any(|keyword| keyword.eq_ignore_ascii_case(value))
}

fn scan_alternative_quote(input: &str, start: usize) -> Option<usize> {
    let mut cursor = start + input[start..].chars().next()?.len_utf8();
    if !input[cursor..].starts_with("'") {
        return None;
    }
    cursor += 1;
    let open = input[cursor..].chars().next()?;
    if open.is_whitespace() {
        return None;
    }
    cursor += open.len_utf8();
    let close = matching_quote_delimiter(open);
    while cursor < input.len() {
        let next = input[cursor..].chars().next()?;
        if next == close {
            let after = cursor + next.len_utf8();
            if input[after..].starts_with("'") {
                return Some(after + 1);
            }
        }
        cursor += next.len_utf8();
    }
    None
}

pub fn matching_quote_delimiter(open: char) -> char {
    match open {
        '[' => ']',
        '{' => '}',
        '(' => ')',
        '<' => '>',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_oracle_alternative_quoted_strings_together() {
        let tokens = lex_sql("select q'[first; second;]' from dual;");
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == TokenKind::String && token.value.contains("first;"))
        );
    }
}
