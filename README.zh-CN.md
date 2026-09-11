# SQL Processor

[English](README.md)

SQL Processor 是一个 Rust CLI 和 Web 应用，用于准备 Oracle SQL 与 SQL*Plus 部署脚本。

## 功能

- 添加 `CURRENT_SCHEMA` 会话设置，不改写对象名称。
- 将支持的 Oracle DDL 包装为幂等 PL/SQL 块。
- 自动补充缺失的 SQL*Plus slash 终止符和 DML commit。
- 支持 UTF-8 与 GB18030/GB2312 输入输出。
- 可选连接 Oracle，通过 parse-only `DBMS_SQL` 校验原始 SQL 和处理后 SQL。
- 由同一个 Rust 进程提供 Nuxt 静态 Web 界面和 JSON API。

## CLI

```bash
cargo run -p sql-processor -- --schema MY_SCHEMA --input input.sql --output output.sql
cat input.sql | cargo run -p sql-processor -- --schema MY_SCHEMA
```

从 Rust 路由生成 API 契约：

```bash
cargo run -p sql-processor -- openapi export --output openapi/openapi.json
```

## Web 应用

构建 Nuxt 静态应用并启动 Rust 服务：

```bash
cargo run -p sql-processor -- openapi export --output openapi/openapi.json
cd frontend
bun install
bun run generate:types
bun run build
cd ..
cargo run -p sql-processor -- --serve --addr :8080 --static-dir frontend/.output/public
```

打开 `http://127.0.0.1:8080`。前端开发时在 `frontend` 目录运行 `bun run dev`，`/api` 请求会代理到 `http://127.0.0.1:8080` 的 Rust 服务。

API 提供 `GET /api/health`、`POST /api/process` 和 `GET /openapi.json`。请求体限制为 4 MiB。Oracle 密码只保存在浏览器内存中，不会写入 `localStorage`。

## 容器

```bash
docker build -t sql-processor-web .
docker run --rm -p 8080:8080 sql-processor-web
```

镜像会构建 Nuxt 静态产物、编译 Rust 服务，并通过 `8080` 端口统一提供服务。
