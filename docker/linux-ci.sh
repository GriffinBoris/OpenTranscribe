#!/usr/bin/env bash

set -euo pipefail

node_version="22.23.1"
rust_toolchain="1.97.1"

setup() {
  export DEBIAN_FRONTEND=noninteractive

  apt-get update
  apt-get install --yes --no-install-recommends \
    build-essential \
    ca-certificates \
    cmake \
    curl \
    file \
    git \
    libappindicator3-dev \
    libasound2-dev \
    libclang-dev \
    libgtk-3-dev \
    libpipewire-0.3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    patchelf \
    pkg-config \
    python3

  case "$(dpkg --print-architecture)" in
    amd64) node_architecture="x64" ;;
    arm64) node_architecture="arm64" ;;
    *)
      echo "Unsupported Node.js architecture: $(dpkg --print-architecture)" >&2
      exit 2
      ;;
  esac

  curl --fail --location --silent --show-error \
    "https://nodejs.org/dist/v${node_version}/node-v${node_version}-linux-${node_architecture}.tar.xz" \
    | tar --extract --xz --strip-components=1 --directory /usr/local

  curl --proto '=https' --tlsv1.2 --silent --show-error --fail https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain "${rust_toolchain}"
}

copy_source() {
  mkdir --parents /workspace
  tar --create --file - \
    --directory /source \
    --exclude=.agents \
    --exclude=.codex \
    --exclude=.git \
    --exclude=node_modules \
    --exclude=apps/desktop/node_modules \
    --exclude=apps/desktop/dist \
    --exclude=apps/desktop/playwright-report \
    --exclude=apps/desktop/test-results \
    --exclude=apps/desktop/src-tauri/binaries/local-transcriber-* \
    --exclude=target \
    . | tar --extract --file - --directory /workspace
}

run() {
  export CARGO_HOME=/opt/cargo
  export RUSTUP_HOME=/opt/rustup
  export PATH="${CARGO_HOME}/bin:/usr/local/bin:${PATH}"

  copy_source
  cd /workspace
  npm ci --prefix apps/desktop

  case "${1:?Choose a Linux CI mode}" in
    native-test) mode="native" ;;
    quality) mode="quality" ;;
    frontend-test) mode="frontend" ;;
    browser-test) mode="browser" ;;
    release) mode="release" ;;
    *)
      echo "Unknown Linux CI mode: $1" >&2
      exit 2
      ;;
  esac

  bash scripts/run-linux-ci.sh "${mode}"
}

case "${1:?Choose setup or run}" in
  setup)
    export CARGO_HOME=/opt/cargo
    export RUSTUP_HOME=/opt/rustup
    export PATH="${CARGO_HOME}/bin:/usr/local/bin:${PATH}"
    setup
    ;;
  run)
    run "${2:?Choose a Linux CI mode}"
    ;;
  *)
    echo "Unknown Linux CI command: $1" >&2
    exit 2
    ;;
esac
