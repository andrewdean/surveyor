-- Rows still waiting to be relayed, as the dashboards read them.
CREATE OR REPLACE VIEW cw_outbox.pending_outbox AS
SELECT id, payload, created_at
FROM cw_outbox.outbox
WHERE sent_at IS NULL;

-- The backlog gauge lives in whichever schema the deployment names; the migration
-- runner passes it to format(), so the view's schema is a run-time argument.
DO $$
DECLARE
    target_schema TEXT := current_schema();
BEGIN
    EXECUTE format($view$
        CREATE OR REPLACE VIEW %I.outbox_backlog_v1 AS
        SELECT count(*) AS waiting FROM %I.outbox WHERE sent_at IS NULL
    $view$, target_schema, target_schema);
END $$;
