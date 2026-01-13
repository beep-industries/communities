-- Create member_roles pivot table
CREATE TABLE member_roles (
    member_id UUID NOT NULL REFERENCES server_members(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    PRIMARY KEY (member_id, role_id)
);

