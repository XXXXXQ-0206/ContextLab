#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workflow_path="${1:-$script_dir/../.github/workflows/verify.yml}"

require() {
  local expected="$1"

  if ! grep -Fq -- "$expected" "$workflow_path"; then
    printf 'expected workflow to contain: %s\n' "$expected" >&2
    exit 1
  fi
}

line_of() {
  local expected="$1"
  grep -Fn -- "$expected" "$workflow_path" | head -n 1 | cut -d: -f1
}

require '      - run: bash scripts/write-production-rehearsal-evidence.test.sh'
require '      - name: Capture rehearsal technical evidence'
require '      - name: Upload rehearsal technical evidence'

web_check_line="$(line_of '      - run: pnpm check:web')"
writer_test_line="$(line_of '      - run: bash scripts/write-production-rehearsal-evidence.test.sh')"
capture_line="$(line_of '      - name: Capture rehearsal technical evidence')"
upload_line="$(line_of '      - name: Upload rehearsal technical evidence')"

if (( web_check_line >= capture_line || writer_test_line >= capture_line || capture_line >= upload_line )); then
  printf 'rehearsal evidence steps must follow checks and precede upload\n' >&2
  exit 1
fi

capture_block="$(sed -n "${capture_line},$((upload_line - 1))p" "$workflow_path")"
upload_block="$(sed -n "${upload_line},\$p" "$workflow_path")"

require_capture() {
  local expected="$1"

  if [[ "$capture_block" != *"$expected"* ]]; then
    printf 'expected capture block to contain: %s\n' "$expected" >&2
    exit 1
  fi
}

require_upload() {
  local expected="$1"

  if [[ "$upload_block" != *"$expected"* ]]; then
    printf 'expected upload block to contain: %s\n' "$expected" >&2
    exit 1
  fi
}

require_capture 'if: ${{ success() }}'
require_capture 'evidence_dir=".rehearsal-evidence"'
require_capture 'bash scripts/write-production-rehearsal-evidence.sh "$evidence_dir/production-rehearsal-evidence.txt"'
require_capture 'sha256sum "$evidence_dir/production-rehearsal-evidence.txt" > "$evidence_dir/production-rehearsal-evidence.txt.sha256"'
require_upload 'if: ${{ success() }}'
require_upload 'uses: actions/upload-artifact@v4'
require_upload 'name: contextlab-rehearsal-evidence'
require_upload 'path: .rehearsal-evidence/'
require_upload 'retention-days: 90'
require_upload 'if-no-files-found: error'

for forbidden in 'postgres://' 'POSTGRES_PASSWORD' 'DATABASE_URL' 'password' 'CONTEXTLAB_'; do
  if [[ "$capture_block" == *"$forbidden"* ]]; then
    printf 'rehearsal evidence capture must not include %s\n' "$forbidden" >&2
    exit 1
  fi
done
