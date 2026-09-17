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

require '      - run: bash scripts/verify-ci-evidence-artifact.test.sh'
require '      - name: Capture CI evidence'
require '      - name: Upload CI evidence'

web_check_line="$(line_of '      - run: pnpm check:web')"
static_test_line="$(line_of '      - run: bash scripts/verify-ci-evidence-artifact.test.sh')"
capture_line="$(line_of '      - name: Capture CI evidence')"
upload_line="$(line_of '      - name: Upload CI evidence')"

if (( web_check_line >= capture_line || static_test_line >= capture_line || capture_line >= upload_line )); then
  printf 'CI evidence steps must follow checks and precede upload\n' >&2
  exit 1
fi

capture_block="$(sed -n "${capture_line},$((upload_line - 1))p" "$workflow_path")"
for forbidden in 'postgres://' 'POSTGRES_PASSWORD' 'DATABASE_URL' 'password' 'CONTEXTLAB_'; do
  if [[ "$capture_block" == *"$forbidden"* ]]; then
    printf 'CI evidence capture must not include %s\n' "$forbidden" >&2
    exit 1
  fi
done

require '        if: ${{ success() }}'
require '          checked_out_commit_sha="$(git rev-parse --verify HEAD^{commit})"'
require '          provider_head_sha="${GITHUB_SHA:?GITHUB_SHA is required}"'
require '          workflow_path=".github/workflows/verify.yml"'
require '          workflow_sha256="$(sha256sum "$workflow_path" | awk '\''{print $1}'\'')"'
require '          sha256sum "$evidence_dir/verification-evidence.txt" > "$evidence_dir/verification-evidence.txt.sha256"'
require '        uses: actions/upload-artifact@v4'
require '          name: contextlab-ci-evidence'
require '          path: .ci-evidence/'
require '          include-hidden-files: true'
require '          retention-days: 90'
require '          if-no-files-found: error'
