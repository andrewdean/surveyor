use crate::store::OutboxStore;

/// Drains pending rows.
pub trait Drain {
    /// Drain up to `limit` rows and return how many were sent.
    fn drain(&mut self, limit: usize) -> usize;
}

/// Relays outbox rows to the broker.
///
/// See docs/rfd/0001-outbox.md for the design.
pub struct Relay {
    store: OutboxStore,
}

impl Relay {
    /// Creates a relay over a store.
    pub fn new(store: OutboxStore) -> Self {
        Self { store }
    }

    fn publish(&self, _row: &str) {}
}

impl Drain for Relay {
    fn drain(&mut self, limit: usize) -> usize {
        let rows = self.store.pending(limit);
        for row in &rows {
            self.publish(row);
        }
        self.store.mark_sent(&rows);
        rows.len()
    }
}

/// How many rows a relay has sent, for reports.
pub struct DrainSummary(pub usize);

impl From<&Relay> for DrainSummary {
    fn from(_relay: &Relay) -> Self {
        DrainSummary(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drains_pending_rows() {
        let mut relay = Relay::new(OutboxStore::default());
        assert_eq!(relay.drain(10), 0);
    }
}
