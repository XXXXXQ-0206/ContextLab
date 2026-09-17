#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$BASH_SOURCE")" && pwd)"
wave1_verifier="$script_dir/verify-wave1-parallel-contract.sh"

if [[ "$#" -gt 2 ]]; then
  printf 'usage: verify-wave2-local-contract.sh [repo-root] [safe-diff-file]\n' >&2
  exit 1
fi

if [[ "$#" -ge 1 ]]; then
  repo_root="$1"
else
  repo_root="$script_dir/.."
fi

if [[ ! -d "$repo_root" || -L "$repo_root" ]]; then
  printf 'wave2_local_contracts=blocked reason=missing-or-unsafe-repository-root\n' >&2
  printf 'overall=blocked\n'
  exit 1
fi

repo_root="$(cd "$repo_root" && pwd)"

if rg --version >/dev/null 2>&1; then
  search_fixed() {
    rg -Fq "$1" "$2"
  }

  search_regex() {
    rg -q "$1" "$2"
  }

  count_matches() {
    rg -n "$1" "$2" | wc -l | tr -d '[:space:]'
  }
else
  search_fixed() {
    grep -Fq -- "$1" "$2"
  }

  search_regex() {
    grep -Eq -- "$1" "$2"
  }

  count_matches() {
    grep -En -- "$1" "$2" | wc -l | tr -d '[:space:]'
  }
fi

if [[ "$#" -eq 2 ]]; then
  wave1_output="$("$BASH" "$wave1_verifier" "$repo_root" "$2" 2>&1)" || wave1_status=$?
else
  wave1_output="$("$BASH" "$wave1_verifier" "$repo_root" 2>&1)" || wave1_status=$?
fi
wave1_status="${wave1_status:-0}"
printf '%s\n' "$wave1_output" | sed '/^public_write_additions=/d; /^secret_reading_additions=/d; /^overall=/d'

local_contract_status=passed
mark_local_contract_blocked() {
  local_contract_status=blocked
  printf 'wave2_contract_detail=blocked reason=%s\n' "$1" >&2
}

required_paths=(
  'crates/diff-engine/src'
  'crates/mcp/src/lib.rs'
  'crates/plugin-runtime/src/lib.rs'
  'crates/workflow/src/lib.rs'
  'crates/knowledge/src/lib.rs'
  'crates/memory/src/lib.rs'
  'apps/cli/src/lib.rs'
  'apps/desktop/src-tauri/src/lib.rs'
  'apps/web/src/app/capability-state-screen.tsx'
)

for relative_path in "${required_paths[@]}"; do
  path="$repo_root/$relative_path"
  if [[ ! -e "$path" || -L "$path" ]]; then
    mark_local_contract_blocked "missing-or-unsafe-required-path:$relative_path"
  fi
done

for member in 'crates/mcp' 'crates/plugin-runtime'; do
  if ! search_fixed "\"$member\"" "$repo_root/Cargo.toml"; then
    mark_local_contract_blocked "missing-root-workspace-member:$member"
  fi
done

for manifest in 'crates/mcp/Cargo.toml' 'crates/plugin-runtime/Cargo.toml'; do
  if [[ ! -f "$repo_root/$manifest" || -L "$repo_root/$manifest" ]]; then
    mark_local_contract_blocked "missing-or-unsafe-manifest:$manifest"
  elif search_regex '^\[workspace\]' "$repo_root/$manifest"; then
    mark_local_contract_blocked "nested-workspace:$manifest"
  fi
done

capability_contract_status=passed
mark_capability_contract_blocked() {
  capability_contract_status=blocked
  mark_local_contract_blocked "$1"
}

capability_contract_files=(
  'apps/cli/crates/contextlab-adapter-contract/src/lib.rs'
  'apps/cli/crates/contextlab-adapter-contract/tests/unavailable_contract.rs'
  'apps/cli/src/lib.rs'
  'apps/desktop/src-tauri/src/lib.rs'
  'apps/web/src/app/local-capability-availability-data.ts'
)

for relative_path in "${capability_contract_files[@]}"; do
  path="$repo_root/$relative_path"
  if [[ ! -f "$path" || -L "$path" ]]; then
    mark_capability_contract_blocked "missing-or-unsafe-capability-contract-path:$relative_path"
  fi
done

