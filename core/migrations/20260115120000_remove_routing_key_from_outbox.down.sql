-- Down migration: add routing_key column back to outbox_messages table

ALTER TABLE outbox_messages ADD COLUMN routing_key VARCHAR(255) NOT NULL DEFAULT '';
