# AGENT.md - SQL Processor

## Project Overview
`sql-processor` is a Go-based library and CLI tool designed to parse, analyze, and transform SQL scripts (primarily Oracle). It provides features such as schema injection, idempotency wrapping for DDL, and automatic injection of `COMMIT` or `/` (slash) markers.

This project was extracted from the `db_publisher` project to serve as a standalone, reusable module.

## Core Components and File Paths

### 1. Lexer (`pkg/lexer`)
Provides low-level tokenization of SQL strings.
- `sql-processor/pkg/lexer/lexer.go`: Implementation of the SQL lexer, token types, and keyword definitions.
- `sql-processor/pkg/lexer/lexer_test.go`: Comprehensive tests for SQL tokenization.

### 2. Processor (`pkg/processor`)
Contains the high-level logic for SQL transformation.
- `sql-processor/pkg/processor/processor.go`: Main entry point (`ProcessSQLText`) that coordinates schema injection, idempotency, and commit/slash injection.
- `sql-processor/pkg/processor/processor_test.go`: Integration tests for the full processing pipeline.
- `sql-processor/pkg/processor/schema.go`: Logic for automatically prefixing object names with a target schema in DDL statements.
- `sql-processor/pkg/processor/schema_test.go`: Tests for schema injection logic and DML warning detection.
- `sql-processor/pkg/processor/idempotent.go`: Wraps `CREATE` and `DROP` statements in PL/SQL blocks to make them idempotent.
- `sql-processor/pkg/processor/idempotent_test.go`: Tests for idempotency wrapping.
- `sql-processor/pkg/processor/encoding.go`: Utilities for handling UTF-8 and GB2312/GB18030 encoding conversions.
- `sql-processor/pkg/processor/encoding_test.go`: Tests for encoding detection and conversion.
- `sql-processor/pkg/processor/utils.go`: Helper functions for comment removal, PL/SQL detection, and SQL statement splitting.
- `sql-processor/pkg/processor/utils_test.go`: Tests for utility functions including comment removal and PL/SQL detection.

### 3. CLI (`cmd/sql-processor`)
Provides a command-line interface for the processing logic.
- `sql-processor/cmd/sql-processor/main.go`: CLI implementation using flags for schema injection, file I/O, and encoding options.
- `sql-processor/cmd/sql-processor/main_test.go`: Integration tests for the CLI tool.

## Key Data Structures
- `lexer.Token`: Represents a single SQL token (Keyword, Identifier, String, etc.) with its value and position.
- `processor.SQLFileData`: Represents processed SQL content in memory with its associated path.

## Common Operations
- **Process SQL Text**: Use `processor.ProcessSQLText(content, targetSchema)` to transform a raw SQL string.
- **Handle File Encodings**: Use `processor.EnsureGB2312Memory` and `processor.ProcessGB2312SQLBytes` when working with legacy GB2312 files.
- **Run CLI**: Use `sql-processor -schema SCHEMA_NAME -input input.sql -output output.sql` after installing via `go install ./cmd/sql-processor`.
- **Run Tests**: Use `go test -cover ./...` to run all tests and check coverage.
- **CI/CD**: Gitea Actions workflow is located in `.gitea/workflows/build.yaml`. It automatically builds binaries for Windows, Linux, and macOS using `PROJECT_NAME` (from repository name) and `MAIN_PATH` environment variables.

## Guidelines for Modification
- **Incremental Updates**: Prefer small, focused changes to existing logic.
- **Test Driven**: Always update or add corresponding tests in `*_test.go` files when modifying processing logic.
- **Maintain Lexer Accuracy**: Ensure any new keywords or symbols are added to the lexer if they affect parsing or injection logic.