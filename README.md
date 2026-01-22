# SQL Processor

A Go library and CLI tool for processing and transforming SQL scripts, specifically tailored for Oracle database deployments. This project was extracted from `db_publisher` to provide a standalone, reusable package for SQL manipulation.

## Features

- **Schema Injection**: Automatically prefixes target schema names to object names in DDL statements (`CREATE`, `DROP`, `ALTER`, `TRUNCATE`).
- **Idempotency Wrapping**: Wraps DDL statements (like `CREATE TABLE` or `DROP INDEX`) in PL/SQL blocks to handle "already exists" or "does not exist" errors, making scripts safe for multiple executions.
- **Automatic COMMIT/Slash Injection**:
    - Detects missing `COMMIT` statements in scripts containing DML.
    - Automatically adds trailing slashes (`/`) for PL/SQL blocks.
- **Encoding Management**: Handles conversion between UTF-8 and GB2312 (GB18030) to support legacy environment requirements.
- **SQL Lexer**: A custom SQL lexer for accurate tokenization and transformation.

## Project Structure

- `pkg/lexer`: SQL tokenization logic.
- `pkg/processor`: Core SQL processing logic, including:
    - `schema.go`: Schema injection rules.
    - `idempotent.go`: PL/SQL wrapping logic.
    - `encoding.go`: Encoding conversion utilities.
    - `processor.go`: The main entry point for processing SQL text.

## Usage

### As a CLI Tool

#### Installation

```bash
go install ./cmd/sql-processor
```

#### Usage Examples

```bash
# Basic usage with schema injection
sql-processor -schema MY_SCHEMA -input test.sql -output out.sql

# Using pipes
cat input.sql | sql-processor -schema MY_SCHEMA > output.sql

# Enable GB2312/GB18030 handling for legacy files
sql-processor -gb2312 -schema MY_SCHEMA -input legacy.sql -output updated.sql
```

### As a Library

```go
import "sql-processor/pkg/processor"

func main() {
    sql := "CREATE TABLE MY_TABLE (ID NUMBER);"
    targetSchema := "MY_SCHEMA"
    
    processedSQL, messages, err := processor.ProcessSQLText(sql, targetSchema)
    if err != nil {
        panic(err)
    }
    
    fmt.Println(processedSQL)
}
```

## Testing

Run tests using the standard Go toolchain:

```bash
go test ./...
```

### Coverage

The project maintains a high test coverage. You can check the coverage with:

```bash
go test -cover ./...
```

Current coverage status:
- `pkg/lexer`: 100.0%
- `pkg/processor`: ~95.0%
- `cmd/sql-processor`: ~82.0%
