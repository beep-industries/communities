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

Create the .env file from the example:

```bash
cp .env.example .env
```

Create network & start rabbitmq:

```bash
docker network create authz_communities
docker compose --profile lazy up rabbitmq rabbitmq-init -d
```

Start the [user service](https://github.com/beep-industries/user)
Start the [authz service](https://github.com/beep-industries/authz)

You are almost done, start the app & db:

```bash
docker compose --profile lazy up -d
```

The application runs two servers on separate ports:

- **Health server** on `http://localhost:9090`
- **API server** on `http://localhost:3003` - Main application endpoints

## Configuration

You can pass down some configuration using `--help`:

```bash
cargo run --bin api -- --help
```

You can now see all the possible way to configure the service:

```bash
Communities API Server

Usage: api [OPTIONS] --database-password <database_password> --jwt-secret-key <jwt_secret_key>

Options:
      --database-host <HOST>
          [env: DATABASE_HOST=] [default: localhost]
      --database-port <PORT>
          [env: DATABASE_PORT=] [default: 5432]
      --database-user <USER>
          [env: DATABASE_USER=] [default: postgres]
      --database-password <database_password>
          [env: DATABASE_PASSWORD=]
      --database-name <database_name>
          [env: DATABASE_NAME=] [default: communities]
      --jwt-secret-key <jwt_secret_key>
          [env: JWT_SECRET_KEY=a-string-secret-at-least-256-bits-long]
      --server-api-port <api_port>
          [env: API_PORT=3003] [default: 8080]
      --server-health-port <HEALTH_PORT>
          [env: HEALTH_PORT=9090] [default: 8081]
        --cors-origins <origins>
          [env: CORS_ORIGINS=http://localhost:3003,https://beep.ovh] [default: http://localhost:3003, https://beep.ovh]
  -h, --help
          Print help
```

## Persistence

To persist data we use PostgreSQL. To handle uuid inside the database we use the `pg-crypto` extension.
In dev mode it should be enabled automatically due to the init script you can find in [`compose/init-uuid.sql`](compose/init-uuid.sql).

The sql migration files are located in the [`core/migrations`](core/migrations) folder.

## Apply Database Migrations

Before running the API in development (or when setting up a fresh DB), apply the migrations:

```zsh
# Start Postgres (if not already running)
docker compose up -d postgres

# Apply all pending migrations
sqlx migrate run --source core/migrations --database-url postgres://postgres:password@localhost:5432/communities

# (Optional) Show migration status
sqlx migrate info --source core/migrations --database-url postgres://postgres:password@localhost:5432/communities
```

## How to create a SQLx migration

```
sqlx migrate add <migration-name> --source core/migrations
```

## Running tests

There are two kinds of tests in this repo:

- Infrastructure tests that hit a real Postgres database (via `sqlx::test`).
- Domain tests that use mocked repositories (no database required).

Recommended workflow for all tests (infrastructure + domain):

```zsh
# Start Postgres from docker-compose & run the migration
docker compose up -d

sqlx migrate run --source core/migrations

# Run the test
cargo test
```

Run only domain tests (no DB needed):

```zsh
cargo test domain::test -- -q
```

Notes:

- `#[sqlx::test(migrations = "./migrations")]` automatically applies migrations to an isolated test database.
- Only a reachable Postgres server and `DATABASE_URL` env var are required; you do not need to run migrations manually for tests.
- If you run the API or any non-`sqlx::test` integration tests that expect existing tables, apply migrations first (see "Apply Database Migrations" below).
