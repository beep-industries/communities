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
- [sqlx-cli](https://crates.io/crates/sqlx-cli)

## Quickstart


Launch postgres:

```bash
docker compose up -d postgres
```
Create the .env file to let sqlx know how to connect to the database:

```bash
cp .env.example .env
```

Run migrations:

```bash
sqlx migrate run --source core/migrations
```

## Persistence

To persist data we use PostgreSQL. To handle uuid inside the database we use the `pg-crypto` extension.
In dev mode it should be enabled automatically due to the init script you can find in [`compose/init-uuid.sql`](compose/init-uuid.sql).

The sql migration files are located in the [`core/migrations`](core/migrations) folder.

## How to create a SQLx migration

````
sqlx migrate add <migration-name> --source core/migrations
```