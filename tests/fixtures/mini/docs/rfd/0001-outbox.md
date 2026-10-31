---
uid: RFD-0001
state: published
---
# Outbox relay

Events leave the system only through the relay in `crates/outbox/src/relay.rs`, which reads
`cw_outbox.outbox` and publishes in order. The API crate (`crates/api`) must never publish directly.

## Invariant

Nothing outside crates/outbox/ writes to the outbox table.
