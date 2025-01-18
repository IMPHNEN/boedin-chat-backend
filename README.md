# IMPHNEN Chat Backend

A WebSocket-based chat server built with Rust and Actix-web framework.

## TODO

- Enhance error logging.
- ~~Enhance the performance.~~

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

The server will start on `127.0.0.1:8080` by default.

## Prerequisites

- Rust & Cargo (Rust Package Manager)
- Cargo Dependencies in `Cargo.toml`:

```
actix-web
actix-ws
chrono
serde
serde_json
tokio
uuid
```

- Cargo Dependencies Features:

```
chrono/serde
serde/derive
tokio/full
uuid/v4
```

## Project Structure

```
src/
  ├── handlers/
  │   ├── chat.rs
  │   └── mod.rs
  ├── routes/
  │   ├── mod.rs
  │   └── ws.rs
  └── main.rs
```

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request
