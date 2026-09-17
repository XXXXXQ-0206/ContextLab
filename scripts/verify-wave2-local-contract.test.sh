#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$BASH_SOURCE")" && pwd)"
verification_script="$script_dir/verify-wave2-local-contract.sh"
temporary_root="$(mktemp -d)"

cleanup() {
  rm -rf "$temporary_root"
}

trap cleanup EXIT

write_fixture() {
  local root="$1"

  mkdir -p \
    "$root/crates/diff-engine/src" \
    "$root/crates/mcp/src" \
    "$root/crates/plugin-runtime/src" \
    "$root/crates/workflow/src" \
    "$root/crates/knowledge/src" \
    "$root/crates/memory/src" \
    "$root/apps/cli/src" \
    "$root/apps/desktop/src-tauri/src" \
    "$root/apps/web/src/app" \
    "$root/apps/cli/crates/contextlab-adapter-contract/src" \
    "$root/apps/cli/crates/contextlab-adapter-contract/tests" \
    "$root/packages/ui" \
    "$root/packages/design-system"

  printf '%s\n' \
    '[workspace]' \
    'members = [' \
    '  "crates/diff-engine",' \
    '  "crates/mcp",' \
    '  "crates/plugin-runtime",' \
    '  "crates/workflow",' \
    '  "crates/knowledge",' \
    '  "crates/memory",' \
    '  "apps/cli",' \
    '  "apps/desktop/src-tauri",' \
    ']' >"$root/Cargo.toml"
  printf '%s\n' 'packages:' '  - "apps/*"' '  - "packages/*"' >"$root/pnpm-workspace.yaml"

  for manifest in \
    crates/diff-engine \
    crates/mcp \
    crates/plugin-runtime \
    crates/workflow \
    crates/knowledge \
    crates/memory \
    apps/cli \
    apps/desktop/src-tauri; do
    printf '%s\n' '[package]' "name = \"fixture-${manifest//\//-}\"" >"$root/$manifest/Cargo.toml"
  done

  printf '%s\n' '{"name":"@contextlab/web"}' >"$root/apps/web/package.json"
  printf '%s\n' '{"name":"@contextlab/ui"}' >"$root/packages/ui/package.json"
  printf '%s\n' '{"name":"@contextlab/design-system"}' >"$root/packages/design-system/package.json"

  printf '%s\n' \
    'const LOCAL_CAPABILITY_AVAILABILITY_V1: &str = "contextlab.local-capability-availability.v1";' \
    'pub struct LocalCapabilityAvailabilityV1 {' \
    '    schema_version: String,' \
    '    operation_id: String,' \
    '    integration: String,' \
    '    availability: LocalCapabilityAvailability,' \
    '    reason: String,' \
    '}' \
    'pub fn local_capability_availability() {}' \
    '"shared_integration_not_registered"' >"$root/apps/cli/crates/contextlab-adapter-contract/src/lib.rs"
  printf '%s\n' 'unavailable_response_projects_a_versioned_local_capability_dto' >"$root/apps/cli/crates/contextlab-adapter-contract/tests/unavailable_contract.rs"
  printf '%s\n' \
    'export const LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1 = "contextlab.local-capability-availability.v1";' \
    'export type LocalCapabilityAvailabilityDto = Readonly<{' \
    '  schema_version: typeof LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1;' \
    '  operation_id: "evaluation-run";' \
    '  integration: "contextlab-evaluation";' \
    '  availability: "unavailable";' \
    '  reason: "shared_integration_not_registered";' \
    '}>;' \
    'export function parseLocalCapabilityAvailability(value: unknown) {}' >"$root/apps/web/src/app/local-capability-availability-data.ts"
  printf '%s\n' \
    'use contextlab_adapter_contract::LocalCapabilityAvailabilityV1;' \
    'response.local_capability_availability(&request);' >"$root/apps/cli/src/lib.rs"
  printf '%s\n' \
    'use contextlab_adapter_contract::LocalCapabilityAvailabilityV1;' \
    'response.local_capability_availability(&request);' >"$root/apps/desktop/src-tauri/src/lib.rs"

  printf '%s\n' 'GraphDiff::between(&original, &revised);' >"$root/crates/diff-engine/src/application.rs"
  for path in \
    crates/mcp/src/lib.rs \
    crates/plugin-runtime/src/lib.rs \
    crates/workflow/src/lib.rs \
    crates/knowledge/src/lib.rs \
    crates/memory/src/lib.rs \
    apps/cli/src/lib.rs \
    apps/desktop/src-tauri/src/lib.rs \
    apps/web/src/app/capability-state-screen.tsx; do
    : >"$root/$path"
  done
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
  local diff_path="${2:-}"
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

passed_result="$(run_case "$fixture_root")"
  assert_contains "$passed_result" 'wave2_local_contracts=passed'
  assert_contains "$passed_result" 'graph_diff_calculators=passed count=1'
  assert_contains "$passed_result" 'capability_availability_contract=passed'
  assert_contains "$passed_result" 'external_release_evidence=deferred'
  assert_contains "$passed_result" 'overall=unobserved'

printf '%s\n' 'GraphDiff::between(&fixture, &fixture);' >"$fixture_root/crates/diff-engine/src/lib.rs"
test_reference_result="$(run_case "$fixture_root")"
  assert_contains "$test_reference_result" 'graph_diff_calculators=passed count=1'

  readonly_diff="$temporary_root/readonly.diff"
  printf '%s\n' \
    '--- a/apps/cli/src/lib.rs' \
    '+++ b/apps/cli/src/lib.rs' \
    '@@' \
    '+response.local_capability_availability(&request);' >"$readonly_diff"
  readonly_result="$(run_case "$fixture_root" "$readonly_diff")"
  assert_contains "$readonly_result" 'capability_availability_contract=passed'
  assert_contains "$readonly_result" 'public_write_additions=passed'
  assert_contains "$readonly_result" 'secret_reading_additions=passed'
  assert_contains "$readonly_result" 'overall=passed'

  public_rest_diff="$temporary_root/public-rest-write.diff"
  printf '%s\n' \
    '--- a/server/api/src/routes.rs' \
    '+++ b/server/api/src/routes.rs' \
    '@@' \
    '+router.route("/api/v1/contexts", post(routes::create_context));' >"$public_rest_diff"
  public_rest_result="$(run_case "$fixture_root" "$public_rest_diff")"
  assert_contains "$public_rest_result" 'public_write_additions=blocked'
  assert_contains "$public_rest_result" 'overall=blocked'

  openapi_diff="$temporary_root/public-openapi-write.diff"
  printf '%s\n' \
    '--- a/docs/api/openapi.json' \
    '+++ b/docs/api/openapi.json' \
    '@@' \
    '+"/api/v1/contexts": {"post": {"operationId": "createContext"}}' >"$openapi_diff"
  openapi_result="$(run_case "$fixture_root" "$openapi_diff")"
  assert_contains "$openapi_result" 'public_write_additions=blocked'
  assert_contains "$openapi_result" 'overall=blocked'

  public_sdk_diff="$temporary_root/public-sdk-write.diff"
  printf '%s\n' \
    '--- a/packages/ts-sdk/src/client.ts' \
    '+++ b/packages/ts-sdk/src/client.ts' \
    '@@' \
    '+async createContextCommit(request: CreateContextCommitRequest): Promise<CommitDetail> {' >"$public_sdk_diff"
  public_sdk_result="$(run_case "$fixture_root" "$public_sdk_diff")"
  assert_contains "$public_sdk_result" 'public_write_additions=blocked'
  assert_contains "$public_sdk_result" 'overall=blocked'

  secret_read_diff="$temporary_root/secret-read.diff"
  printf '%s\n' \
    '--- a/apps/cli/src/lib.rs' \
    '+++ b/apps/cli/src/lib.rs' \
    '@@' \
    '+let token = std::env::var("TOKEN");' >"$secret_read_diff"
  secret_read_result="$(run_case "$fixture_root" "$secret_read_diff")"
  assert_contains "$secret_read_result" 'secret_reading_additions=blocked'
  assert_contains "$secret_read_result" 'overall=blocked'

printf '%s\n' 'GraphDiff::between(&original, &revised);' >>"$fixture_root/crates/diff-engine/src/application.rs"
  duplicate_result="$(run_case "$fixture_root")"
  assert_contains "$duplicate_result" 'graph_diff_calculators=blocked count=2'
  assert_contains "$duplicate_result" 'overall=blocked'

  missing_capability_root="$temporary_root/missing-capability"
  write_fixture "$missing_capability_root"
  rm "$missing_capability_root/apps/cli/crates/contextlab-adapter-contract/src/lib.rs"
  missing_capability_result="$(run_case "$missing_capability_root")"
  assert_contains "$missing_capability_result" 'capability_availability_contract=blocked'
  assert_contains "$missing_capability_result" 'overall=blocked'

bash -n "$verification_script"
