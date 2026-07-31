# OpenTranscribe

OpenTranscribe is a local-first desktop meeting workspace. It records audio to
readable files you control, keeps notes beside each session, and can transcribe
completed recordings locally or with an OpenAI API key.

OpenAI API keys are stored in the native macOS Keychain, Windows Credential
Manager, or Linux Secret Service vault. See
[`docs/credentials.md`](docs/credentials.md) for storage details and macOS
development prompt behavior.

The project is under active development. It includes microphone and
system-output capture adapters for macOS, Windows, and Linux; separate source
tracks and a mixed playback file; crash-recovery chunks; tray controls; durable
transcription jobs; timestamped notes; local whisper.cpp transcription; OpenAI
file transcription; editable transcripts and speakers; full-text library
search; cached waveform playback; recoverable trash; media import; and Markdown,
text, JSON, SRT, and VTT export. First run configures the library, capture
sources, and default transcription path before a real ten-second audio test.

macOS capture is the current physically tested development path. Windows WASAPI
loopback and Linux PipeWire capture compile in the platform CI matrix but still
need the hardware, interruption, and duration validation in
`docs/platform-validation.md`. OpenAI live transcription is implemented; local
transcription runs after the recording is durably finalized so model latency
cannot interrupt capture.

Pull requests run the shared native tests on macOS, Windows, and Linux plus
compile checks for every supported release triple. Trusted `main` and nightly
runs also retain unsigned installers for seven days so current packages can be
smoke-tested before signing is configured.

## Stack

- Tauri 2 and Rust for capture, storage, credentials, and providers
- Vue 3, TypeScript, Pinia, Vue Router, and PrimeVue for the interface
- SQLite FTS as a rebuildable index; JSON, Markdown, and audio files remain the
  durable source of truth
- A versioned MessagePack protocol for the local transcription sidecar

## Development

Requirements:

- Node.js 22 or newer
- Rust 1.92
- CMake 3.20 or newer for the bundled whisper.cpp sidecar
- Tauri's operating-system prerequisites
- Linux builds also need PipeWire 1.0 or newer development headers
- Go Task is optional; every task delegates to npm or Cargo

```bash
npm ci
npm run tauri dev
```

Useful checks:

```bash
task check
task build
task build:native
APPLE_SIGNING_IDENTITY="Developer ID Application: …" task build:signed:macos
npm run icons
```

`npm run tauri dev` is the single hot-reload entrypoint. Stop that process
before starting another instance, and quit any installed or release-bundle
copy of OpenTranscribe, so permission prompts, tray state, and the visible
window stay attached to the development process.

`npm run icons` regenerates every native application icon and the dedicated
monochrome system-tray icon from the canonical artwork in `src/assets/`.

The signed macOS task requires an installed Developer ID Application identity,
builds the app and DMG with that identity, and rejects an ad-hoc signature.
See [docs/macos-signing.md](docs/macos-signing.md) for local and CI setup.

Without Task:

```bash
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

## Data and privacy

OpenTranscribe does not require an OpenTranscribe account. A selected library
contains readable projects and sessions. OpenAI transcription is opt-in per
session. The API key is entered in the Settings webview, passed directly to the
native credential command, and never written to frontend persistence or the
library. See [PRIVACY.md](PRIVACY.md).

Recording laws and policies vary. Obtain the required participant consent
before recording.

## Licenses

Unless a file says otherwise, this project is available under either the
Apache License 2.0 or the MIT License, at your option.
