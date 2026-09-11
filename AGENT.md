# AGENT.md - SQL Processor

## Project Overview

`sql-processor` is a Rust workspace for transforming Oracle SQL and SQL*Plus scripts. Rust (Axum HTTP API + Nuxt static SPA) is the only implementation and the API source of truth.

## Workspace Layout

- `crates/sql-processor-core`: lexer, SQL*Plus splitting, DDL idempotency, schema directives, terminator/commit injection, string rewriting, and GB18030 handling.
- `crates/sql-processor-validation`: Oracle parse-only validation, CLOB binding, timeout handling, and ORA issue extraction.
- `crates/sql-processor-api`: Axum routes, DTOs, error responses, OpenAPI annotations, and static asset serving.
- `crates/sql-processor-cli`: compatible file/stdin/stdout CLI plus the OpenAPI export command.
- `frontend`: Nuxt static SPA with Monaco, Nuxt UI, Vue I18n, and the generated OpenAPI client types.
- `openapi/openapi.json`: generated API contract. Never edit it by hand.

## Common Operations

Generate the OpenAPI document from Rust routes:

```bash
cargo run -p sql-processor -- openapi export --output openapi/openapi.json
```

Generate frontend types and build the static SPA:

```bash
cd frontend
bun install
bun run generate:types
bun run typecheck
bun test
bun run build
```

Run Rust checks:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run the integrated server after building the frontend:

```bash
cargo run -p sql-processor -- --serve --addr :8080 --static-dir frontend/.output/public
```

## Contract Rules

- Keep `GET /api/health`, `POST /api/process`, and the existing camelCase JSON fields compatible unless a deliberate versioned change is agreed.
- Update Rust DTOs and `utoipa` route annotations first, regenerate `openapi/openapi.json`, then regenerate `frontend/src/lib/api-types.ts`.
- Do not duplicate request/response models in frontend code.
- Request bodies are limited at the Axum layer. Do not log request bodies or Oracle credentials.
- Oracle passwords may be sent in a request-scoped config but must never be persisted in browser storage.

## Code Guidelines

- Keep top-level API crates focused on transport, routing, orchestration, and composition.
- Keep business logic and domain-specific persistence in the owning capability crate.
- Keep non-generated files cohesive and below the repository's 400-line hard cap; extract a responsibility before crossing the 300-line planning threshold.
- Add focused tests for transformation boundaries, API contracts, security behavior, and Oracle error handling.
- Do not introduce PostgreSQL, SQLx, or persistent application storage without an explicit requirement.
