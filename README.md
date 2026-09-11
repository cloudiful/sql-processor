# SQL Processor

[简体中文](README.zh-CN.md)

SQL Processor is a Rust CLI and web application for preparing Oracle SQL and SQL*Plus deployment scripts.

## Features

- Adds `CURRENT_SCHEMA` session setup without rewriting object names.
- Wraps supported Oracle DDL in idempotent PL/SQL blocks.
- Adds missing SQL*Plus slash terminators and DML commits.
- Handles UTF-8 and GB18030/GB2312 input and output.
- Optionally validates original and processed SQL against Oracle using parse-only `DBMS_SQL` calls.
- Serves the Nuxt static web interface and JSON API from one Rust process.

## CLI

```bash
cargo run -p sql-processor -- --schema MY_SCHEMA --input input.sql --output output.sql
cat input.sql | cargo run -p sql-processor -- --schema MY_SCHEMA
```

Generate the API contract from the Rust routes:

```bash
cargo run -p sql-processor -- openapi export --output openapi/openapi.json
```

## Web Application

Build the static Nuxt application and start the Rust server:

```bash
cargo run -p sql-processor -- openapi export --output openapi/openapi.json
cd frontend
bun install
bun run generate:types
bun run build
cd ..
cargo run -p sql-processor -- --serve --addr :8080 --static-dir frontend/.output/public
```

Open `http://127.0.0.1:8080`. During frontend development, run `bun run dev` in `frontend`; its `/api` requests proxy to the Rust server at `http://127.0.0.1:8080`.

The API exposes `GET /api/health`, `POST /api/process`, and `GET /openapi.json`. Request bodies are limited to 4 MiB. Oracle passwords are kept in browser memory and are not persisted in `localStorage`.

## Container

```bash
docker build -t sql-processor-web .
docker run --rm -p 8080:8080 sql-processor-web
```

The image builds the Nuxt static output, compiles the Rust service, and serves both from port `8080`.
