# Communities service

The Communities service is designed to facilitate the creation, management, and interaction of user communities within the platform.
It will handle:

- Servers
- Members
- Roles
- Channels

## Prerequisites

- [Docker](https://www.docker.com/get-started/)
- Rust and Cargo
- [slqx-cli](https://crates.io/crates/sqlx-cli)

## Quickstart

Launch postgres:

```bash
docker compose up -d postgres
```
Run migrations:

```bash
slqx migrate run --source db/migrations
```
