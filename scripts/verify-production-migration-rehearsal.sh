#!/usr/bin/env bash
set -euo pipefail

database_url="${CONTEXTLAB_REHEARSAL_DATABASE_URL:-}"
expected_database_name="${CONTEXTLAB_REHEARSAL_DATABASE_NAME:-contextlab_rehearsal}"
expected_database_role="${CONTEXTLAB_REHEARSAL_DATABASE_ROLE:-contextlab_rehearsal_runner}"

if [[ -z "$database_url" ]]; then
  echo "CONTEXTLAB_REHEARSAL_DATABASE_URL must name a disposable PostgreSQL database." >&2
  exit 1
fi

if [[ "${CONTEXTLAB_ALLOW_REHEARSAL_DATABASE_RESET:-}" != "1" ]]; then
  echo "CONTEXTLAB_ALLOW_REHEARSAL_DATABASE_RESET=1 is required before schema reset." >&2
  exit 1
fi

case "$database_url" in
  postgres://*@localhost:*|postgresql://*@localhost:*|postgres://*@127.0.0.1:*|postgresql://*@127.0.0.1:*|postgres://*@\[::1\]:*|postgresql://*@\[::1\]:*) ;;
  *)
    echo "CONTEXTLAB_REHEARSAL_DATABASE_URL must use a loopback PostgreSQL host." >&2
    exit 1
    ;;
esac

if ! command -v psql >/dev/null 2>&1; then
  echo "psql is required to reset the rehearsal PostgreSQL schema." >&2
  exit 1
fi

active_database_name="$(psql "$database_url" -X -At -v ON_ERROR_STOP=1 -c 'SELECT current_database()')"
if [[ "$active_database_name" != "$expected_database_name" ]]; then
  echo "The rehearsal database name does not match CONTEXTLAB_REHEARSAL_DATABASE_NAME." >&2
  exit 1
fi

active_database_role="$(psql "$database_url" -X -At -v ON_ERROR_STOP=1 -c 'SELECT current_user')"
if [[ "$active_database_role" != "$expected_database_role" ]]; then
  echo "The rehearsal database role does not match CONTEXTLAB_REHEARSAL_DATABASE_ROLE." >&2
  exit 1
fi

psql "$database_url" -X -v ON_ERROR_STOP=1 <<'SQL'
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO CURRENT_USER;
SQL

CONTEXTLAB_TEST_DATABASE_URL="$database_url" \
  cargo test -p contextlab-storage \
  postgres::tests::production_like_migration_rehearsal_upgrades_history_and_preserves_ledger \
  --lib -- --ignored --exact --test-threads=1
