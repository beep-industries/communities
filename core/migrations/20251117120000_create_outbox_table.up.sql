-- Up migration: create outbox table, function, and trigger

-- Ensure extension for gen_random_uuid if not already present
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Create outbox_messages table
CREATE TABLE IF NOT EXISTS outbox_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange_name VARCHAR(255) NOT NULL,
    routing_key VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'READY',
    failed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for dispatcher queries (status + created_at for efficient polling)
CREATE INDEX IF NOT EXISTS idx_outbox_messages_status_created_at 
    ON outbox_messages(status, created_at);

-- Function to notify listeners when a message becomes READY
CREATE OR REPLACE FUNCTION notify_outbox_message() RETURNS TRIGGER AS $$
BEGIN
    -- Notify when status is READY (case-insensitive)
    IF LOWER(NEW.status) = 'ready' THEN
        PERFORM pg_notify(
            'outbox_channel',
            json_build_object(
                'operation', TG_OP,
                'table', TG_TABLE_NAME,
                'data', row_to_json(NEW)
            )::text
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger on insert/update to notify listeners
CREATE TRIGGER outbox_notify_trigger
    AFTER INSERT OR UPDATE ON outbox_messages
    FOR EACH ROW EXECUTE PROCEDURE notify_outbox_message();
