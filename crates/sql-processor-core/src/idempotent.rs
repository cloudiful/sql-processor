use regex::Regex;
use std::sync::LazyLock;

use crate::utils::{remove_sql_comments, split_sql_plus_statements};

static CREATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^CREATE\s+(?:OR\s+REPLACE\s+)?(?:PUBLIC\s+)?(?:(?:UNIQUE|BITMAP)\s+)?(INDEX|TABLE|SEQUENCE|SYNONYM|VIEW|TRIGGER|USER|ROLE|DATABASE\s+LINK)\b").unwrap()
});
static DROP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^DROP\s+(?:PUBLIC\s+)?(INDEX|TABLE|VIEW|SEQUENCE|SYNONYM|TRIGGER|USER|ROLE|DATABASE\s+LINK)\b").unwrap()
});

pub(crate) fn try_make_idempotent(sql: &str) -> (String, bool) {
    let mut result = Vec::new();
    let mut modified = false;
    for raw in split_sql_plus_statements(sql) {
        let trimmed_raw = raw.trim();
        if trimmed_raw.is_empty() {
            continue;
        }
        let cleaned = remove_sql_comments(trimmed_raw).trim().to_string();
        if let Some(captures) = CREATE.captures(&cleaned) {
            let object_type = captures[1]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_uppercase();
            let (code, comment) = match object_type.as_str() {
                "USER" => (
                    -1920,
                    "user or role name conflicts with another user or role name",
                ),
                "ROLE" => (-1921, "role name conflicts with another user or role name"),
                "DATABASE LINK" => (-2011, "duplicate database link name"),
                _ => (-955, "name is already used by an existing object"),
            };
            result.push(create_block(&cleaned, code, comment));
            modified = true;
        } else if let Some(captures) = DROP.captures(&cleaned) {
            let object_type = captures[1]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_uppercase();
            let (code, comment) = match object_type.as_str() {
                "INDEX" => (-1418, "specified index does not exist"),
                "SEQUENCE" => (-2289, "sequence does not exist"),
                "SYNONYM" if cleaned.to_uppercase().contains("PUBLIC") => {
                    (-1432, "public synonym does not exist")
                }
                "SYNONYM" => (-1434, "private synonym does not exist"),
                "TRIGGER" => (-4043, "object does not exist"),
                "USER" => (-1918, "user does not exist"),
                "ROLE" => (-1919, "role does not exist"),
                "DATABASE LINK" => (-2024, "database link not found"),
                _ => (-942, "table or view does not exist"),
            };
            result.push(drop_block(&cleaned, code, comment));
            modified = true;
        } else if cleaned.to_uppercase().starts_with("ALTER TABLE") {
            if let Some(block) = alter_block(&cleaned) {
                result.push(block);
                modified = true;
            } else {
                result.push(format!("{trimmed_raw};"));
            }
        } else {
            result.push(format!("{trimmed_raw};"));
        }
    }
    if modified {
        (result.join("\n\n"), true)
    } else {
        (sql.to_string(), false)
    }
}

fn create_block(statement: &str, code: i32, comment: &str) -> String {
    format!(
        "-- [AUTO-GENERATED] Idempotent wrapper for:\n{}\nDECLARE\n    e_exists EXCEPTION;\n    PRAGMA EXCEPTION_INIT(e_exists, {code}); -- ORA-{:05}: {comment}\nBEGIN\n    EXECUTE IMMEDIATE '{}';\nEXCEPTION\n    WHEN e_exists THEN\n        DBMS_OUTPUT.PUT_LINE('Warning: Object already exists, skipping creation.');\n        NULL;\nEND;\n/",
        comment_statement(statement),
        -code,
        statement.replace("'", "''")
    )
}

