#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 1 ]]; then
  echo "usage: write-production-rehearsal-evidence.sh <output-path>" >&2
  exit 1
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
output_path="$1"
output_directory="$(dirname "$output_path")"
migration_directory="$repo_root/crates/storage/migrations"
privileged_directory="$repo_root/crates/storage/privileged"
rehearsal_script="$repo_root/scripts/verify-production-migration-rehearsal.sh"

mapfile -t migration_files < <(find "$migration_directory" -maxdepth 1 -type f -name '*.sql' -print | LC_ALL=C sort)
mapfile -t privileged_files < <(find "$privileged_directory" -maxdepth 1 -type f -name '*.sql' -print | LC_ALL=C sort)

if (( ${#migration_files[@]} == 0 )); then
  echo "no regular migration assets found" >&2
  exit 1
fi

if (( ${#privileged_files[@]} == 0 )); then
  echo "no privileged SQL assets found" >&2
  exit 1
fi

if [[ ! -f "$rehearsal_script" ]]; then
  echo "production rehearsal script is missing" >&2
  exit 1
fi

mkdir -p "$output_directory"
temporary_output="${output_path}.tmp.$$"

cleanup() {
  rm -f "$temporary_output"
}

trap cleanup EXIT

write_asset() {
  local record_type="$1"
  local asset_path="$2"
  local relative_path="${asset_path#$repo_root/}"
  local digest

  digest="$(sha256sum "$asset_path" | awk '{print $1}')"
  printf '%s=%s sha256=%s\n' "$record_type" "$relative_path" "$digest"
}

{
  printf '%s\n' 'schema_version=production-rehearsal-evidence-capture.v1'
  printf '%s\n' 'capture_type=technical-input-only'

  for migration_file in "${migration_files[@]}"; do
    write_asset 'regular_migration' "$migration_file"
  done

  for privileged_file in "${privileged_files[@]}"; do
    write_asset 'privileged_artifact' "$privileged_file"
  done

  write_asset 'rehearsal_script' "$rehearsal_script"
  printf '%s\n' 'capture_scope=asset-integrity-only'
  printf '%s\n' 'redaction_statement=no-environment-values-connection-urls-credentials-or-logs'
} > "$temporary_output"

mv "$temporary_output" "$output_path"
