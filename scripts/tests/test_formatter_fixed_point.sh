#!/usr/bin/env bash
# The formatter fixed point (Q1): cac fmt output is a no-op under Prettier
# and markdownlint for the Markdown examples. Pinned tool versions live in
# package.json; run with `npm ci && npm run format:check` or through CI.
set -euo pipefail

cd "$(dirname "$0")/../.."

# The debug binary from `cargo build --package cac`, or cac.exe on
# Windows-style checkouts. If neither exists, build it once and re-probe.
CACC=()
if [[ -x target/debug/cac ]]; then
  CACC=(target/debug/cac)
elif [[ -x target/debug/cac.exe ]]; then
  CACC=(target/debug/cac.exe)
else
  cargo build --locked --package cac
  if [[ -x target/debug/cac ]]; then
    CACC=(target/debug/cac)
  elif [[ -x target/debug/cac.exe ]]; then
    CACC=(target/debug/cac.exe)
  fi
fi
if [[ "${#CACC[@]}" -eq 0 ]]; then
  echo "cargo build did not produce target/debug/cac" >&2
  exit 1
fi
EXAMPLES=(basic-markdown complete-cv format-conversion job-specific-cv json-resume-import)

for example in "${EXAMPLES[@]}"; do
  # The CV source, not README prose (READMEs are plain Markdown documents).
  md_file=$(find "docs/examples/$example" -maxdepth 1 -name 'Ada_Lovelace*.md' | head -1)
  if [[ -z "${md_file}" ]]; then
    echo "no Markdown CV source under docs/examples/$example" >&2
    exit 1
  fi
  # Read-only assertion: --dry-run fails when the committed file is not
  # already at the fixed point, so the check never rewrites the repository.
  "${CACC[@]}" fmt --dry-run "${md_file}"
done

# The versions are installed by `npm ci` from the lockfile; --no-install
# refuses to reach the network or resolve anything else.
npx --no-install prettier@3.6.2 --check \
  docs/examples/basic-markdown/Ada_Lovelace.md \
  docs/examples/complete-cv/Ada_Lovelace.md \
  docs/examples/format-conversion/Ada_Lovelace.md \
  docs/examples/job-specific-cv/Ada_Lovelace_JobSpecific.md \
  docs/examples/json-resume-import/Ada_Lovelace.md

npx --no-install markdownlint-cli2@0.18.1 \
  --config .markdownlint.jsonc \
  docs/examples/*/Ada_Lovelace*.md

echo "PASS formatter fixed point"
