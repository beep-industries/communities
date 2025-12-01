-- Add down migration script here

-- Remove visibility column from servers table
ALTER TABLE servers
DROP COLUMN visibility;

-- Drop the visibility enum type
DROP TYPE server_visibility;
