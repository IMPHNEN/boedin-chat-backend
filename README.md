# IMPHNEN Chat Backend

A WebSocket-based chat server built with Rust.

## READ THIS!

**Create New Branch** for Features/Testing and then you can merge it to the ***main*** branch after verified.

## API Endpoints

### WebSocket Connection

- **Protocol**: WebSocket
- **URL**: `/ws/`

## Installation

1. Clone the repository
2. Build the project:

```sh
cargo build --release
```

## Running the Server

```sh
cargo run --release
```

The server will start on `0.0.0.0:8080` by default.

## Prerequisites

- **Rust** - You can get from [here](https://www.rust-lang.org/tools/install)
- Cargo Dependencies in `Cargo.toml`:

```
axum
chrono
futures
serde
serde_json
tokio
tracing
tracing-subscriber
```

- Cargo Dependencies Features:

```
axum/ws
chrono/serde
serde/derive
tokio/full
```

## Project Structure

```
src/
  ├── routes/
  │   ├── chat.rs
  │   └── mod.rs
  └── main.rs
  └── models.rs
  └── state.rs
```

## Contributing

1. Fork the repository
2. Create your **feature** branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request
