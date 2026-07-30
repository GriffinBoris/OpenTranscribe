# OpenTranscribe Implementation Review

Date: 2026-07-30

## Outcome

The reliable recorder MVP described in `docs/product-blueprint.md` is implemented
at the code, automated-test, and macOS packaging levels. This review fixed all
verifiable repository findings it uncovered.

Release readiness is still conditional on work that cannot be completed through
repository changes alone:

- validate physical Windows and Linux capture
- configure one stable macOS development or distribution signing identity, then
  revalidate Screen & System Audio Recording for that identity
- run interruption, low-disk, sleep/wake, and long-duration hardware tests
- provide protected platform signing, notarization, and updater credentials

Realtime transcription remains deliberately deferred. Summaries, decisions, and
action-item generation remain outside the accepted product scope until a new
product decision is made.

## Review scope

Reviewed:

- the accepted product blueprint and phase boundaries
- Vue routes, shell state, route stores, app-owned PrimeVue wrappers, semantic
  styling, accessibility hooks, global-shortcut orchestration, and browser
  smoke tests
- Tauri command boundaries, capabilities, credentials, capture ownership,
  storage, recovery, imports, exports, jobs, transcription providers, and the
  local-model sidecar protocol
- GitHub Actions, task-runner commands, package configuration, and the current
  macOS application bundle
- dependency and secret-handling surfaces

Lightly checked or externally blocked:

- Windows WASAPI behavior on physical output devices
- Linux PipeWire behavior across distributions and desktop portals
- macOS ScreenCaptureKit after approval for one stable signed identity
- real OpenAI billing/account behavior and a live provider upload
- full local Whisper inference with every catalog model
- signed installers, notarization, updater signatures, and update installation
- long-running, device-interruption, sleep/wake, and low-disk behavior

## Guidance and examples reviewed

- repository `AGENTS.md`
- `agents/guidance/guidance.md`
- `agents/guidance/frameworks/vue/guidance.md`
- `agents/guidance/project/guidance.md`
- `agents/reference/review/architecture-rubric.md`
- Vue feature-store, app-wrapper, route-folder, app-layout, and workspace-shell
  examples
- the context-gatherer, architecture-audit, frontend-homogeneity-audit, and
  full-review skills

Official technical references checked:

