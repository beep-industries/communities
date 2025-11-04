CREATE TABLE servers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    icon_asset_id UUID,
    banner_asset_id UUID,
    owner_user_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE server_members (
    id UUID PRIMARY KEY,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    nickname TEXT,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ,
    UNIQUE (server_id, user_id)
);

CREATE TYPE channel_visibility AS ENUM ('public', 'private');
CREATE TYPE channel_kind AS ENUM ('category', 'text', 'voice', 'stage', 'forum', 'announcement');

CREATE TABLE channels (
    id UUID PRIMARY KEY,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    parent_channel_id UUID REFERENCES channels(id) ON DELETE SET NULL,
    owner_member_id UUID REFERENCES server_members(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    topic TEXT,
    kind channel_kind NOT NULL,
    visibility channel_visibility NOT NULL DEFAULT 'public',
    position INTEGER NOT NULL DEFAULT 0,
    rate_limit_per_user_seconds INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT channels_parent_same_server CHECK (
        parent_channel_id IS NULL
        OR EXISTS (
            SELECT 1 FROM channels parent
            WHERE parent.id = parent_channel_id AND parent.server_id = server_id
        )
    ),
    CONSTRAINT channels_private_requires_owner CHECK (
        visibility = 'public' OR owner_member_id IS NOT NULL
    )
);

CREATE TABLE private_channel_members (
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    server_member_id UUID NOT NULL REFERENCES server_members(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (channel_id, server_member_id)
);

CREATE TABLE roles (
    id UUID PRIMARY KEY,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color INTEGER,
    position INTEGER NOT NULL DEFAULT 0,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_mentionable BOOLEAN NOT NULL DEFAULT FALSE,
    permissions BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE member_roles (
    server_member_id UUID NOT NULL REFERENCES server_members(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (server_member_id, role_id)
);

CREATE TYPE permission_override_target AS ENUM ('role', 'member');

CREATE TABLE permission_overrides (
    id UUID PRIMARY KEY,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    target_type permission_override_target NOT NULL,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    server_member_id UUID REFERENCES server_members(id) ON DELETE CASCADE,
    allow_permissions BIGINT NOT NULL DEFAULT 0,
    deny_permissions BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT permission_overrides_target CHECK (
        (target_type = 'role' AND role_id IS NOT NULL AND server_member_id IS NULL)
        OR (target_type = 'member' AND server_member_id IS NOT NULL AND role_id IS NULL)
    )
);

CREATE TABLE webhooks (
    id UUID PRIMARY KEY,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    created_by_member_id UUID REFERENCES server_members(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    avatar_asset_id UUID,
    token TEXT NOT NULL,
    url TEXT,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    UNIQUE (channel_id, name)
);

CREATE INDEX idx_channels_server_id ON channels (server_id);
CREATE INDEX idx_channels_parent_channel_id ON channels (parent_channel_id);
CREATE INDEX idx_private_channel_members_member ON private_channel_members (server_member_id);
CREATE INDEX idx_roles_server_id_position ON roles (server_id, position);
CREATE UNIQUE INDEX idx_roles_server_id_name_unique ON roles (server_id, LOWER(name));
CREATE INDEX idx_permission_overrides_channel_id ON permission_overrides (channel_id);
CREATE INDEX idx_permission_overrides_role_id ON permission_overrides (role_id);
CREATE INDEX idx_permission_overrides_member_id ON permission_overrides (server_member_id);
CREATE INDEX idx_webhooks_channel_id ON webhooks (channel_id);

ALTER TABLE servers
    ADD CONSTRAINT servers_owner_exists
    CHECK (owner_user_id IS NOT NULL);
