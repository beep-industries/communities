-- Up migration: remove routing_key column from outbox_messages table

ALTER TABLE outbox_messages DROP COLUMN IF EXISTS routing_key;
