#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$BASH_SOURCE")" && pwd)"
if [[ "$#" -ge 1 ]]; then
  repo_root="$1"
else
  repo_root="$script_dir/.."
fi

if [[ "$#" -gt 2 || ! -d "$repo_root" ]]; then
  printf 'usage: verify-wave1-parallel-contract.sh [repo-root] [safe-diff-file]\n' >&2
  exit 1
fi

repo_root="$(cd "$repo_root" && pwd)"

is_secret_path() {
  local path="$1"
  local base

  base="$(basename "$path")"
  case "$base" in
    .env|.env.*|*.pem|*.key|*.p12|*.pfx)
      return 0
      ;;
  esac

  case "/$path/" in
    */secrets/*)
      return 0
      ;;
  esac

  return 1
}

is_wave1_path() {
  case "$1" in
    crates/evaluation/*|crates/diff-engine/*|crates/workflow/*|crates/knowledge/*|crates/memory/*|crates/mcp/*|crates/plugin-runtime/*|apps/cli/*|apps/desktop/*|apps/web/*|packages/ui/*|packages/design-system/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

workspace_status=passed
rust_member_count=0
package_directory_count=0
package_manifest_count=0

mark_workspace_blocked() {
  workspace_status=blocked
  printf 'workspace_members_detail=blocked reason=%s\n' "$1" >&2
}

cargo_manifest="$repo_root/Cargo.toml"
if [[ ! -f "$cargo_manifest" || -L "$cargo_manifest" ]]; then
  mark_workspace_blocked 'missing-or-unsafe-Cargo.toml'
else
  cargo_members="$(awk '
    {
      if (!in_members && $0 ~ /^[[:space:]]*members[[:space:]]*=/) {
        in_members=1
        line=$0
        sub(/^.*\[/, "", line)
      } else if (in_members) {
        line=$0
      }
      if (in_members) {
        while (match(line, /"[^"]+"/)) {
          value=substr(line, RSTART + 1, RLENGTH - 2)
          print value
          line=substr(line, RSTART + RLENGTH)
        }
        if (index(line, "]")) {
          exit
        }
      }
    }
  ' "$cargo_manifest")"

  if [[ -z "$cargo_members" ]]; then
    mark_workspace_blocked 'Cargo.toml-declares-no-members'
  else
    while IFS= read -r member; do
      [[ -n "$member" ]] || continue
      rust_member_count=$((rust_member_count + 1))
      case "$member" in
        /*|../*|*/../*|*\\*)
          mark_workspace_blocked "unsafe-member-path:$member"
          continue
          ;;
      esac
      member_path="$repo_root/$member"
      if [[ -L "$member_path" || ! -d "$member_path" || ! -f "$member_path/Cargo.toml" || -L "$member_path/Cargo.toml" ]]; then
        mark_workspace_blocked "missing-member-manifest:$member"
      fi
    done <<< "$cargo_members"
  fi
fi

pnpm_workspace="$repo_root/pnpm-workspace.yaml"
if [[ ! -f "$pnpm_workspace" || -L "$pnpm_workspace" ]]; then
  mark_workspace_blocked 'missing-or-unsafe-pnpm-workspace.yaml'
else
  pnpm_patterns="$(awk '
    /^[[:space:]]*-[[:space:]]*"/ {
      line=$0
      sub(/^[^"]*"/, "", line)
      sub(/".*$/, "", line)
      print line
    }
  ' "$pnpm_workspace")"

  if [[ -z "$pnpm_patterns" ]]; then
    mark_workspace_blocked 'pnpm-workspace.yaml-declares-no-patterns'
  else
    while IFS= read -r pattern; do
      [[ -n "$pattern" ]] || continue
      pattern_matches="$(compgen -G "$repo_root/$pattern" || true)"
      if [[ -z "$pattern_matches" ]]; then
        mark_workspace_blocked "workspace-pattern-matches-no-directory:$pattern"
        continue
      fi

      while IFS= read -r candidate; do
        [[ -d "$candidate" ]] || continue
        package_directory_count=$((package_directory_count + 1))
        relative_candidate="$(printf '%s\n' "$candidate" | sed "s|^$repo_root/||")"
        if is_secret_path "$relative_candidate" || [[ -L "$candidate" ]]; then
          mark_workspace_blocked "unsafe-package-directory:$relative_candidate"
        elif [[ -f "$candidate/package.json" ]]; then
          package_manifest_count=$((package_manifest_count + 1))
        else
          printf 'workspace_package_detail=ignored reason=no-package-manifest path=%s\n' "$relative_candidate" >&2
        fi
      done <<< "$pattern_matches"
    done <<< "$pnpm_patterns"
  fi
fi

change_set_status=unobserved
diff_source_available=0
diff_input=''
secret_path_addition=0
temporary_diff=''

cleanup() {
  if [[ -n "$temporary_diff" ]]; then
    rm -f "$temporary_diff"
  fi
}

trap cleanup EXIT

collect_git_diff_mode() {
  local mode="$1"
  local changed_paths
  local changed_path

  if [[ "$mode" = cached ]]; then
    changed_paths="$(git -C "$repo_root" diff --cached --name-only -- crates/evaluation crates/diff-engine crates/workflow crates/knowledge crates/memory crates/mcp crates/plugin-runtime apps/cli apps/desktop apps/web packages/ui packages/design-system 2>/dev/null || true)"
  else
    changed_paths="$(git -C "$repo_root" diff --name-only -- crates/evaluation crates/diff-engine crates/workflow crates/knowledge crates/memory crates/mcp crates/plugin-runtime apps/cli apps/desktop apps/web packages/ui packages/design-system 2>/dev/null || true)"
  fi

  while IFS= read -r changed_path; do
    [[ -n "$changed_path" ]] || continue
    if is_secret_path "$changed_path"; then
      secret_path_addition=1
      continue
    fi
    is_wave1_path "$changed_path" || continue
    if [[ "$mode" = cached ]]; then
      git -C "$repo_root" diff --cached --no-ext-diff --no-color --unified=0 -- "$changed_path" >>"$temporary_diff"
    else
      git -C "$repo_root" diff --no-ext-diff --no-color --unified=0 -- "$changed_path" >>"$temporary_diff"
    fi
  done <<< "$changed_paths"
}

if [[ "$#" -ge 2 ]]; then
  diff_input="$2"
  if is_secret_path "$diff_input"; then
    change_set_status=blocked
    printf 'change_set_detail=blocked reason=refusing secret-looking input path\n' >&2
  elif [[ ! -f "$diff_input" || -L "$diff_input" ]]; then
    change_set_status=blocked
    printf 'change_set_detail=blocked reason=missing-or-unsafe-diff-file\n' >&2
  else
    change_set_status=passed
    diff_source_available=1
  fi
elif command -v git >/dev/null 2>&1 && [[ "$(git -C "$repo_root" rev-parse --is-inside-work-tree 2>/dev/null || true)" = true ]]; then
  temporary_diff="$(mktemp)"
  collect_git_diff_mode working
  collect_git_diff_mode cached
  untracked_paths="$(git -C "$repo_root" ls-files --others --exclude-standard -- crates/evaluation crates/diff-engine crates/workflow crates/knowledge crates/memory crates/mcp crates/plugin-runtime apps/cli apps/desktop apps/web packages/ui packages/design-system 2>/dev/null || true)"
  while IFS= read -r untracked_path; do
    [[ -n "$untracked_path" ]] || continue
    if is_secret_path "$untracked_path"; then
      secret_path_addition=1
      continue
    fi
    is_wave1_path "$untracked_path" || continue
    untracked_full_path="$repo_root/$untracked_path"
    if [[ -L "$untracked_full_path" || ! -f "$untracked_full_path" ]]; then
      change_set_status=blocked
      printf 'change_set_detail=blocked reason=unsafe-untracked-path:%s\n' "$untracked_path" >&2
      continue
    fi
    {
      printf '%s\n' "diff --git a/$untracked_path b/$untracked_path"
      printf '%s\n' "--- /dev/null" "+++ b/$untracked_path" '@@'
      sed 's/^/+/' "$untracked_full_path"
    } >>"$temporary_diff"
  done <<< "$untracked_paths"
  if [[ "$change_set_status" != blocked ]]; then
    change_set_status=passed
  fi
  diff_input="$temporary_diff"
  diff_source_available=1
else
  printf 'change_set_detail=unobserved reason=no-safe-local-diff-available\n' >&2
fi

if [[ "$secret_path_addition" -eq 1 ]]; then
  change_set_status=blocked
fi

public_write_findings=0
secret_read_findings=0
public_write_regex="(export[[:space:]]+(async[[:space:]]+)?function[[:space:]]+(POST|PUT|PATCH|DELETE))|(method[[:space:]]*:[[:space:]]*[\"'](POST|PUT|PATCH|DELETE)[\"'])|(\\.(post|put|patch|delete)[[:space:]]*\\()|(route[[:space:]]*\\([^)]*(post|put|patch|delete)[[:space:]]*\\()|(/api/v1/[^\"']*(create|update|delete|write))"
secret_read_regex="(std::env([:][:]var)?|process[.]env|dotenv|[.]env|DATABASE_URL|[A-Z0-9_]*(PASSWORD|SECRET|API_KEY|ACCESS_KEY|PRIVATE_KEY|TOKEN))"

if [[ "$diff_source_available" -eq 1 && "$change_set_status" != blocked ]]; then
  while IFS=$'\t' read -r changed_path added_line; do
    [[ -n "$changed_path" ]] || continue
    if is_secret_path "$changed_path"; then
      secret_path_addition=1
      continue
    fi
    is_wave1_path "$changed_path" || continue
    if [[ "$added_line" =~ $public_write_regex ]]; then
      public_write_findings=$((public_write_findings + 1))
      printf 'public_write_detail=blocked path=%s\n' "$changed_path" >&2
    fi
    if [[ "$added_line" =~ $secret_read_regex ]]; then
      secret_read_findings=$((secret_read_findings + 1))
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
  ' "$diff_input")
fi

if [[ "$change_set_status" = blocked ]]; then
  public_status=blocked
  secret_status=blocked
elif [[ "$diff_source_available" -eq 0 ]]; then
  public_status=unobserved
  secret_status=unobserved
else
  public_status=passed
  secret_status=passed
  if [[ "$public_write_findings" -gt 0 ]]; then
    public_status=blocked
  fi
  if [[ "$secret_read_findings" -gt 0 || "$secret_path_addition" -eq 1 ]]; then
    secret_status=blocked
  fi
fi

printf 'workspace_members=%s rust=%s package_directories=%s package_manifests=%s\n' "$workspace_status" "$rust_member_count" "$package_directory_count" "$package_manifest_count"
printf 'change_set=%s\n' "$change_set_status"
printf 'public_write_additions=%s\n' "$public_status"
printf 'secret_reading_additions=%s\n' "$secret_status"
printf 'docker_runtime=ignored reason=offline-only\n'
printf 'network_access=ignored reason=offline-only\n'

if [[ "$workspace_status" = blocked || "$change_set_status" = blocked || "$public_status" = blocked || "$secret_status" = blocked ]]; then
  printf 'overall=blocked\n'
  exit 1
fi

if [[ "$diff_source_available" -eq 0 ]]; then
  printf 'overall=unobserved\n'
else
  printf 'overall=passed\n'
fi
