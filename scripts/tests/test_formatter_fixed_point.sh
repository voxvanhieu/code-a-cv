#!/usr/bin/env bash
# The formatter fixed point (Q1): cac fmt output is a no-op under Prettier
# and markdownlint for the Markdown examples. Pinned tool versions live in
# package.json; run with `npm ci && npm run format:check` or through CI.
set -euo pipefail

cd "$(dirname "$0")/../.."

if [[ -x target/debug/cac ]]; then
  CACC="./target/debug/cac"
else
  CACC="cargo run --quiet --bin cac --"
fi
EXAMPLES=(basic-markdown complete-cv format-conversion job-specific-cv json-resume-import)

for example in "${EXAMPLES[@]}"; do
  # The CV source, not README prose (READMEs are plain Markdown documents).
  md_file=$(find "docs/examples/$example" -maxdepth 1 -name 'Ada_Lovelace*.md' | head -1)
  "$CACC" fmt "$md_file"
done

npx --yes prettier@3.6.2 --check \
  docs/examples/basic-markdown/Ada_Lovelace.md \
  docs/examples/complete-cv/Ada_Lovelace.md \
  docs/examples/format-conversion/Ada_Lovelace.md \
  docs/examples/job-specific-cv/Ada_Lovelace_JobSpecific.md \
  docs/examples/json-resume-import/Ada_Lovelace.md

npx --yes markdownlint-cli2@0.18.1 \
  --config .markdownlint.jsonc \
  docs/examples/*/Ada_Lovelace*.md

echo "PASS formatter fixed point"
