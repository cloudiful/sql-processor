FROM debian:trixie-slim AS runtime

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates tini \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# CI prepares ci-image-input/ (binary + frontend-dist), see .github/workflows/docker-publish.yml
COPY --chmod=755 ci-image-input/sql-processor /usr/local/bin/sql-processor
COPY ci-image-input/frontend-dist /app/public

EXPOSE 8080
ENTRYPOINT ["tini", "--", "/usr/local/bin/sql-processor", "--serve", "--addr", ":8080", "--static-dir", "/app/public"]
