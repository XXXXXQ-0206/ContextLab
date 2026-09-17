#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
writer="$script_dir/write-production-rehearsal-evidence.sh"
output_path="$(mktemp)"

cleanup() {
  rm -f "$output_path"
}

trap cleanup EXIT

bash "$writer" "$output_path"

require_line() {
  local expected="$1"

  if ! grep -Fxq -- "$expected" "$output_path"; then
    printf 'expected manifest line: %s\n' "$expected" >&2
    exit 1
  fi
}

require_line 'schema_version=production-rehearsal-evidence-capture.v1'
require_line 'capture_type=technical-input-only'
require_line 'capture_scope=asset-integrity-only'
require_line 'redaction_statement=no-environment-values-connection-urls-credentials-or-logs'

assert_manifest_group() {
  local prefix="$1"
  local directory="$2"
  local label="$3"
  local expected_count=0

  while IFS= read -r asset; do
    local relative_path="${asset#$repo_root/}"
    local digest
    digest="$(sha256sum "$asset" | awk '{print $1}')"
    require_line "${prefix}=${relative_path} sha256=${digest}"
    expected_count=$((expected_count + 1))
  done < <(find "$directory" -maxdepth 1 -type f -name '*.sql' -print | LC_ALL=C sort)

  local actual_count
  actual_count="$(grep -c "^${prefix}=" "$output_path")"
  if [[ "$actual_count" != "$expected_count" ]]; then
    printf 'expected %s %s entries, got %s\n' "$expected_count" "$label" "$actual_count" >&2
    exit 1
  fi
}

assert_manifest_group 'regular_migration' "$repo_root/crates/storage/migrations" 'regular migration'
assert_manifest_group 'privileged_artifact' "$repo_root/crates/storage/privileged" 'privileged artifact'

rehearsal_digest="$(sha256sum "$repo_root/scripts/verify-production-migration-rehearsal.sh" | awk '{print $1}')"
require_line "rehearsal_script=scripts/verify-production-migration-rehearsal.sh sha256=${rehearsal_digest}"

for prefix in regular_migration privileged_artifact; do
  if ! sort -c < <(grep "^${prefix}=" "$output_path"); then
    printf '%s entries must be lexically sorted\n' "$prefix" >&2
    exit 1
  fi
done

for forbidden in 'postgres://' 'password' 'DATABASE_URL' 'CONTEXTLAB_' 'psql' 'cargo test'; do
  if grep -Fq -- "$forbidden" "$output_path"; then
    printf 'manifest must not contain %s\n' "$forbidden" >&2
    exit 1
  fi

  if grep -Fq -- "$forbidden" "$writer"; then
    printf 'writer must not contain %s\n' "$forbidden" >&2
    exit 1
  fi
done
