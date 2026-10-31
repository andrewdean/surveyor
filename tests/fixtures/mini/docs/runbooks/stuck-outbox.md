# Stuck outbox

When `outbox_pending_idx` grows, check the relay in `crates/outbox/src/relay.rs` and run `scripts/drain.sh`.
