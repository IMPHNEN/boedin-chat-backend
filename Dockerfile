FROM lukemathwalker/cargo-chef:latest-rust-1.83.0-alpine3.21 AS chef
WORKDIR /app
RUN apk update && apk add --no-cache lld clang

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM rust:slim-bookworm

WORKDIR /app
COPY --from=builder /app/target/release/imphnen-chat-backend ./imphnen-chat-backend

CMD ["./imphnen-chat-backend"]
