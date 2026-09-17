#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
verification_script="$script_dir/verify-disposable-postgres-storage.sh"
temporary_bin="$(mktemp -d)"
invocations="$temporary_bin/invocations"

cleanup() {
  rm -rf "$temporary_bin"
}

trap cleanup EXIT

psql() {
  printf '%s\n' "$*" >>"$CONTEXTLAB_TEST_INVOCATIONS"

  case "$*" in
    *current_database*)
      printf '%s\n' "${CONTEXTLAB_TEST_ACTIVE_DATABASE_NAME:?}"
      ;;
    *current_user*)
      printf '%s\n' "${CONTEXTLAB_TEST_ACTIVE_DATABASE_USER:?}"
      ;;
  esac
}

cargo() {
  printf '%s\n' "$*" >>"$CONTEXTLAB_TEST_INVOCATIONS"
}

export -f psql cargo

run_verification() {
  local database_url="$1"
  local expected_role="$2"
  local active_role="$3"
  local output
  local status

  : >"$invocations"
  set +e
  output="$(
    CONTEXTLAB_TEST_DATABASE_URL="$database_url" \
      CONTEXTLAB_TEST_DATABASE_NAME="contextlab_test" \
      CONTEXTLAB_TEST_DATABASE_ROLE="$expected_role" \
      CONTEXTLAB_TEST_ACTIVE_DATABASE_NAME="contextlab_test" \
      CONTEXTLAB_TEST_ACTIVE_DATABASE_USER="$active_role" \
      CONTEXTLAB_TEST_INVOCATIONS="$invocations" \
      CONTEXTLAB_ALLOW_DISPOSABLE_DATABASE_RESET=1 \
      "$BASH" "$verification_script" 2>&1
  )"
  status=$?
  set -e

  printf '%s\n%s\n' "$status" "$output"
}

assert_contains() {
  local haystack="$1"
  local needle="$2"

  if [[ "$haystack" != *"$needle"* ]]; then
    printf 'expected output to contain %q, got: %s\n' "$needle" "$haystack" >&2
    exit 1
  fi
}

assert_no_psql_invocation() {
  if [[ -s "$invocations" ]]; then
    printf 'expected URL guard to reject before psql, got: %s\n' "$(cat "$invocations")" >&2
    exit 1
  fi
}

remote_result="$(run_verification 'postgres://contextlab_test_runner:test-password@example.test:5432/contextlab_test' 'contextlab_test_runner' 'contextlab_test_runner')"
assert_contains "$remote_result" 'must use a loopback PostgreSQL host'
assert_no_psql_invocation

wrong_role_result="$(run_verification 'postgres://contextlab_test_runner:test-password@localhost:55432/contextlab_test' 'contextlab_test_runner' 'postgres')"
assert_contains "$wrong_role_result" 'does not match CONTEXTLAB_TEST_DATABASE_ROLE'

valid_result="$(run_verification 'postgres://contextlab_test_runner:test-password@127.0.0.1:55432/contextlab_test' 'contextlab_test_runner' 'contextlab_test_runner')"
assert_contains "$valid_result" '0'

cargo_invocation_count="$(rg --count '^test -p contextlab-storage' "$invocations")"
if [[ "$cargo_invocation_count" != '26' ]]; then
  printf 'expected 26 isolated storage test invocations, got %s\n' "$cargo_invocation_count" >&2
  exit 1
fi
