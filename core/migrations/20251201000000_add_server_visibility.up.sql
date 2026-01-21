-- Add up migration script here

-- Create the visibility enum type
CREATE TYPE server_visibility AS ENUM ('public', 'private');

-- Add visibility column to servers table
ALTER TABLE servers
ADD COLUMN visibility server_visibility NOT NULL DEFAULT 'public';
