-- Add up migration script here

-- Create the server_invitation_status enum type
CREATE TYPE server_invitation_status AS ENUM ('pending', 'accepted', 'rejected', 'expired');

-- Create the server_invitations table
CREATE TABLE server_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    inviter_id UUID NOT NULL,
    invitee_id UUID,
    status server_invitation_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

-- Create trigger to automatically update updated_at on UPDATE
CREATE TRIGGER update_server_invitations_updated_at
    BEFORE UPDATE ON server_invitations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
