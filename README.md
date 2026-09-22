# Rincuma

Self-hostedable To-Do list application

## Name definition

rincuma
Q. noun. task, charge, mission

[Source - Parf Edhellen](https://www.elfdict.com/wt/526035)

## Development

### Running the API with Docker

The development stack runs PostgreSQL and the API with hot reload. From the
`api` directory run:

```bash
cp .env.example .env
docker compose --env-file .env -f docker/compose.dev.yaml up --build
```

The API is available at `http://localhost:7878` and PostgreSQL at
`localhost:5432`. Changes in `api/src` automatically restart the API.

Stop the development stack with:

```bash
docker compose --env-file .env -f docker/compose.dev.yaml down
```

### Database migrations

The migration CLI is optional and does not become part of the API binary. Run
these commands from the `api` directory with PostgreSQL available:

```bash
cargo run --features migration-cli --bin rincuma-migrate -- migration generate --name auth
cargo run --features migration-cli --bin rincuma-migrate -- migration apply
```

Generated migrations, schema snapshots, and migration history are written to
the `api/toasty` directory.
