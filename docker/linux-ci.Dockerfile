FROM ubuntu:22.04 AS base

ENV CARGO_HOME=/opt/cargo \
    RUSTUP_HOME=/opt/rustup \
    PATH=/opt/cargo/bin:/usr/local/bin:${PATH}

COPY docker/linux-ci.sh /usr/local/bin/opentranscribe-linux-ci
RUN chmod +x /usr/local/bin/opentranscribe-linux-ci \
    && opentranscribe-linux-ci setup

WORKDIR /workspace

COPY apps/desktop/package.json apps/desktop/package-lock.json apps/desktop/
RUN npm ci --prefix apps/desktop

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY apps/desktop/src-tauri/Cargo.toml apps/desktop/src-tauri/Cargo.toml
COPY crates/domain/Cargo.toml crates/domain/Cargo.toml
COPY crates/transcriber-protocol/Cargo.toml crates/transcriber-protocol/Cargo.toml
COPY sidecars/local-transcriber/Cargo.toml sidecars/local-transcriber/Cargo.toml
RUN cargo fetch --locked

FROM base AS workspace

COPY . .

FROM workspace AS native-test
RUN bash scripts/run-linux-ci.sh native

FROM workspace AS quality
RUN bash scripts/run-linux-ci.sh quality

FROM workspace AS frontend-test
RUN bash scripts/run-linux-ci.sh frontend

FROM workspace AS browser-test
RUN bash scripts/run-linux-ci.sh browser

FROM workspace AS release
RUN bash scripts/run-linux-ci.sh release
