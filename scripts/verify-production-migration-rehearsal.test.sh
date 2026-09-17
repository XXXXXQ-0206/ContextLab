#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
rehearsal_script="$script_dir/verify-production-migration-rehearsal.sh"

set +e
output="$(bash "$rehearsal_script" 2>&1)"
status=$?
set -e

if [[ "$status" -eq 0 ]]; then
  echo "expected rehearsal script to reject missing configuration" >&2
  exit 1
fi

if [[ "$output" != *"CONTEXTLAB_REHEARSAL_DATABASE_URL must name a disposable PostgreSQL database."* ]]; then
  printf 'unexpected missing-configuration output: %s\n' "$output" >&2
  exit 1
fi
