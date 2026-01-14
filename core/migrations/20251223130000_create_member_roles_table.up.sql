-- Create member_roles pivot table
CREATE TABLE member_roles (
    member_id UUID NOT NULL REFERENCES server_members(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    PRIMARY KEY (member_id, role_id)
);

-- Function to check that member and role belong to the same server
CREATE OR REPLACE FUNCTION check_member_role_same_server()
RETURNS TRIGGER AS $$
DECLARE
    member_server_id UUID;
    role_server_id UUID;
BEGIN
    SELECT server_id INTO member_server_id FROM server_members WHERE id = NEW.member_id;
    SELECT server_id INTO role_server_id FROM roles WHERE id = NEW.role_id;
    
    IF member_server_id != role_server_id THEN
        RAISE EXCEPTION 'Member and role must belong to the same server';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to enforce same server constraint
CREATE TRIGGER member_role_same_server_check
    BEFORE INSERT OR UPDATE ON member_roles
    FOR EACH ROW
    EXECUTE FUNCTION check_member_role_same_server();

