//! Pure transformation logic for Oracle SQL and SQL*Plus scripts.

mod encoding;
mod idempotent;
mod lexer;
mod processor;
mod schema;
mod script;
mod strings;
mod utils;

pub use encoding::{decode_gb18030, ensure_gb2312_memory};
pub use lexer::{Token, TokenKind, lex_sql};
pub use processor::process_sql_text;
pub use schema::{build_current_schema_directive, normalize_schema_name};
pub use script::{ScriptUnit, detect_statement_type, split_sql_plus_script_units};

/// Errors returned by the text processor.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid target schema: {0}")]
    InvalidSchema(String),
    #[error("failed to decode GB2312/GB18030 data: {0}")]
    Decode(String),
    #[error("failed to encode GB2312/GB18030 data: {0}")]
    Encode(String),
}