- [OpenAI audio transcription API](https://developers.openai.com/api/reference/resources/audio/subresources/transcriptions/methods/create)
- [OpenAI migration compatibility notes](https://developers.openai.com/cookbook/examples/migrating_from_whisper_to_gpt_transcribe#8-check-compatibility-before-switching)
- [Apple code requirements](https://developer.apple.com/documentation/security/applying-code-requirements)
- [Apple ScreenCaptureKit permission identity diagnosis](https://developer.apple.com/forums/thread/819406)
- [ScreenCaptureKit Rust requirements and permissions](https://github.com/doom-fish/screencapturekit-rs#requirements--permissions)
- [Official whisper.cpp model repository](https://huggingface.co/ggerganov/whisper.cpp)
- [Tauri GitHub Action](https://github.com/tauri-apps/tauri-action)
- [Tauri global-shortcut JavaScript API](https://v2.tauri.app/reference/javascript/global-shortcut/)
- [Tauri global-shortcut Rust plugin](https://docs.rs/tauri-plugin-global-shortcut/latest/tauri_plugin_global_shortcut/)
- [GitHub artifact retention](https://docs.github.com/en/actions/tutorials/store-and-share-data)

### Guidance and example review map

| Source                                                                     | Verdict             | Applicability                                                                                                                                                                     |
| -------------------------------------------------------------------------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Repository `AGENTS.md`                                                     | `matched`           | Required simplicity, verification, dependency, security, and living-guidance rules were applied.                                                                                  |
| `agents/guidance/guidance.md`                                              | `matched`           | Repository structure, verification, dead-code, and boundary rules match the reviewed implementation.                                                                              |
| `agents/guidance/frameworks/vue/guidance.md`                               | `matched`           | Route ownership, app-owned PrimeVue wrappers, semantic tokens, stores, accessibility, and error states match.                                                                     |
| `agents/guidance/project/guidance.md`                                      | `matched`           | Local-first storage, native ownership, capture safety, utility UI, and release boundaries match.                                                                                  |
| `agents/reference/review/architecture-rubric.md`                           | `matched`           | The repository was checked across all rubric categories; externally blocked validation is listed separately.                                                                      |
| `vue-app-layout.md`                                                        | `matched`           | Routes, shell state, shared controls, types, styles, and the native bridge each have one clear home.                                                                              |
| `vue-app-owned-wrapper-component.md`                                       | `matched`           | Interactive controls stay behind app-owned PrimeVue wrappers; the dialog pass-through defect was fixed there.                                                                     |
| `vue-composable-reactivity.md`                                             | `matched`           | First-run state and derived readiness remain in one focused composable with reactive inputs.                                                                                      |
| `vue-dialog-form.md`                                                       | `matched`           | Dialogs use shared controls, explicit loading/error state, focus trapping, Escape close, and focus restoration.                                                                   |
| `vue-feature-store-route.md`                                               | `matched`           | Session and trash workflows use focused route-local stores instead of expanding the shell store.                                                                                  |
| `vue-form-validation.md`                                                   | `matched`           | Forms use explicit required state and owned workflow errors without a parallel validation abstraction.                                                                            |
| `vue-loading-error-states.md`                                              | `matched`           | Shell and route workflows distinguish loading, empty, mutation, and error state without duplicated placeholders.                                                                  |
| `vue-route-folder.md`                                                      | `matched`           | Route views, local components, composables, and stores are colocated under `src/views/`.                                                                                          |
| `vue-type-interface-pattern.md`                                            | `matched`           | Desktop DTOs are typed explicitly at the canonical native bridge boundary.                                                                                                        |
| `vue-view-pattern.md`                                                      | `matched`           | Route views compose focused local components and keep shell structure out of route content.                                                                                       |
| `vue-workspace-shell-page.md`                                              | `matched`           | One compact shell owns the sidebar, title bar, main pane, recording dock, and full-width route outlet.                                                                            |
| `vue-api-client.md`, `vue-standardized-error-helpers.md`                   | `not_applicable`    | This desktop app uses one typed Tauri bridge rather than HTTP, Axios, DRF casing, or DRF error payloads.                                                                          |
| `vue-auth-shell.md`, `vue-route-auth-guard.md`, `vue-session-sso-login.md` | `not_applicable`    | OpenTranscribe is a local desktop utility without application accounts or authenticated routes.                                                                                   |
| `vue-clipboard.md`                                                         | `not_applicable`    | No reviewed workflow writes to the clipboard.                                                                                                                                     |
| `vue-multi-step-form.md`                                                   | `not_applicable`    | First run is one compact setup surface rather than a routed or stepper form.                                                                                                      |
| `vue-notification-system.md`                                               | `partially_matched` | One shell-level alert surface owns native attention events; this desktop utility intentionally keeps recoverable failures visible instead of timing them out as transient toasts. |
| `vue-polling.md`, `vue-task-polling.md`                                    | `not_applicable`    | Recording and job updates use one ordered native event channel rather than browser polling.                                                                                       |
| `vue-route-query-state.md`                                                 | `not_applicable`    | Search and dialog filters are intentionally transient rather than shareable routed state.                                                                                         |
| `vue-table-wrapper.md`                                                     | `not_applicable`    | Current bounded desktop lists do not share pagination or a repeated table workflow that justifies a wrapper.                                                                      |

## Review map

| Area                        | Verdict             | Evidence                                                                                                                                                                                          |
| --------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Product scope               | `matched`           | Recorder-first MVP is implemented; realtime and new AI-derived content remain explicitly deferred.                                                                                                |
| Frontend ownership          | `matched`           | Route folders own local views/stores; shell workflows are split into application, recording, global-shortcut, credential, and model stores. Post-stop provider work stays native-owned.           |
| Component consistency       | `matched`           | Interactive controls use app-owned PrimeVue wrappers; raw native form primitives are linted out.                                                                                                  |
| Layout and visual hierarchy | `matched`           | The sidebar and inset utility pane use semantic tokens, full-width route pages, adaptive gutters, and restrained copy.                                                                            |
| Native boundary             | `matched`           | Vue calls a typed bridge; Rust owns capture, files, credentials, providers, process supervision, and the narrowly permitted global-shortcut plugin.                                               |
| Security and privacy        | `matched`           | The API key stays in the OS credential vault, CSP/capabilities are narrow, and playback grants asset access only to validated session files.                                                      |
| Storage and recovery        | `matched`           | Readable files are authoritative, writes are atomic, editable documents use revision checks, session moves rebase artifact paths, and recovery chunks are validated before removal.               |
| Transcription               | `matched`           | OpenAI diarized timestamps/speakers and local timed segments feed one canonical transcript; durable jobs, progress, retry/cancel, and immutable run artifacts are contract-tested.                |
| Capture adapters            | `partially_matched` | macOS, Windows, and Linux adapters compile; microphone recording/playback passed on macOS, while packaged system audio awaits a stable signed identity and non-macOS hardware validation remains. |
| Release automation          | `partially_matched` | Pull requests compile, trusted branch/nightly runs retain unsigned installers for seven days, and tag builds create drafts; signing/updater credentials are not configured.                       |
| Automated verification      | `matched`           | Formatting, linting, type checking, unit, schema, protocol, browser-error, light/dark accessibility, dependency, workflow, and macOS package checks pass.                                         |
| Realtime transcription      | `not_applicable`    | Explicitly deferred until the recorder MVP is release-hardened.                                                                                                                                   |
| Summaries/actions           | `not_applicable`    | Explicitly excluded without a new product decision.                                                                                                                                               |

## Findings fixed

### Session organization and utility UI polish

Completed meetings can now move directly between Inbox and any project from a
compact PrimeVue project selector in the session header. The typed bridge calls
a thin Tauri command, while a focused repository module moves the complete
session directory, updates its project assignment and revision, rebases every
library-relative artifact path, and refreshes the search index. Recording and
active-processing guards prevent relocation while another native workflow owns
the session. A failed manifest rewrite moves the directory back instead of
leaving its readable metadata inconsistent with its location. Native and
browser regression tests cover artifact preservation, search scope, project
removal, Inbox restoration, and the visible control.

The same UI pass replaced the notes editor's accent focus border with a quiet
token-based background change and aligned the Shortcuts status and description
inside the standard section-heading structure. Session operation failures now
use a compact inline notice instead of expanding a large blocking error state.
Shared progress tracks are now more substantial, preserve fractional updates,
and animate their fill with a short compositor-driven transform instead of a
lagging layout-driven width transition.

The follow-up visual consistency pass now routes Project, Processing, and Trash
through one compact empty-state component with the same Lucide treatment,
spacing, title, and actionable supporting copy. Processing no longer renders
table columns when there are no jobs. Settings section headers and permission
actions share one alignment model, and an unassigned session's project selector
now visibly reads Inbox instead of appearing blank.

### Unified application icon system

The sidebar previously used a generic Lucide microphone while the system tray
reused the full-color application tile as a macOS template image. Because
template images use transparency rather than their original colors, that
opaque tile collapsed toward a solid rounded square in the menu bar. The
sidebar, packaged application, installers, Dock or taskbar, and mobile-size
assets now derive from one canonical full-color OpenTranscribe mark. The tray
uses the same three-part silhouette as a transparent monochrome template.
`npm run icons` regenerates all 53 native assets deterministically, while
functional interface icons remain in the single Lucide family.

### Unified post-recording actions

Stopping from the recording dock or global shortcut previously finalized the
audio in Rust and then asked the Vue shell to enqueue the selected
transcription. The tray called the native stop command directly, so the same
recording could silently skip its local or OpenAI transcription when stopped
from the tray. Recording intent is now passed through the typed bridge and held
with the active native session. Every stop origin runs one native finalization
path, and Rust enqueues the selected post-recording action only after the audio
is durably ready. A missing model or credential emits an attention event while
leaving the recording available for manual retry. Focused model-selection tests
and a browser workflow cover the automatic local path.

The same stop review found that a finalizer error after capture shutdown left
the tray enabled and the session looking actively recording. That failure now
resets native recording indicators, marks the session as needing attention,
keeps its recovery chunks available, refreshes the library, and reports the
failure without attempting transcription.

### Reopenable tray lifecycle

The tray advertised an Open action, but closing the main window could destroy
the only webview, leaving nothing for that action to show. The native lifecycle
now hides the main window on close and keeps it reopenable from the tray.
User-initiated application and tray quit actions are refused while capture is
active, bring the recording surface back into view, and explain that recording
must stop first. Programmatic exit remains available after the tray has applied
the same guard.

### Local model preset alignment

The first catalog labeled Whisper Base as “Fast” and Whisper Small as
“Balanced,” while the accepted model plan specifies Small, Medium, and Large v3
Turbo. The catalog and preview adapter now expose those three exact quality
tiers using pinned official whisper.cpp artifacts with explicit byte counts and
SHA-256 verification. Existing selection still prefers an installed Balanced
model and falls back deterministically to the first installed tier.

### Current-only native event contract

The shared event enum still advertised device-change, capture-gap, realtime
transcript-delta, committed-segment, and updater variants that no runtime path
could emit. Those speculative variants made the typed boundary broader than the
implemented product and implied deferred realtime behavior. The contract now
contains only recording state and levels, jobs, library/import requests, and
attention events that have real producers and consumers.

### Opt-in global recording shortcut

The accepted native-behavior plan called for configurable global shortcuts that
remain disabled until the user accepts them, while the application only exposed
window-local keyboard actions. Settings now persist an opt-in record/stop
shortcut selected from three validated presets. One focused shell store owns
registration state and visible conflicts, the typed native bridge registers
only press events through Tauri's global-shortcut plugin, and the shell
serializes start and stop actions so one key press cannot overlap another.
Capabilities grant only register and unregister operations. Legacy settings
default safely to disabled, and browser coverage exercises enable, start, stop,
runtime-error detection, and the enabled accessibility state.

### Session-specific transcription progress

The session view previously selected the first global transcription job. A job
for another session could disable actions or show progress in the wrong
workspace. Active jobs are now selected by session, transcription kind, and
active state, with focused unit coverage.

### External notes conflict safety

A library refresh adopted an externally edited file's new hash without loading
that content into the open editor. The next autosave could therefore overwrite
the external edit with stale text. Workspace refresh now updates transcripts
without replacing the editor's expected notes hash. A stale save reaches the
native revision boundary, reports a conflict, and leaves the external content
untouched. A native regression test covers the data-preservation contract.

### Trusted installer artifacts

The reusable native workflow previously performed compile-only checks for every
trigger. Pull requests remain compile-only, while trusted `main` and nightly
runs now build unsigned installers on macOS, Windows, and Linux and retain them
for seven days.

### Linux CI prerequisites

Quality, test, native-package, and tag-release jobs now install the same complete
Tauri, PipeWire, tray, image, CMake, and packaging prerequisites. Release builds
also use locked dependencies and explicitly skip signing until protected
credentials are configured.

### System-audio permission truthfulness

Adapter availability was previously presented as a successful system-audio
permission state, and macOS permission copy could appear on Windows or Linux.
The native boundary now returns ScreenCapture preflight status separately from
adapter support. The UI shows ready, permission-required, supported, and
unavailable states accurately and offers a focused macOS action that opens
Screen & System Audio Recording settings without exposing a broad shell
capability.

### Native system-audio error preservation

The macOS adapter previously discarded every ScreenCaptureKit content-access and
stream-start error and replaced it with the same permission message. The startup
boundary now preserves the native failure context, limits permission recovery
copy to content access, and does not call unrelated stream-start failures
permission problems. Focused native tests cover both messages. The live retry
then exposed the real cause: macOS declined TCC access for the rebuilt unsigned
binary identity.

### Stable macOS permission identity

The visible Screen & System Audio Recording switch and ScreenCaptureKit's
`user declined TCCs` result were both real but referred to different code
identities. The packaged development app was ad-hoc signed with a designated
requirement based on its changing code hash, and the machine has no stable code
signing identities installed. Apple documents that privacy grants follow the
designated requirement and confirms that ad-hoc rebuilds are treated as new
apps. The app now exposes the native preflight result, while the remaining
durable fix is intentionally held at the signing boundary because creating a
keychain identity requires explicit user approval.

### Timestamped OpenAI transcripts

The OpenAI path previously requested a text-only GPT response and assigned one
synthetic segment to each upload chunk. That made transcript seeking and
SRT/VTT timing appear more precise than the provider response. The completed
cloud action now requests `gpt-4o-transcribe-diarize` with `diarized_json` and
automatic chunking, preserves every provider timestamp and speaker label, and
offsets later upload chunks onto the session timeline. Text-only models remain
accepted as an explicit compatibility path with coarse chunk timing. Multipart
request, response parsing, chunk offset, fallback, and canonical speaker
mapping tests cover the contract, and Test connection now checks access to the
same diarization model used by the primary action.

### Concurrent quality-gate reliability

The accepted command surface named a dedicated `task test:ui`, but the Taskfile
did not expose it and `task test` skipped browser flows. Both commands now run
the workspace UI suite. Running the full Task dependency graph then exposed an
ESLint/Playwright race: ESLint could traverse `test-results` while Playwright
recreated it. Generated coverage, report, and test-result directories are now
ignored by ESLint, and the concurrent `task check` completes reliably.

### Credential documentation accuracy

The privacy documentation previously said an API key never entered frontend
JavaScript or the webview. A user necessarily types it into the Vue Settings
form before it is passed to Rust. The docs now state the precise boundary: the
key is transient in the form, passed directly to the native credential command,
stored in the operating-system credential vault, and never persisted by the
frontend or meeting library.

The same pass removed UI concepts the implementation did not support: the
single credential is now labeled as one stored API key rather than an OpenAI
"profile," the duplicate empty-state pitch beneath the key form is gone, and
the completed-session action says what it does: Transcribe with OpenAI.

### Legacy task-runner surface

Unused Django, Docker, deployment, and database task files inherited from the
repository template were removed. The root Taskfile now exposes only current
OpenTranscribe and authored-guidance commands.

### Accessibility and native dialog stability

The browser suite exercised keyboard activation but did not run an accessibility
engine, so repeated light-theme contrast failures in the accent, secondary-text,
and semantic-status tokens were invisible to automation. The primary routes now
run axe-core WCAG A/AA scans in both light and dark themes, and focused browser
flows verify dialog focus containment and restoration plus keyboard recording,
timestamp, and stop actions. The semantic token layer was corrected once instead
of adding route-specific color overrides.

The same gate now fails on uncaught page and console errors. That exposed a
PrimeVue dialog pass-through bug where a nested icon class string was spread as
numeric DOM attributes during portal updates. The shared `AppDialog` wrapper now
passes attribute objects to the nested close-button sections, preventing the
native webview from going blank even though the partially rendered dialog had
previously allowed browser assertions to pass.

The permission-status pass also removed the macOS recovery action after native
preflight reports success. Open System Settings remains available only when
permission is actually required.

## Verification

Passed:

- `npm run format:check`
- `npm run lint`
- `npm run typecheck`
- `npm run test` — 3 tests
- `npm run test:ui` — 25 browser flows, including seven primary routes in light
  and dark themes with WCAG A/AA scans
- `npm run build`
- `npm audit` — 0 vulnerabilities
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` — 100 unit, schema, protocol, and smoke
  tests
- `task check` — the full concurrent local quality graph
- `task ai:check` — all authored guidance targets build successfully
- deterministic `npm run icons` regeneration across all 53 native assets
- workflow YAML parsing
- Taskfile discovery
- generated-guidance stability across consecutive generations
- `git diff --check`
- `npm run tauri -- build --ci --no-sign -- --locked`
- `npm run tauri build` — fresh unsigned macOS app and DMG
- `hdiutil verify` on the generated DMG
- arm64 architecture and dynamic-library inspection for both packaged binaries
- a real ten-second macOS microphone recording and native playback

Generated package:

- `target/release/bundle/macos/OpenTranscribe.app`
- `target/release/bundle/dmg/OpenTranscribe_0.1.0_aarch64.dmg`
- DMG SHA-256:
  `09a25229d49e968d828490b7907a61aaba041531d4ce4dca1051dca725e8e126`

The development package is intentionally unsigned. Its DMG checksum and
contents verify, while app-bundle code-signature verification remains
inapplicable until a stable signing identity is configured.

The macOS system-output attempt reached ScreenCaptureKit. After native error
preservation was added, the rebuilt app reported ScreenCaptureKit's exact
failure: TCC access was declined for its ad-hoc binary identity. The application
preserved the session boundary, and a microphone-only retry recorded, finalized,
exposed, and played ten seconds successfully. The hot-reload executable now
reports the process preflight value in the UI, but a final packaged
system-output success check requires stable signing, a grant for that identity,
and relaunch.

## Guidance decision

The durable full-width route-page rule discovered during visual QA is recorded
in project guidance. System-audio guidance now also covers native error
preservation and the macOS code-identity/TCC boundary. Transcription guidance
requires real model timestamps for seekable segments and subtitle export.
Accessibility guidance now requires light/dark primary-route scans, semantic
token fixes for repeated contrast issues, accurate permission recovery actions,
and browser-error detection for portal failures. Global-shortcut guidance now
requires opt-in validated presets, narrow capabilities, press-only serialized
handling, and visible registration conflicts. Recording guidance now requires
all stop origins to share native finalization and post-processing while keeping
transcription failure independent from a saved recording. Session-move guidance
now requires the complete directory, project assignment, revision, and every
artifact path to move together, with rollback if the manifest rewrite fails.
Generated guidance targets are current and stable across consecutive
generations.
