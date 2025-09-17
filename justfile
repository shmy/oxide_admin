dev:
    cargo watch -q -c -x "run --package server -- serve" -w app -w crates -w bin

check:
    cargo watch -q -c -x "check --workspace" -w app -w crates -w bin

setup:
    sqlx migrate run --source app/infrastructure/migration/sql

sqlx_prepare:
    cargo clean
    cargo sqlx prepare -- --package server

build_frontend:
    #! /bin/sh
    set -e
    cd frontend
    # pnpm build
    pnpm build_brotli

build: build_frontend
    cargo build --package server --release --locked
    ls -lh target/release
    terminal-notifier -title "构建成功" -message "已完成"

build_linux_x86_64_musl: build_frontend sqlx_prepare
    cross build --package server --release --target x86_64-unknown-linux-musl --locked
    ls -lh target/x86_64-unknown-linux-musl/release
    terminal-notifier -title "构建成功" -message "x86_64-unknown-linux-musl 已完成"

build_windows_x86_64_msvc: build_frontend
    cargo-xwin build --package server --release --target x86_64-pc-windows-msvc --locked
    ls -lh target/x86_64-pc-windows-msvc/release
    terminal-notifier -title "构建成功" -message "x86_64-pc-windows-msvc 已完成"

build_container:
    docker buildx build --platform linux/amd64 -t server --pull .

sort:
    cargo sort app/**
    cargo sort crates/**
    cargo sort bin/**
    cargo sort