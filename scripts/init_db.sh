#!/usr/bin/env bash

# fails if any command fails
set -x
# fails if any pipe fails
set -eo pipefail
# fails if any env variable is not set
set -o nounset

NEWSLETTER_DB_USER="${NEWSLETTER_DB_USER:=postgres}"
NEWSLETTER_DB_PASSWORD="${NEWSLETTER_DB_PASSWORD:=password}"
NEWSLETTER_DB_NAME="${NEWSLETTER_DB_NAME:=newsletter}"
NEWSLETTER_DB_PORT="${NEWSLETTER_DB_PORT:=9000}"
NEWSLETTER_DB_URL="postgres://${NEWSLETTER_DB_USER}:${NEWSLETTER_DB_PASSWORD}@localhost:${NEWSLETTER_DB_PORT}/${NEWSLETTER_DB_NAME}"

POSTGRES_CONTAINER_ID=$(docker run \
  -e POSTGRES_USER="${NEWSLETTER_DB_USER}" \
  -e POSTGRES_PASSWORD="${NEWSLETTER_DB_PASSWORD}" \
  -e POSTGRES_NAME="${NEWSLETTER_DB_NAME}" \
  -e POSTGRES_PORT="${NEWSLETTER_DB_PORT}" \
  -p "${NEWSLETTER_DB_PORT}":5432 \
  -d postgres \
  postgres -N 1000)

echo >&2 "started postgres container with id: ${POSTGRES_CONTAINER_ID}"

# wait postgres to be ready to accept commands
while ! docker exec "${POSTGRES_CONTAINER_ID}" pg_isready -U postgres; do
  echo >&2 "postgres is still unavailable - sleeping"
  sleep 1
done
echo >&2 "postgres is up and running on port ${NEWSLETTER_DB_PORT}!"

DATABASE_URL="${NEWSLETTER_DB_URL}" sqlx database create
