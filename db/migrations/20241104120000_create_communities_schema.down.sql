DROP INDEX IF EXISTS idx_webhooks_channel_id;
DROP TABLE IF EXISTS webhooks;

DROP INDEX IF EXISTS idx_permission_overrides_member_id;
DROP INDEX IF EXISTS idx_permission_overrides_role_id;
DROP INDEX IF EXISTS idx_permission_overrides_channel_id;
DROP TABLE IF EXISTS permission_overrides;
DROP TYPE IF EXISTS permission_override_target;

DROP TABLE IF EXISTS member_roles;

DROP INDEX IF EXISTS idx_roles_server_id_name_unique;
DROP INDEX IF EXISTS idx_roles_server_id_position;
DROP TABLE IF EXISTS roles;

DROP INDEX IF EXISTS idx_private_channel_members_member;
DROP TABLE IF EXISTS private_channel_members;

DROP INDEX IF EXISTS idx_channels_parent_channel_id;
DROP INDEX IF EXISTS idx_channels_server_id;
DROP TABLE IF EXISTS channels;
DROP TYPE IF EXISTS channel_kind;
DROP TYPE IF EXISTS channel_visibility;

DROP TABLE IF EXISTS server_members;

ALTER TABLE IF EXISTS servers DROP CONSTRAINT IF EXISTS servers_owner_exists;
DROP TABLE IF EXISTS servers;
