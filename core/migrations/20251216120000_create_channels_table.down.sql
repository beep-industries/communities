-- Down migration: drop channels table

DROP TRIGGER IF EXISTS update_channels_updated_at ON channels;
DROP TABLE IF EXISTS channels;
DROP TYPE IF EXISTS channel_type;

