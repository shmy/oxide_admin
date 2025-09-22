FROM oven/bun:alpine AS frontend-build
WORKDIR /frontend
COPY frontend/ ./
RUN bun install --registry=https://mirrors.cloud.tencent.com/npm/
RUN bun run build

FROM rust:latest AS backend-build
WORKDIR /_
RUN apt-get update && apt-get install -y musl-tools build-essential
RUN rustup target add x86_64-unknown-linux-musl
COPY . /_
COPY --from=frontend-build "/frontend/dist" /_/frontend/dist
RUN cargo build --package server --release --target x86_64-unknown-linux-musl --locked

# https://github.com/GoogleContainerTools/distroless/blob/main/README.md#debian-12
FROM gcr.io/distroless/static-debian12:latest
WORKDIR /opt
COPY --from=backend-build /_/target/x86_64-unknown-linux-musl/release/server app
ENTRYPOINT ["./app", "serve"]
