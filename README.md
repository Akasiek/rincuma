# Rincuma

Self-hostedable To-Do list application

## Name definition

rincuma
Q. noun. task, charge, mission

[Source - Parf Edhellen](https://www.elfdict.com/wt/526035)

## Development

### Local API and PostgreSQL

The development Docker stack runs only PostgreSQL. From the `api` directory,
create `.env` on the first setup and start the database:

```bash
cp .env.example .env
docker compose --env-file .env -f docker/compose.dev.yaml up -d
```

Check that `DB_PASS` in `.env` matches the password in an existing PostgreSQL
volume. Run the Rust API locally from a second terminal in the `api` directory:

```bash
# with hot-reload
systemfd --no-pid -s http::7878 -- cargo watch -x run

# or without hot-reload
cargo run
```

The API is available at `http://localhost:7878` and PostgreSQL at
`localhost:5432`. The local API loads `.env`, including `DB_HOST=localhost`
and `COOKIE_SECURE=false`.

Stop the development stack with:

```bash
docker compose --env-file .env -f docker/compose.dev.yaml down
```

### Database migrations

The migration CLI is optional and does not become part of the API binary. Run
this command from the `api` directory with PostgreSQL available to migrate database:

```bash
cargo run --features migration-cli --bin rincuma-migrate -- migration apply
```

Generated migrations, schema snapshots, and migration history are written to
the `api/toasty` directory.
