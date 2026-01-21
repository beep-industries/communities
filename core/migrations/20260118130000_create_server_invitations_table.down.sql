-- Add down migration script here

-- Drop the trigger first
DROP TRIGGER IF EXISTS update_server_invitations_updated_at ON server_invitations;

-- Drop the table
DROP TABLE IF EXISTS server_invitations;

-- Drop the enum type
DROP TYPE IF EXISTS server_invitation_status;
