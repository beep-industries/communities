# Communities service

The Communities service is designed to facilitate the creation, management, and interaction of user communities within the platform.
It will handle:

- Servers
- Members
- Roles
- Channels

## Database migrations

- Migrations live under `db/migrations` and are embedded at compile time via `sqlx::migrate!`.
- Run them locally with `cargo +nightly sqlx migrate run --source db/migrations` once a `DATABASE_URL` pointing at a PostgreSQL instance is available.
- The schema currently provisions tables for `servers`, `server_members`, `channels`, `private_channel_members`, `roles`, `member_roles`, `permission_overrides`, and `webhooks` together with supporting enums and indexes.

### Design assumptions

- `owner_user_id` on `servers` and `user_id` on `server_members` reference an external identity service and therefore use plain `UUID` columns without foreign keys.
- Channels include a `kind` enum (text, voice, stage, forum, announcement, category) and a `visibility` enum to distinguish private channels. Private channel membership is modeled via the `private_channel_members` join table.
- Permission overrides are scoped at the channel level and target either a role or a member; mutually exclusive foreign keys plus a discriminator column enforce this at the database layer.
- Role permissions are stored as a `BIGINT` bitset to allow the application layer to evolve without schema changes.
