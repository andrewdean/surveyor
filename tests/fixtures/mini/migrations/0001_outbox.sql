CREATE SCHEMA cw_outbox;

CREATE TABLE cw_outbox.outbox (
    id uuid PRIMARY KEY,
    payload jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    sent_at timestamptz
);

CREATE INDEX outbox_pending_idx ON cw_outbox.outbox (created_at) WHERE sent_at IS NULL;
