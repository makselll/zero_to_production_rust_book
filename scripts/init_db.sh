#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install it."
  exit 1
fi

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${APP__DATABASE__USERNAME:=postgres}
# Check if a custom password has been set, otherwise default to 'password'
DB_PASSWORD="${APP__DATABASE__PASSWORD:=password}"
# Check if a custom database name has been set, otherwise default to 'newsletter'
DB_NAME="${APP__DATABASE__DATABASE_NAME:=newsletter}"
# Check if a custom port has been set, otherwise default to '5432'
DB_PORT="${APP__DATABASE__PORT:=5432}"

DB_HOST="${APP__DATABASE__HOST:=localhost}"

# Start the services using Docker Compose
if [[ -z "${SKIP_DB}" ]]
then
  docker compose up -d
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"