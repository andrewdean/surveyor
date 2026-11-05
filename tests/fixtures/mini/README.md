# mini

A fixture repository: an outbox relay crate, an API binary that reports the backlog,
the migrations that create the outbox and users tables, and a web client.

## Layout

- `crates/outbox` holds the relay and the store.
- `crates/api/src/main.rs` reports the backlog and the users.

## Development database

Run `mise run db:migrate` before starting the API.
