-- Create member_roles pivot table
CREATE TABLE member_roles (
    member_id UUID NOT NULL REFERENCES server_members(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    PRIMARY KEY (member_id, role_id),
    CONSTRAINT check_same_server CHECK (
        (SELECT server_id FROM server_members WHERE id = member_id) = 
        (SELECT server_id FROM roles WHERE id = role_id)
    )
);

