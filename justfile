dev:
    cargo watch -q -c -x "run --package server -- serve" -w app -w crates -w bin

check:
    cargo watch -q -c -x "check --workspace" -w app -w crates -w bin

setup:
    sqlx migrate run --source app/infrastructure/migration/sql


integration_test_watch:
    cargo watch -q -c -x "nextest run --package server --no-default-features --features server/test" -w app -w crates -w bin

test_watch:
    cargo watch -q -c -x "nextest run --workspace --no-default-features --features server/test" -w app -w crates -w bin

test_coverage:
    cargo llvm-cov nextest --workspace --no-default-features --features server/test --ignore-filename-regex 'bin/|src/'  --html

test_coverage_ci:
    cargo llvm-cov nextest --workspace --no-default-features --features server/test --ignore-filename-regex 'bin/|src/'  --lcov > lcov.info

sqlx_prepare:
    cargo clean
    cargo sqlx prepare -- --package server

sqlx_add name:
    sqlx migrate add {{name}} --source app/infrastructure/migration/sql

build_frontend:
    #! /bin/sh
    set -e
    cd frontend
    bun run build

build: build_frontend
    cargo build --package server --release --locked
    ls -lh target/release
    terminal-notifier -title "构建成功" -message "已完成"

build_linux_x86_64_musl: build_frontend sqlx_prepare
    cross build --package server --release --target x86_64-unknown-linux-musl --locked
    ls -lh target/x86_64-unknown-linux-musl/release
    terminal-notifier -title "构建成功" -message "x86_64-unknown-linux-musl 已完成"

build_linux_x86_64_musl_ci: build_frontend
    cargo build --package server --release --target x86_64-unknown-linux-musl --locked
    ls -lh target/x86_64-unknown-linux-musl/release

build_windows_x86_64_msvc: build_frontend
    cargo-xwin build --package server --release --target x86_64-pc-windows-msvc --locked
    ls -lh target/x86_64-pc-windows-msvc/release
    terminal-notifier -title "构建成功" -message "x86_64-pc-windows-msvc 已完成"

build_image:
    docker buildx build --platform linux/amd64 -t server --pull .

pre_commit:
    cargo sort app/**
    cargo sort crates/**
    cargo sort bin/**
    cargo sort
    cargo fmt --all
    cargo clippy --workspace