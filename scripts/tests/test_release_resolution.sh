#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
resolver="$script_dir/../resolve-published-release.sh"
test_root=$(mktemp -d)
trap 'rm -rf -- "$test_root"' EXIT

mkdir "$test_root/bin"
cat > "$test_root/bin/gh" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
case "$1 $2" in
  'run download')
    test "$3" = 123
    mkdir plan
    printf '%s\n' "$PLAN" > plan/plan-dist-manifest.json
    ;;
  'release view')
    printf '%s\n' "$RELEASE"
    ;;
  *) exit 99 ;;
esac
MOCK
chmod +x "$test_root/bin/gh"

run_case() (
  name=$1
  expected=$2
  shift 2
  mkdir "$test_root/$name"
  cd "$test_root/$name"

  export PATH="$test_root/bin:$PATH"
  export EVENT_NAME=workflow_run REQUESTED_TAG=v0.3.0
  export RELEASE_RUN_EVENT=push RELEASE_RUN_RESULT=success
  export RELEASE_RUN_REPOSITORY=owner/repo RELEASE_RUN_ID=123
  export GITHUB_REPOSITORY=owner/repo GITHUB_REF=refs/heads/main
  export GITHUB_OUTPUT="$PWD/output"
  prerelease=false
  draft=false
  for setting in "$@"; do
    export "$setting"
  done
  PLAN=$(jq -n --arg tag "$REQUESTED_TAG" --argjson prerelease "$prerelease" \
    '{announcement_tag: $tag, announcement_is_prerelease: $prerelease}')
  RELEASE=$(jq -n --arg tag "$REQUESTED_TAG" --argjson prerelease "$prerelease" \
    --argjson draft "$draft" '{tagName: $tag, isPrerelease: $prerelease, isDraft: $draft}')
  export PLAN RELEASE

  status=0
  bash "$resolver" > log 2>&1 || status=$?
  touch "$GITHUB_OUTPUT"
  case "$expected" in
    publish)
      printf 'tag=%s\n' "$REQUESTED_TAG" > expected
      if [[ "$status" -eq 0 ]] && cmp -s expected "$GITHUB_OUTPUT"; then
        printf 'PASS %s\n' "$name"
        exit 0
      fi
      ;;
    skip)
      if [[ "$status" -eq 0 && ! -s "$GITHUB_OUTPUT" ]]; then
        printf 'PASS %s\n' "$name"
        exit 0
      fi
      ;;
    reject)
      if [[ "$status" -ne 0 && ! -s "$GITHUB_OUTPUT" ]]; then
        printf 'PASS %s\n' "$name"
        exit 0
      fi
      ;;
  esac
  printf 'FAIL %s: expected %s, exit status %s\n' "$name" "$expected" "$status" >&2
  cat log "$GITHUB_OUTPUT" >&2
  exit 1
)

run_case successful_release_resolves_exact_run_tag publish
run_case prerelease_skips_publication skip REQUESTED_TAG=v0.3.0-beta.1 prerelease=true
run_case failed_run_cannot_publish reject RELEASE_RUN_RESULT=failure
run_case pull_request_cannot_publish reject RELEASE_RUN_EVENT=pull_request
run_case foreign_run_cannot_publish reject RELEASE_RUN_REPOSITORY=fork/repo
run_case leading_zero_tag_cannot_publish reject REQUESTED_TAG=v01.2.3
run_case multiline_tag_cannot_publish reject REQUESTED_TAG=$'v1.2.3\ntag=other'
run_case latest_tag_cannot_publish reject REQUESTED_TAG=latest
run_case draft_cannot_publish reject draft=true
run_case manual_retry_from_main publish EVENT_NAME=workflow_dispatch
run_case manual_retry_requires_main reject EVENT_NAME=workflow_dispatch GITHUB_REF=refs/heads/other

printf 'All 11 release resolution cases passed.\n'