fn drop_block(statement: &str, code: i32, comment: &str) -> String {
    format!(
        "-- [AUTO-GENERATED] Idempotent wrapper for:\n{}\nDECLARE\n    e_not_exists EXCEPTION;\n    PRAGMA EXCEPTION_INIT(e_not_exists, {code}); -- ORA-{:05}: {comment}\nBEGIN\n    EXECUTE IMMEDIATE '{}';\nEXCEPTION\n    WHEN e_not_exists THEN\n        DBMS_OUTPUT.PUT_LINE('Warning: Object does not exist, skipping drop.');\n        NULL;\nEND;\n/",
        comment_statement(statement),
        -code,
        statement.replace("'", "''")
    )
}

fn alter_block(statement: &str) -> Option<String> {
    let upper = statement.to_uppercase();
    let (codes, comments) = if upper.contains(" CONSTRAINT ") {
        if upper.contains(" ADD ") {
            let mut codes = vec![-2264];
            let mut comments = vec!["name already used by an existing constraint"];
            if upper.contains(" UNIQUE") {
                codes.push(-2261);
                comments.push("unique or primary key already exists");
            } else if upper.contains(" PRIMARY KEY") {
                codes.push(-2260);
                comments.push("table can have only one primary key");
            } else if upper.contains(" FOREIGN KEY") {
                codes.push(-2275);
                comments.push("referential constraint already exists");
            }
            (codes, comments)
        } else if upper.contains(" DROP ") {
            (vec![-2443], vec!["nonexistent constraint"])
        } else {
            return None;
        }
    } else if upper.contains(" PRIMARY KEY") {
        if upper.contains(" ADD ") {
            (vec![-2260], vec!["table can have only one primary key"])
        } else if upper.contains(" DROP ") {
            (vec![-2441], vec!["nonexistent primary key"])
        } else {
            return None;
        }
    } else if upper.contains(" UNIQUE") {
        if upper.contains(" ADD ") {
            (vec![-2261], vec!["unique or primary key already exists"])
        } else if upper.contains(" DROP ") {
            (vec![-2442], vec!["nonexistent unique key"])
        } else {
            return None;
        }
    } else if upper.contains(" FOREIGN KEY") && upper.contains(" ADD ") {
        (vec![-2275], vec!["referential constraint already exists"])
    } else if upper.contains(" ADD ") || upper.contains(" ADD(") {
        (
            vec![-1430],
            vec!["column being added already exists in table"],
        )
    } else if upper.contains(" DROP ") {
        (
            vec![-904],
            vec!["invalid identifier (column does not exist)"],
        )
    } else {
        return None;
    };
    let conditions = codes
        .iter()
        .map(|code| format!("SQLCODE = {code}"))
        .collect::<Vec<_>>()
        .join(" OR ");
    let comments_text = codes
        .iter()
        .zip(comments.iter())
        .map(|(code, comment)| format!("ORA-{:05}: {comment}", -code))
        .collect::<Vec<_>>()
        .join("; ");
    Some(format!(
        "-- [AUTO-GENERATED] Idempotent wrapper for:\n{}\n-- Ignored Oracle errors: {comments_text}\nBEGIN\n    EXECUTE IMMEDIATE '{}';\nEXCEPTION\n    WHEN OTHERS THEN\n        IF {conditions} THEN\n            DBMS_OUTPUT.PUT_LINE('Warning: Operation already performed or invalid target, skipping.');\n            NULL;\n        ELSE\n            RAISE;\n        END IF;\nEND;\n/",
        comment_statement(statement),
        statement.replace("'", "''")
    ))
}

fn comment_statement(statement: &str) -> String {
    format!("-- {}", statement.replace('\n', "\n-- "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_supported_create_and_alter_statements() {
        let (output, modified) = try_make_idempotent(
            "CREATE INDEX IDX_T ON T (ID);\nALTER TABLE T ADD (NAME VARCHAR2(20))",
        );
        assert!(modified);
        assert!(output.contains("PRAGMA EXCEPTION_INIT(e_exists, -955)"));
        assert!(output.contains("SQLCODE = -1430"));
    }
}
