-- Down migration: drop trigger, function, and table

DROP TRIGGER IF EXISTS outbox_notify_trigger ON outbox_messages;
DROP FUNCTION IF EXISTS notify_outbox_message();
DROP INDEX IF EXISTS idx_outbox_messages_status_created_at;
DROP TABLE IF EXISTS outbox_messages;
