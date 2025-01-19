FROM rust:slim-bookworm AS builder
WORKDIR /app

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/imphnen-chat-backend /tmp/imphnen-chat-backend

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /tmp/imphnen-chat-backend ./imphnen-chat-backend

CMD ["./imphnen-chat-backend"]
