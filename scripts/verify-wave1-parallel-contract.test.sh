#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$BASH_SOURCE")" && pwd)"
verification_script="$script_dir/verify-wave1-parallel-contract.sh"
temporary_root="$(mktemp -d)"

cleanup() {
  rm -rf "$temporary_root"
}

trap cleanup EXIT

write_fixture() {
  local root="$1"

  mkdir -p "$root/crates/evaluation/src" "$root/apps/web" "$root/packages/ui"
  printf '%s\n' '[workspace]' 'members = [' '  "crates/evaluation",' ']' >"$root/Cargo.toml"
  printf '%s\n' '[package]' 'name = "contextlab-evaluation"' >"$root/crates/evaluation/Cargo.toml"
  printf '%s\n' 'packages:' '  - "apps/*"' '  - "packages/*"' >"$root/pnpm-workspace.yaml"
  printf '%s\n' '{"name":"@contextlab/web"}' >"$root/apps/web/package.json"
  printf '%s\n' '{"name":"@contextlab/ui"}' >"$root/packages/ui/package.json"
}

write_diff() {
  local path="$1"
  local content="$2"

  printf '%s\n' "--- a/crates/evaluation/src/lib.rs" "+++ b/crates/evaluation/src/lib.rs" '@@' "+$content" >"$path"
}

assert_contains() {
  local haystack="$1"
  local needle="$2"

  if [[ "$haystack" != *"$needle"* ]]; then
    printf 'expected output to contain %q, got: %s\n' "$needle" "$haystack" >&2
    exit 1
  fi
}

run_case() {
  local root="$1"
  local diff_path=''
  if [[ "$#" -ge 2 ]]; then
    diff_path="$2"
  fi
  local output
  local status

  set +e
  if [[ -n "$diff_path" ]]; then
    output="$("$BASH" "$verification_script" "$root" "$diff_path" 2>&1)"
  else
    output="$("$BASH" "$verification_script" "$root" 2>&1)"
  fi
  status=$?
  set -e
  printf '%s\n%s\n' "$status" "$output"
}

fixture_root="$temporary_root/fixture"
write_fixture "$fixture_root"

no_diff_result="$(run_case "$fixture_root")"
assert_contains "$no_diff_result" 'workspace_members=passed'
assert_contains "$no_diff_result" 'change_set=unobserved'
assert_contains "$no_diff_result" 'public_write_additions=unobserved'
assert_contains "$no_diff_result" 'secret_reading_additions=unobserved'
assert_contains "$no_diff_result" 'docker_runtime=ignored'
assert_contains "$no_diff_result" 'network_access=ignored'

readonly_diff="$temporary_root/readonly.diff"
write_diff "$readonly_diff" 'pub fn read_contract() {}'
readonly_result="$(run_case "$fixture_root" "$readonly_diff")"
assert_contains "$readonly_result" 'workspace_members=passed'
assert_contains "$readonly_result" 'change_set=passed'
assert_contains "$readonly_result" 'public_write_additions=passed'
assert_contains "$readonly_result" 'secret_reading_additions=passed'
assert_contains "$readonly_result" 'overall=passed'

public_write_diff="$temporary_root/public-write.diff"
write_diff "$public_write_diff" 'router.route("/api/v1/contexts", post(create_context));'
public_write_result="$(run_case "$fixture_root" "$public_write_diff")"
assert_contains "$public_write_result" 'public_write_additions=blocked'
assert_contains "$public_write_result" 'overall=blocked'

secret_read_diff="$temporary_root/secret-read.diff"
write_diff "$secret_read_diff" 'let key = std::env::var("API_KEY");'
secret_read_result="$(run_case "$fixture_root" "$secret_read_diff")"
assert_contains "$secret_read_result" 'secret_reading_additions=blocked'
assert_contains "$secret_read_result" 'overall=blocked'

secret_input="$temporary_root/.env"
printf '%s\n' 'API_KEY=must-not-be-read' >"$secret_input"
set +e
secret_input_output="$("$BASH" "$verification_script" "$fixture_root" "$secret_input" 2>&1)"
secret_input_status=$?
set -e
if [[ "$secret_input_status" -eq 0 ]]; then
  echo 'expected secret-looking diff input to be rejected' >&2
  exit 1
fi
assert_contains "$secret_input_output" 'change_set=blocked'
assert_contains "$secret_input_output" 'refusing secret-looking input'

incomplete_root="$temporary_root/incomplete"
write_fixture "$incomplete_root"
rm "$incomplete_root/crates/evaluation/Cargo.toml"
incomplete_result="$(run_case "$incomplete_root")"
assert_contains "$incomplete_result" 'workspace_members=blocked'
assert_contains "$incomplete_result" 'overall=blocked'

if rg -n '(^|[[:space:]])(docker|podman|curl|wget|nc|psql)([[:space:]]|$)' "$verification_script"; then
  echo 'verification script must not invoke Docker or network clients' >&2
  exit 1
fi

bash -n "$verification_script"
