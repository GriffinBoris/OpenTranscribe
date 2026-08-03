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

- `apps/desktop/` is the desktop product entrypoint and owns its npm, Vite,
  Playwright, and Tauri configuration.
- `apps/desktop/src/` contains the Vue application. Route views live in
  `apps/desktop/src/views/`; shared controls live in
  `apps/desktop/src/components/`.
- `apps/desktop/src-tauri/` contains the desktop process and thin Tauri command
  adapters. Keep native services and orchestration beside, not inside, the
  `commands/` adapter folder.
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
- Linux release bundles and native compile checks build on Ubuntu 22.04, our
  glibc 2.35 compatibility baseline. Keep Linux PipeWire bindings compatible
  with the runner's PipeWire 0.3 development headers; do not move those jobs
  to a newer base image without intentionally raising the published glibc
  requirement.
- Keep the macOS deployment target at 14.0 in both Cargo and Tauri bundle
  configuration. The ScreenCaptureKit Rust adapter includes a Swift bridge, so
  `build.rs` derives the active toolchain's Swift runtime library path through
  `xcrun` instead of hardcoding an Xcode installation path.
- When a dependency update raises the Rust compiler requirement, update the
  workspace `rust-version`, `rust-toolchain.toml`, and every GitHub Actions
  toolchain pin together so local development, CI, and release builds stay on
  one supported compiler.
- Local inference runs in the supervised sidecar. A sidecar failure must not stop or corrupt recording.
- Build the macOS sidecar with Whisper Metal support and explicitly request its GPU path. Keep Windows and Linux acceleration as opt-in target variants rather than universally enabling CUDA or Vulkan, because those backends impose hardware- and SDK-specific build requirements that would make ordinary cross-platform installs unreliable.
- Keep recording completion native-owned. Dock, tray, and global-shortcut stop
  actions must converge on the same finalization path, and any selected
  post-recording transcription starts only after durable audio finalization.
  Provider, credential, or model failures must leave the saved recording ready
  for manual retry and surface an attention event instead of turning recording
  completion into a failure.
- Stop capture synchronously, then finalize tracks, mixes, and waveform data in
  a durable `FinalizeRecording` job. Never run that disk-heavy work on the
  stop command path; a finalization job is recoverable and intentionally cannot
  be canceled.
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
- Cache RMS waveform energy levels as a small JSON artifact after audio
  finalization. This avoids saturating long recordings where a per-bucket peak
  would make every bar look full-height. A missing waveform must not make
  recorded audio or playback unavailable.
- When moving a session between Inbox and a project, move the complete session
  directory, update its project assignment and revision, rebase every
  library-relative artifact path, and refresh the rebuildable index. A failed
  manifest rewrite must move the directory back so readable files do not
  disagree with their location.
- Trash is app-managed, recoverable, and never automatically purged. A manual
  purge permanently removes every trashed session only after explicit user
  confirmation.
- Reset settings restores recording, shortcut, and appearance preferences while
  preserving the library selection, OpenAI credential, downloaded models, and
  configured model location. It returns to recording setup and must be
  unavailable while recording, processing, or downloading a model.
- Delete-all-data clears app-owned configuration, remembered library selection,
  credentials, downloaded models, caches, and the selected library's
  OpenTranscribe-owned `Inbox`, `Projects`, `Trash`, and `.opentranscribe`
  folders. It must never remove unrelated user files beside a custom library,
  requires explicit typed confirmation, leaves operating-system permission
  grants under OS control, and is unavailable while recording, processing, or
  downloading a model.
- Keep downloaded local models in a configurable global folder, defaulting to
  `Documents/OpenTranscribe/models`. Show the resolved path in settings and
  move existing model files only after explicit confirmation; do not allow a
  move while recording, processing, or downloading. Local model artifacts come
  from the pinned upstream `ggerganov/whisper.cpp` Hugging Face revisions and
  must pass their catalog SHA-256 integrity check before use.
- Schema changes require sequential migrations and fixtures covering the previous schema.

## Frontend Conventions

- The application shell owns the sidebar, recording status, theme, bootstrap
  state, and router outlet. Keep cross-route recording and credential workflows
  in focused shell-level stores beside the application store instead of growing
  one catch-all store.
- Route folders own their views, local components, and local stores. Avoid a global catch-all store.
- Wrap PrimeVue primitives in app-owned components under
  `apps/desktop/src/components/ui/`.
- Keep the OpenTranscribe application, installer, Dock/taskbar, sidebar, and
  tray icons derived from the canonical artwork in `apps/desktop/src/assets/`. Use the
  full-color application tile for branded surfaces and the matching transparent
  monochrome mark for system tray or menu-bar template surfaces. Functional UI
  icons use Lucide so stroke weight and optical sizing remain consistent.
- Render interactive form controls through those app-owned PrimeVue wrappers.
  Raw `button`, `dialog`, `input`, `select`, and `textarea` elements are linted
  as errors so focus behavior, overlays, disabled states, and visual treatment
  stay consistent across the desktop app.
