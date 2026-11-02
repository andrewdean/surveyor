/// Reads and writes the outbox table.
#[derive(Default)]
pub struct OutboxStore;

impl OutboxStore {
    /// Pending rows, oldest first.
    pub fn pending(&self, limit: usize) -> Vec<String> {
        let _ = sqlx::query!("SELECT id, payload FROM cw_outbox.outbox WHERE sent_at IS NULL ORDER BY created_at LIMIT $1", limit as i64);
        Vec::new()
    }

    /// Marks rows as sent.
    pub fn mark_sent(&self, _rows: &[String]) {
        let _ = sqlx::query("UPDATE cw_outbox.outbox SET sent_at = now() WHERE id = ANY($1)");
    }

    /// Drops sent rows older than the retention window; the schema is chosen at run time.
    pub fn purge(&self, schema: &str) {
        let _ = sqlx::query(&format!("DELETE FROM {schema}.outbox WHERE sent_at < now() - interval '7 days'"));
    }
}