adapter_contract="$repo_root/apps/cli/crates/contextlab-adapter-contract/src/lib.rs"
web_contract="$repo_root/apps/web/src/app/local-capability-availability-data.ts"
if [[ "$capability_contract_status" = passed ]]; then
  if ! search_fixed 'contextlab.local-capability-availability.v1' "$adapter_contract" \
    || ! search_fixed 'LocalCapabilityAvailabilityV1' "$adapter_contract" \
    || ! search_fixed 'local_capability_availability' "$adapter_contract" \
    || ! search_fixed 'shared_integration_not_registered' "$adapter_contract"; then
    mark_capability_contract_blocked 'invalid-rust-capability-availability-contract'
  fi
  if ! search_fixed 'LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1' "$web_contract" \
    || ! search_fixed 'parseLocalCapabilityAvailability' "$web_contract"; then
    mark_capability_contract_blocked 'invalid-web-capability-availability-contract'
  fi
fi

graph_diff_count=0
graph_diff_application="$repo_root/crates/diff-engine/src/application.rs"
if [[ -f "$graph_diff_application" && ! -L "$graph_diff_application" ]]; then
  graph_diff_count="$(count_matches 'GraphDiff::between' "$graph_diff_application")"
fi

if [[ "$graph_diff_count" != '1' ]]; then
  mark_local_contract_blocked "graph-diff-calculator-count:$graph_diff_count"
  graph_diff_status=blocked
else
  graph_diff_status=passed
fi

public_write_status=unobserved
secret_reading_status=unobserved
if [[ "$wave1_output" = *'change_set=blocked'* ]]; then
  public_write_status=blocked
  secret_reading_status=blocked
elif [[ "$#" -eq 2 && "$wave1_output" = *'change_set=passed'* ]]; then
  public_write_status=passed
  secret_reading_status=passed
  while IFS=$'\t' read -r changed_path added_line; do
    [[ -n "$changed_path" ]] || continue
    case "$changed_path" in
      server/api/*)
        if [[ "$added_line" =~ (routing::(post|put|patch|delete)|(post|put|patch|delete)\() ]]; then
          public_write_status=blocked
          printf 'public_write_detail=blocked path=%s\n' "$changed_path" >&2
        fi
        ;;
      docs/api/*|*openapi*.json|*openapi*.yaml|*openapi*.yml)
        if [[ "$added_line" =~ \"(post|put|patch|delete)\"[[:space:]]*: ]]; then
          public_write_status=blocked
          printf 'public_write_detail=blocked path=%s\n' "$changed_path" >&2
        fi
        ;;
      packages/ts-sdk/*)
        if [[ "$added_line" =~ (async[[:space:]]+(create|update|delete)|method[[:space:]]*:[[:space:]]*\"(POST|PUT|PATCH|DELETE)\") ]]; then
          public_write_status=blocked
          printf 'public_write_detail=blocked path=%s\n' "$changed_path" >&2
        fi
        ;;
    esac

    if [[ "$added_line" =~ (std::env([:][:]var)?|process[.]env|dotenv|[.]env|DATABASE_URL|[A-Z0-9_]*(PASSWORD|SECRET|API_KEY|ACCESS_KEY|PRIVATE_KEY|TOKEN)) ]]; then
      secret_reading_status=blocked
      printf 'secret_reading_detail=blocked path=%s\n' "$changed_path" >&2
    fi
  done < <(awk '
    /^\+\+\+ b\// {
      path=substr($0, 7)
      next
    }
    /^\+\+\+/ {
      next
    }
    /^\+/ && path != "" {
      print path "\t" substr($0, 2)
    }
  ' "$2")
fi

printf 'wave2_local_contracts=%s\n' "$local_contract_status"
printf 'graph_diff_calculators=%s count=%s\n' "$graph_diff_status" "$graph_diff_count"
printf 'capability_availability_contract=%s\n' "$capability_contract_status"
printf 'public_write_additions=%s\n' "$public_write_status"
printf 'secret_reading_additions=%s\n' "$secret_reading_status"
printf 'external_release_evidence=deferred reason=outside-local-engineering-scope\n'

if [[ "$wave1_status" -ne 0 || "$local_contract_status" = blocked || "$public_write_status" = blocked || "$secret_reading_status" = blocked ]]; then
  printf 'overall=blocked\n'
  exit 1
fi

if [[ "$wave1_output" = *'overall=unobserved'* ]]; then
  printf 'overall=unobserved\n'
else
  printf 'overall=passed\n'
fi