- Use semantic tokens from `apps/desktop/src/styles/tokens.css`; do not hardcode theme colors
  in components. Reuse the shared typography, spacing, control-size, radius,
  shadow, motion, opacity, layering, and shell-layout scales whenever a value
  participates in app-wide visual consistency. Keep intrinsic media dimensions,
  responsive breakpoints, and genuinely component-specific geometry local
  instead of creating a token for every pixel value.
- Use Tailwind CSS v4 through the Vite plugin for layout, spacing, typography,
  color, borders, and responsive states. Keep OpenTranscribe's semantic CSS
  tokens as Tailwind theme values so utilities continue to adapt to light and
  dark themes. Limit authored CSS to token definitions, base rules,
  PrimeVue pass-through states, animations, and browser-specific behavior;
  do not add new layout or control selectors when a utility class expresses
  the intent clearly.
- Keep the live transcript read-only while recording. Notes and markers remain editable.
- Stream OpenAI live captions from the native backend so credentials never enter
  the webview. Fan microphone and system packets into bounded, non-blocking
  queues so network latency cannot interrupt the recording writer. Treat live
  captions as ephemeral feedback; after stop, create the durable transcript
  from the finalized recording with the user's selected file model.
- Treat streamed file-transcription output as ephemeral feedback too. Persist
  only the completed provider response as the durable transcription run.
- Every provider action displays whether audio stays local or is sent to OpenAI.
- Preserve provider-reported transcription usage with each immutable run. Derive
  cloud cost estimates from one centralized, dated pricing table, include both
  the live and saved-transcript passes when both are requested, count concurrent
  source streams separately, and label the result as an estimate because
  provider billing and prices can change. Never invent a price for a model whose
  official rate is not published.
- All strings go through Vue I18n even though the first release ships only English.
- Maintain keyboard navigation, visible focus, reduced motion, and WCAG 2.2 AA contrast.
- Keep the desktop shell compact and product-first: one quiet sidebar, one inset
  content pane, and a macOS overlay title bar that preserves native traffic
  lights. Use the dedicated non-interactive top strip for window dragging and
  grant only `core:window:allow-start-dragging` for that behavior.
- Keep global recording shortcuts opt-in and bind them through a focused capture
  flow rather than a free-text accelerator field. Persist portable
  `CommandOrControl` accelerator strings, migrate legacy preset values when
  settings load, require a Command/Control modifier plus a supported key, and
  render the platform-specific label in the UI. Register them through the
  typed native bridge with narrow plugin capabilities, key registrations by
  action so independent features do not replace each other's handler, act only
  on press events, serialize record/stop handling, and surface registration
  conflicts in Settings.
- Keep global Dictation shortcuts opt-in and configurable through the same
  focused capture flow. Dictation may use an Option/Alt modifier without
  Command/Control for a quick toggle, but it must still require a modifier and
  a supported non-text key. Prevent a new Dictation run while the previous one
  is transcribing, and snapshot its provider, model, and delivery preferences
  before capture so a later settings change cannot alter the active run.
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
- Keep Inbox and project library pages on the shared checkbox selection and
  bulk-move-dialog workflow rather than adding a project selector to every
  row. Session workspaces use the compact move utility in their header; the
  Home recents list may retain its compact single-row control.
- First-run setup confirms the default Documents library, microphone and
  system-audio choices, and the default transcription path. Keep it compact
  and skippable: do not require a provider account, model download, permission
  grant, or audio test to continue. The optional audio test uses the normal
  recorder, finalizer, and playback path, requests only the permissions needed
  by selected sources, and shows recording-consent guidance before the test.
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
  Tauri updater artifacts are created only when the protected release
  environment provides both the immutable updater private key and its public
  counterpart. The application checks the static GitHub Release manifest on
  startup, but it must ask before downloading or restarting and must defer the
  install while recording, processing, or downloading a model.
- Release builds upload their platform installers as workflow artifacts. One
  publish job downloads the complete set and creates the draft GitHub Release;
  do not let matrix jobs race to create or mutate the same release.
- Pull requests and the main pipeline own quality and test coverage. Tagged
  releases run only version validation and installer packaging so a release
  does not duplicate those expensive checks before building its artifacts.
- Pull requests run the focused Ubuntu 22.04 native contract job alongside
  frontend checks. Use the manual pipeline for macOS or Windows native checks
  when a change needs platform-specific validation.
- Public releases are dual licensed under MIT OR Apache-2.0.
- Do not publish updater metadata until every intended artifact and updater signature is available.
- Preserve the updater signing key for the lifetime of every updater-enabled
  release line. A replacement key cannot update installations that trust the
  original public key.
- The static GitHub Release update feed carries one Linux payload, so enable
  in-app updating only from the AppImage. Leave `.deb` installs to the system
  package manager rather than serving them the wrong updater artifact.

## Guidance Checklist

- Confirm storage contracts still match the readable-on-disk rule.
- Confirm recording safety remains independent of transcription.
- Confirm secrets and user content stay out of logs and diagnostics.
- Confirm UI changes reuse shared tokens and components.
- Report verification and reconsider this guidance before finishing work.
