#!/usr/bin/env zsh
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

if false; then
docker run \
  -e POSTGRES_USER="${NEWSLETTER_DB_USER}" \
  -e POSTGRES_PASSWORD="${NEWSLETTER_DB_PASSWORD}" \
  -e POSTGRES_NAME="${NEWSLETTER_DB_NAME}" \
  -e POSTGRES_PORT="${NEWSLETTER_DB_PORT}" \
  -p "${NEWSLETTER_DB_PORT}":5432 \
  -d postgres \
  postgres -N 1000
fi

DATABASE_URL="${NEWSLETTER_DB_URL}" sqlx database create
