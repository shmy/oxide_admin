FROM alpine:latest AS tini-builder
RUN apk add --no-cache tini

FROM oven/bun:alpine AS frontend-build
WORKDIR /frontend
COPY frontend/ ./
RUN bun install --registry=https://mirrors.cloud.tencent.com/npm/
RUN bun run build

FROM rust:alpine AS backend-build
WORKDIR /_
RUN apk add --no-cache musl-dev build-base
RUN rustup target add x86_64-unknown-linux-musl
COPY . /_
COPY --from=frontend-build "/frontend/dist" /_/frontend/dist
RUN cargo build --package server --release --target x86_64-unknown-linux-musl --locked

# https://github.com/GoogleContainerTools/distroless/blob/main/README.md#debian-12
FROM gcr.io/distroless/static-debian12:latest
WORKDIR /opt
COPY --from=backend-build /_/target/x86_64-unknown-linux-musl/release/server app
COPY --from=tini-builder /sbin/tini /tini
ENTRYPOINT ["/tini", "--"]
CMD ["/opt/app", "serve"]