FROM oven/bun:alpine AS frontend-build
WORKDIR /frontend
COPY frontend/ ./
RUN bun install --registry=https://mirrors.cloud.tencent.com/npm/
RUN bun run build

FROM rust:alpine AS backend-build
WORKDIR /_
RUN rustup target add x86_64-unknown-linux-gnu
COPY . ./
COPY --from=frontend-build /frontend/dist /_/frontend/dist
RUN cargo build --package server --release --target x86_64-unknown-linux-gnu --locked

# https://github.com/GoogleContainerTools/distroless/blob/main/README.md#debian-12
FROM gcr.io/distroless/cc-debian12:latest
WORKDIR /opt
COPY --from=backend-build /_/target/x86_64-unknown-linux-gnu/release/server server
ENTRYPOINT ["/opt/server", "serve"]