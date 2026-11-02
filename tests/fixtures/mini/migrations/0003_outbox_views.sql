-- Rows still waiting to be relayed, as the dashboards read them.
CREATE OR REPLACE VIEW cw_outbox.pending_outbox AS
SELECT id, payload, created_at
FROM cw_outbox.outbox
WHERE sent_at IS NULL;
