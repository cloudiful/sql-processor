FROM oven/bun:1.3.5 AS frontend-builder

WORKDIR /src

COPY frontend/package.json ./frontend/
COPY frontend/nuxt.config.ts frontend/tsconfig.json ./frontend/
COPY openapi/openapi.json ./openapi/openapi.json
COPY frontend/src ./frontend/src

RUN cd frontend \
    && bun install \
    && bun run generate:types \
    && bun run build

FROM rust:1.88-bookworm AS rust-builder

WORKDIR /src

COPY Cargo.toml ./
COPY crates ./crates

RUN --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    --mount=type=cache,target=/root/.cargo/git,sharing=locked \
    --mount=type=cache,target=/src/target,sharing=locked \
    cargo build --release -p sql-processor

COPY --from=frontend-builder /src/frontend/.output/public ./public

FROM debian:trixie-slim

WORKDIR /app

COPY docker/debian.sources /etc/apt/sources.list.d/debian.sources
RUN rm -f /etc/apt/sources.list.d/debian.sources.bak /etc/apt/sources.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /src/target/release/sql-processor /usr/local/bin/sql-processor
COPY --from=rust-builder /src/public /app/public

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/sql-processor", "--serve", "--addr", ":8080", "--static-dir", "/app/public"]
