---
id: project-guidance
title: OpenTranscribe Project Guidance
description: Repository-specific architecture, privacy, storage, and verification decisions for OpenTranscribe.
kind: guidance
scope: project
name: project
tags:
  - project
  - tauri
  - rust
  - vue
status: active
order: 0
---

# OpenTranscribe Project Guidance

## Purpose

- OpenTranscribe is a local-first Tauri desktop application for recording, transcribing, organizing, and exporting meetings.
- Keep durable cross-project rules in the global, Rust, and Vue guidance. Keep product and repository decisions here.
- Update this authored file and regenerate agent targets instead of editing generated guidance.

## Repository Layout

- `src/` contains the Vue application. Route views live in `src/views/`; shared controls live in `src/components/`.
- `src-tauri/` contains the desktop process and thin Tauri command adapters.
- `crates/domain/` owns persisted manifests, IDs, transcript types, jobs, and shared DTOs.
- `crates/transcriber-protocol/` owns the versioned MessagePack protocol shared with the local transcriber.
- `sidecars/local-transcriber/` owns whisper.cpp model lifecycle and inference.
- `tests/fixtures/` contains non-sensitive audio, provider, migration, and recovery fixtures.
- Keep files focused. Split modules before they become mixed command, persistence, capture, and provider catch-alls.

## Product Boundaries

- The selected library directory is the durable source of truth. SQLite is a rebuildable index and queue accelerator.
- Content remains readable without the application: JSON manifests, Markdown
  notes, 24-bit PCM WAV source/mix tracks, and JSON transcripts. A future
  compressed codec migration must preserve the readable library contract and
  cannot happen implicitly during recording.
- Use ULIDs for persisted identities, RFC 3339 UTC for wall-clock timestamps, and integer milliseconds for media positions.
- API keys live only in the operating-system credential vault. Never put them in application settings, the library, logs, diagnostics, tests, or screenshots.
- Do not add telemetry, remote crash reporting, hosted accounts, summaries, action items, translation, cloud sync, or collaboration without a new product decision.
- Recording reliability has priority over waveform generation, indexing, model inference, provider uploads, and exports.
- Never silently switch recording devices, providers, models, or local-model quality.

## Rust And Tauri Boundaries

- Tauri commands validate DTOs and call focused services. They do not contain storage, capture, or provider workflows.
- Rust owns files, SQLite, capture, jobs, credentials, providers, exports, and native process supervision.
- The Vue webview owns presentation and short-lived editing state only.
- Use commands for request/response work and one ordered channel for recording levels, transcript deltas, job progress, and attention events.
- Raw audio never crosses Tauri IPC.
- Platform system-audio adapters remain behind one trait:
  - ScreenCaptureKit on macOS
  - WASAPI loopback on Windows
  - PipeWire on Linux
- Keep the macOS deployment target at 14.0 in both Cargo and Tauri bundle
  configuration. The ScreenCaptureKit Rust adapter includes a Swift bridge, so
  `build.rs` derives the active toolchain's Swift runtime library path through
  `xcrun` instead of hardcoding an Xcode installation path.
- Local inference runs in the supervised sidecar. A sidecar failure must not stop or corrupt recording.
- Keep recording completion native-owned. Dock, tray, and global-shortcut stop
  actions must converge on the same finalization path, and any selected
  post-recording transcription starts only after durable audio finalization.
  Provider, credential, or model failures must leave the saved recording ready
  for manual retry and surface an attention event instead of turning recording
  completion into a failure.
- Build the local transcriber through `scripts/prepare-sidecar.mjs`. Tauri
  `externalBin` requires the copied executable to include the Rust target
  triple, and whisper.cpp builds require CMake 3.20 or newer. Host development
  builds ignore Tauri's target-triple environment variable unless `--target`
  is explicit so hot reload does not compile the sidecar for a second target.
- Keep Tauri, Vite, and Playwright on the same `127.0.0.1:1420` development
  origin. Vite must ignore `dist/` and Rust `target/` output so frontend hot
  reload is not interrupted by generated native-build files.
- Run one Tauri development process and quit any installed or release-bundle
  copy first. macOS treats both copies as the same product, which can attach
  permissions, tray state, and UI inspection to the wrong process.
- Keep the main window reopenable from the tray by treating its close control as
  hide-to-tray. User-initiated application quit must be refused while capture
  is active and bring the recording surface back into view; an explicit quit is
  allowed after recording has stopped.
- macOS privacy grants follow the app's designated code requirement. Ad-hoc
  rebuilds receive a new code hash and therefore do not retain Screen & System
  Audio Recording access even when an older OpenTranscribe entry remains
  enabled in Settings. Use one stable signing identity for repeatable capture
  validation, and report support, preflight permission, and capture startup as
  distinct states.
- Keep Tauri capabilities narrow. Do not expose an arbitrary shell, arbitrary executable paths, or broad filesystem scopes.

## Storage Rules

- Write JSON and Markdown through a same-directory temporary file, flush it, sync it, and atomically rename it.
- Check the caller's expected revision or hash before overwriting user-editable content.
- Preserve immutable transcription runs and provider artifacts. User edits affect only the canonical current transcript.
- Canonical transcript segments used for seeking and subtitle export must keep
  real model timestamps. Default cloud transcription to a response format that
  explicitly supplies timed segments; do not present a text-only response as
  precise timing.
- Recovery audio is written in sealed ten-second chunks. Delete recovery chunks only after final artifacts pass duration and checksum validation.
- Recording elapsed time represents captured-media time, not wall-clock time.
  Exclude paused intervals so the dock timer and timestamped notes stay aligned
  with final playback.
