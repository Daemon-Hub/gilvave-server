# Gilvave Server — Agent Instructions

## Quick Commands

```bash
# Build entire workspace
cargo build

# Build specific service
cargo build -p gilvave-http-api
cargo build -p gilvave-gateway

# Run services (separate terminals)
cargo run -p gilvave-http-api    # REST API on :3000
cargo run -p gilvave-gateway     # WebSocket on :3100/ws

# Run S3 integration test (requires test.png in crates/s3/tests/)
cargo test -p gilvave-s3

# Check for compile errors without building
cargo check --workspace

# Format
cargo fmt

# Lint
cargo clippy --workspace
```

## Architecture

Two binary services, five library crates:

| Member | Role |
|--------|------|
| `services/http-api` | REST endpoints (users, servers, channels). Axum on `:3000` |
| `services/gateway` | WebSocket relay. Axum on `:3100/ws` |
| `crates/core` | Domain types: models, DTOs, error enum, ID newtypes |
| `crates/infra` | DB layer, JWT, auth middleware, service structs |
| `crates/settings` | Compile-time env config via `settings!()` macro |
| `crates/messaging` | RabbitMQ client (lapin). Fanout exchange `"messages"` |
| `crates/s3` | S3/SeaweedFS client (buckets: `static`, `images`, `videos`, `files`) |

Dependency flow: `core ← infra ← services`, `settings` and `messaging` used by all.

## Critical: Settings Are Baked at Compile Time

`crates/settings/build.rs` reads `.env` and emits `cargo:rustc-env` directives. All config values are `&'static str` via `std::env!()`.

**If you change `.env`, you MUST rebuild** (`cargo build`) — runtime env changes are ignored.

Access config with: `gilvave_settings::settings!()` or the `settings!()` macro (imported via `use gilvave_settings::settings;`).

## Database

- Postgres via SQLx. Migrations in `migrations/` run automatically on startup (`sqlx::migrate!`).
- No `.sqlx` offline cache — requires live DB for `cargo check`/`cargo build` if using compile-time query checking.
- Connection string: `DATABASE_URL` in `.env` (currently points to `213.108.39.127:5432`).

## Event System (Gateway)

WebSocket events use tagged JSON: `{"op": "EventName", "d": {...}}`.

- `ClientEvent` — client → server (defined in `services/gateway/src/events/client.rs`)
- `ServerEvent` — server → client (defined in `services/gateway/src/events/server.rs`)
- `BrokerEvent` — RabbitMQ → gateway (defined in `services/gateway/src/events/broker.rs`)

New event types: add variant to the enum, implement `EventHandler` trait, register in `dispatch_event!` macro in `ws.rs`.

## Error Handling

Use `CoreError` enum from `crates/core/src/error.rs` — maps to HTTP status codes with JSON `{"error": "message"}`.

Services return `anyhow::Result<T>`. Convert with `.map_err(CoreError::from)?` or let `From<anyhow::Error>` handle it.

## Conventions

- Russian comments throughout — maintain consistency.
- Services use `mimalloc` as global allocator.
- `AppState` structs in each service hold service instances and are `Clone` (wrapped in `Arc` where needed).
- New services go in `crates/infra/src/service/` with a `mod.rs` re-export.
- New models in `crates/core/src/model/`, DTOs in `crates/core/src/dto/`.
- IDs are newtype wrappers (`UserId`, `ChannelId`, etc.) in `crates/core/src/ids/`.

## Infrastructure

Docker Compose provides Redis and RabbitMQ. Postgres is external (commented out in compose).

```bash
docker compose up -d    # Start Redis + RabbitMQ
```
