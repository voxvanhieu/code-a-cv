#!/usr/bin/env bash
set -euo pipefail
tag="$REQUESTED_TAG"
if [[ "$EVENT_NAME" == workflow_run ]]; then
  test "$RELEASE_RUN_EVENT" = push
  test "$RELEASE_RUN_RESULT" = success
  test "$RELEASE_RUN_REPOSITORY" = "$GITHUB_REPOSITORY"
  gh run download "$RELEASE_RUN_ID" --repo "$GITHUB_REPOSITORY" \
    --name artifacts-plan-dist-manifest --dir plan
  if jq -e '.announcement_is_prerelease == true' plan/plan-dist-manifest.json; then
    echo 'Skipping prerelease publication.'
    exit 0
  fi
  tag=$(jq -er '.announcement_tag' plan/plan-dist-manifest.json)
else
  test "$EVENT_NAME" = workflow_dispatch
  test "$GITHUB_REF" = refs/heads/main
fi
if [[ ! "$tag" =~ ^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  echo 'Expected a stable release tag such as v0.3.0.' >&2
  exit 1
fi
gh release view "$tag" --repo "$GITHUB_REPOSITORY" \
  --json isDraft,isPrerelease,tagName > release.json
jq -e --arg tag "$tag" \
  '.isDraft == false and .isPrerelease == false and .tagName == $tag' release.json
echo "tag=$tag" >> "$GITHUB_OUTPUT"