- External valid Markdown and JSON edits are reindexed while idle. Conflicts are surfaced instead of overwritten.
- Rebuild the SQLite title, notes, and transcript index from durable library
  files when a library opens. Search results may use the index for speed but
  must resolve timestamps and session identity from durable manifests.
- Cache waveform peaks as a small JSON artifact after audio finalization. A
  missing waveform must not make recorded audio or playback unavailable.
- When moving a session between Inbox and a project, move the complete session
  directory, update its project assignment and revision, rebase every
  library-relative artifact path, and refresh the rebuildable index. A failed
  manifest rewrite must move the directory back so readable files do not
  disagree with their location.
- Trash is app-managed, recoverable, and never automatically purged.
- Schema changes require sequential migrations and fixtures covering the previous schema.

## Frontend Conventions

- The application shell owns the sidebar, recording status, theme, bootstrap
  state, and router outlet. Keep cross-route recording and credential workflows
  in focused shell-level stores beside the application store instead of growing
  one catch-all store.
- Route folders own their views, local components, and local stores. Avoid a global catch-all store.
- Wrap PrimeVue primitives in app-owned components under `src/components/ui/`.
- Keep the OpenTranscribe application, installer, Dock/taskbar, sidebar, and
  tray icons derived from the canonical artwork in `src/assets/`. Use the
  full-color application tile for branded surfaces and the matching transparent
  monochrome mark for system tray or menu-bar template surfaces. Functional UI
  icons use Lucide so stroke weight and optical sizing remain consistent.
- Render interactive form controls through those app-owned PrimeVue wrappers.
  Raw `button`, `dialog`, `input`, `select`, and `textarea` elements are linted
  as errors so focus behavior, overlays, disabled states, and visual treatment
  stay consistent across the desktop app.
- Use semantic tokens from `src/styles/tokens.css`; do not hardcode theme colors
  in components. Reuse the shared typography, spacing, control-size, radius,
  shadow, motion, opacity, layering, and shell-layout scales whenever a value
  participates in app-wide visual consistency. Keep intrinsic media dimensions,
  responsive breakpoints, and genuinely component-specific geometry local
  instead of creating a token for every pixel value.
- Keep global CSS ordered through `src/styles/main.css`, with base, shared
  component, and focused view styles split into named files before any one
  stylesheet becomes a catch-all.
- Keep the live transcript read-only while recording. Notes and markers remain editable.
- Every provider action displays whether audio stays local or is sent to OpenAI.
- All strings go through Vue I18n even though the first release ships only English.
- Maintain keyboard navigation, visible focus, reduced motion, and WCAG 2.2 AA contrast.
- Keep the desktop shell compact and product-first: one quiet sidebar, one inset
  content pane, and a macOS overlay title bar that preserves native traffic
  lights. Use the dedicated non-interactive top strip for window dragging and
  grant only `core:window:allow-start-dragging` for that behavior.
- Keep global recording shortcuts opt-in and limited to validated presets.
  Register them through the typed native bridge with narrow plugin capabilities,
  act only on press events, serialize record/stop handling, and surface
  registration conflicts in Settings.
- Route pages fill the available main pane with adaptive horizontal gutters.
  Do not center the whole utility workspace inside a fixed desktop max-width;
  constrain only genuinely prose-heavy content when readability requires it.
- Prefer hierarchy, alignment, spacing, and contrasting backgrounds over
  repeated bordered cards. Avoid marketing-style heroes, decorative dashboard
  ornaments, redundant privacy claims, and equal visual weight for primary and
  secondary actions.
- Reuse one compact empty-state pattern across library routes. Pair a quiet
  Lucide icon with a short title and one actionable sentence instead of leaving
  an oversized blank surface or rendering an empty table header.
- First-run audio testing uses the normal recorder, finalizer, and playback
  path. Keep it compact, request only the permissions needed by selected
  sources, and show recording-consent guidance before the test.
- Treat adapter support and operating-system permission as separate states.
  Do not label a capture source ready merely because its adapter is available;
  give platform-specific recovery guidance when the app can open the relevant
  permission settings, and hide recovery actions once permission is granted.
  Preserve the native adapter's failure context instead of rewriting every
  startup failure as a permission denial.
- Run automated WCAG A/AA scans across primary routes in both light and dark
  themes. Fix repeated contrast failures in semantic tokens instead of applying
  local component overrides.
- Browser workflow fixtures must fail on uncaught page errors and console
  errors. PrimeVue portal and pass-through failures can leave enough DOM in
  place for a visual assertion to pass while the native webview is broken.

## Local And CI Commands

```bash
task install
task dev
task format
task format:check
task lint
task typecheck
task test
task check
task build
```

- CI uses `npm ci` and Cargo `--locked`.
- Pull requests receive no API keys or release secrets.
- Run `cargo fmt --check`, Clippy with warnings denied, Rust tests, ESLint, Vue type checking, frontend tests, and the production build before completion.
- Native CI proves compilation. Physical hardware tests prove microphone and system-audio behavior.

## Release Decisions

- Supported release targets are macOS 14+ arm64/x64 and Windows 10 22H2/11 x64. Linux x64/PipeWire is beta.
- Keep CI preview packages unsigned and short-lived. Public macOS packages use
  the protected release environment's Developer ID Application identity and
  notarization credentials; local signed builds receive the identity through
  `APPLE_SIGNING_IDENTITY` instead of committing machine-specific configuration.
  Tauri updater artifacts are always signed.
- Public releases are dual licensed under MIT OR Apache-2.0.
- Do not publish updater metadata until every intended artifact and updater signature is available.

## Guidance Checklist

- Confirm storage contracts still match the readable-on-disk rule.
- Confirm recording safety remains independent of transcription.
- Confirm secrets and user content stay out of logs and diagnostics.
- Confirm UI changes reuse shared tokens and components.
- Report verification and reconsider this guidance before finishing work.
