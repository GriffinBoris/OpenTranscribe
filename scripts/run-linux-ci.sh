#!/usr/bin/env bash

set -euo pipefail

mode="${1:?Choose native, quality, frontend, browser, or release}"

build_frontend_and_sidecar() {
  npm --prefix apps/desktop run build
  npm --prefix apps/desktop run prepare:sidecar:debug -- --target x86_64-unknown-linux-gnu
}

case "${mode}" in
  native)
    build_frontend_and_sidecar
    cargo test --workspace --locked
    ;;
  quality)
    git init --quiet
    git add --all
    python3 agents/build_agents.py --target opencode --out . --layout in-place
    python3 agents/build_agents.py --target claude --out . --layout in-place
    python3 agents/build_agents.py --target copilot --out . --layout in-place
    python3 agents/build_agents.py --target codex --out . --layout in-place
    python3 agents/build_agents.py --target gemini --out . --layout in-place
    build_frontend_and_sidecar
    npm --prefix apps/desktop run types:generate
    npm --prefix apps/desktop run format:check
    cargo fmt --all --check
    npm --prefix apps/desktop run lint
    cargo clippy --workspace --all-targets --locked -- -D warnings
    npm --prefix apps/desktop run typecheck
    git diff --exit-code
    ;;
  frontend)
    npm --prefix apps/desktop run test
    npm --prefix apps/desktop audit
    ;;
  browser)
    npx --prefix apps/desktop playwright install --with-deps chromium
    npm --prefix apps/desktop run test:ui
    ;;
  release)
    npm --prefix apps/desktop run tauri -- build --target x86_64-unknown-linux-gnu --bundles deb --ci --no-sign -- --locked
    node scripts/verify-linux-glibc.mjs 2.35 \
      target/x86_64-unknown-linux-gnu/release/opentranscribe \
      apps/desktop/src-tauri/binaries/local-transcriber-x86_64-unknown-linux-gnu
    ;;
  *)
    echo "Unknown Linux CI mode: ${mode}" >&2
    exit 2
    ;;
esac
