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

Launch the API server:

```bash
cargo run --bin api
```
The application runs two servers on separate ports:
- **Health server** on `http://localhost:9090` - Isolated health checks (prevents DDOS on API)
  - `GET /health` - Health check with database connectivity
- **API server** on `http://localhost:3001` - Main application endpoints
  - Future business logic endpoints will be added here

This dual-server architecture provides DDOS protection by isolating health checks from API traffic.

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
          [env: API_PORT=3001] [default: 8080]
      --server-health-port <HEALTH_PORT>
          [env: HEALTH_PORT=9090] [default: 8081]
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
# 1) Start Postgres from docker-compose
docker compose up -d postgres

# 2) Point SQLx to your database server (the tests will create/drop their own DBs)
export DATABASE_URL="postgres://postgres:password@localhost:5432/communities"

# 3) Run tests for the core crate (includes infrastructure + domain tests)
cargo test -p communities_core -- --nocapture

# Or run the entire workspace
cargo test --workspace -- --nocapture
```

Run only domain tests (no DB needed):

```zsh
cargo test domain::test -- -q
```

Notes:
- `#[sqlx::test(migrations = "./migrations")]` automatically applies migrations to an isolated test database.
- Only a reachable Postgres server and `DATABASE_URL` env var are required; you do not need to run migrations manually for tests.
 - If you run the API or any non-`sqlx::test` integration tests that expect existing tables, apply migrations first (see "Apply Database Migrations" below).