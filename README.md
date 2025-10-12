> A Starter Template for Admin Panel Based on Rust and Amis.js/React

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/shmy/oxide_admin)
[![Build](https://github.com/shmy/oxide_admin/actions/workflows/build.yaml/badge.svg)](https://github.com/shmy/oxide_admin/actions/workflows/build.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/shmy/oxide_admin)](https://app.codecov.io/github/shmy/oxide_admin)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

English | [简体中文](./README_ZH_CN.md)

## 🎯 Project Goals
- Provide a quick starting point for building admin systems
- Use a modern Rust + Amis.js/React technology stack
- Follow Domain-Driven Design (DDD) and Clean Architecture principles

## 👀 Online Preview
> Since the free plan of `Render` is used, access may be slow. The instance will freeze after 15 minutes of inactivity, and subsequent access will go through `Render`’s interstitial page. Please be aware.

[https://oxide-admin.onrender.com/_](https://oxide-admin.onrender.com/_)  
> Please do not modify the password.

- Account: `admin`  
- Password: `123456`

## ✨ Features
- **DDD**: Separation of adapter (presentation), application services, domain models, and infrastructure layers.
- **CQRS**: Lightweight CQRS built-in, supporting read/write separation.
- **PostgreSQL**: Integrated `PostgreSQL`, based on `sqlx`.
- **Event Bus**: Built-in event system to decouple business logic via domain events.
- **Dependency Injection**: Supported by [`nject`](https://github.com/nicolascotton/nject).
- **Code Generation**: One-click generation of module code such as `CRUD`, `CommandHandler`, `QueryHandler`, etc.
- **Timezone Config**: Configurable for database and scheduled jobs.
- **Comprehensive coverage**: complete unit test/integration test;
- **API Docs**: Generated using [`utoipa`](https://github.com/juhaku/utoipa), available at [`/scalar`](https://oxide-admin.onrender.com/scalar), configurable to disable.
- **Authentication**: JWT-based with `refresh_token` and `access_token` issuance, validation, and refresh.
- **Authorization**: Built-in RBAC for flexible menu and API permission control.
- **DB Auto Migration**: No manual migrations required during deployment.
- **Rate Limiting Middleware**: Route-level rate limiting.
- **Captcha**: Prevent brute force and malicious requests.
- **Logging & Tracing**: Multiple logging options, supports [`OpenTelemetry`](https://opentelemetry.io/).
- **Built-in `single_flight` macro**: Reduce DB load.
- **File Upload & Access Signature**: APIs for single file, image, and chunked upload; supports `local FS` and `S3-compatible` storage.
- **KV Cache**: With TTL support, via `redis` or [`moka`](https://github.com/moka-rs/moka).
- **Background Tasks**: Single-node via `sqlite`, distributed via `postgres`.
- **Scheduled Task**: Supports embedded execution or separate execution.
- **Graceful Shutdown**: Properly terminates services and releases resources.
- **Multi-Source Config**: Supports env vars, `.env`, and CLI args.
- **Feature Flags**: Supports [`flipt`](https://github.com/flipt-io/flipt).
- **Github CI**: Auto build for `x86_64-unknown-linux-gnu`.
- **Docker Image**: Provides a `Dockerfile` for containerized deployment.

### 🎖️ Built-in Features
<table>
    <tr>
        <th>Feature</th>
        <th>Name</th>
        <th>Notes</th>
        <th>Enabled by Default</th>
    </tr>
    <tr>
        <td>Postgres tls</td>
        <td>postgres_tls</td>
        <td>Enable tls</td>
        <td></td>
    </tr>
    <tr>
        <td rowspan="3">KV Storage <b>(choose one only)</b></td>
        <td>kv_redb</td>
        <td>Use redb as kv/cache, suitable for monolithic projects</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>kv_redis</td>
        <td>Use redis as kv/cache, suitable for distributed projects</td>
        <td></td>
    </tr>
    <tr>
        <td>kv_redis_tls</td>
        <td>Use redis as kv/cache with TLS enabled</td>
        <td></td>
    </tr>
    <tr>
        <td rowspan="2">Background Tasks <b>(choose one only)</b></td>
        <td>bg_sqlite</td>
        <td>Use sqlite for background tasks, suitable for monolithic projects</td>
        <td></td>
    </tr>
    <tr>
        <td>bg_postgres</td>
        <td>Use postgres for background tasks, suitable for distributed projects</td>
        <td>✅</td>
    </tr>
    <tr>
        <td rowspan="2">Cache <b>(choose one only)</b></td>
        <td>cache_moka</td>
        <td>Use moka for cache, suitable for monolithic projects</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>cache_redis</td>
        <td>Use redis for cache, suitable for distributed projects</td>
        <td></td>
    </tr>
    <tr>
        <td>Scheduled Tasks</td>
        <td>serve_with_sched</td>
        <td>Embed scheduled tasks in web server process (single-node). When disabled, can run separately via `server sched` (distributed).</td>
        <td>✅</td>
    </tr>
    <tr>
        <td rowspan="3">Object Storage <b>(choose one only)</b></td>
        <td>object_storage_fs</td>
        <td>Use local filesystem</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>object_storage_s3</td>
        <td>Use S3-compatible service as object storage</td>
    </tr>
    <tr>
        <td>object_storage_s3_tls</td>
        <td>Use S3-compatible service with TLS enabled</td>
    </tr>
    <tr>
        <td rowspan="4">Logging & Trace <b>(multiple options allowed)</b></td>
        <td>trace_console</td>
        <td>Log output to console</td>
        <td>✅</td>
    </tr>
    <tr>
        <td>trace_rolling</td>
        <td>Rolling logs in JSON format</td>
        <td></td>
    </tr>
    <tr>
        <td>trace_otlp</td>
        <td>Integrate with OpenTelemetry, suitable for distributed projects</td>
        <td></td>
    </tr>
    <tr>
        <td>trace_otlp_tls</td>
        <td>Integrate with OpenTelemetry with TLS enabled</td>
        <td></td>
    </tr>
    <tr>
        <td>Feature Flags <b>(choose one only)</b></td>
        <td>flag_flipt</td>
        <td>Use flipt as feature flag, suitable for distributed projects</td>
        <td></td>
    </tr>
</table>

> Modify in `bin/server/Cargo.toml`.

## 🎈 Frontend
- **Architecture**: Powered by [`Amis.js`](https://github.com/baidu/amis) low-code with rich components for fast CRUD, extendable with React custom components.
- **Optimization**: Auto obfuscation, gzip compression (brotli optional) at build time.
- **Embedding**: Static assets embedded into binaries.

## ⚙️ Tech Stack
- **Backend**: Rust + Axum + Nject + SQLx + Postgres
- **Frontend**: Amis.js + React + TypeScript + Rsbuild
- **Tools**: just + Bun

## 📁 Project Structure
```txt
oxide_admin/
├── app/                    # Rust backend
│   ├── adapter/            # API layer (REST endpoints)
│   ├── application/        # Application layer (use cases/services)
│   ├── domain/             # Domain layer (entities/value objects)
│   ├── infrastructure/     # Infrastructure layer (technical details)
│         └── port/             # Domain implementations
│         └── migration/        # Database migrations
│         └── repository/       # Repository implementations
├── frontend/             # Frontend app
├── target/               # Build output
└── Cargo.toml            # Workspace config
```
> Strictly follows DDD design principles to ensure maintainability and scalability.

## 🛠️ Quick Start

> Please ensure that you have installed [Rust](https://www.rust-lang.org/tools/install) and [Bun](https://bun.com/docs/installation), as well as [just](https://just.systems/man/en/introduction.html).

### Clone the project and initialize
```bash
git clone git@github.com:shmy/oxide_admin.git
cd oxide_admin
# start a postgres
docker compose up -d
# setup env
cp .env.example .env
# install sqlx-cli & cargo-watch
cargo install sqlx-cli cargo-watch
# setup sqlx migration
just setup
```

### Run Backend
```base
just dev
```
> The backend listens on `127.0.0.1:8080` by default, and the frontend will have a `dev server` to proxy; 

### Run Frontend
```base
cd frontend
bun install
bun run dev
```
> Access `http://127.0.0.1:3000/_`

## 📦 Build Commands
- Local Architecture:
```bash
just build
```
- Cross Compilation: `Linux/x86_64-unknown-linux-musl`
> Ensure that `cross` is installed, use `cargo install cross` to install.
```bash
just build_linux_x86_64_musl
```
- Cross Compilation: `Windows/x86_64-pc-windows-msvc`
> Ensure that `xwin` is installed, use `cargo install cargo-xwin` to install.
```bash
just build_windows_x86_64_msvc
```
- Build Docker image
```bash
just build_image
```

## 🉑 Test
> Install the following tools
```bash
cargo install cargo-llvm-cov
cargo install cargo-nextest
cargo install hurl
```
### Run
```bash
just test
```

### Generate Coverage Report
```bash
just test_coverage
```

## 📃 Code Generation
```bash
cargo g scaffold -h
```

### More
```bash
cargo g -h
```