# OpenTranscribe

OpenTranscribe is a local-first desktop meeting workspace. It records audio to
readable files you control, keeps notes beside each session, and can transcribe
completed recordings locally or with an OpenAI API key.

## What it does

| Area              | Features                                                                                                                                                                                                                                                                                                              |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Record            | Capture a microphone and system output as separate tracks, optionally reduce speaker bleed locally with echo cancellation, pause and resume, monitor live levels, control recording from the Dock, tray, menu, or an optional global shortcut, and recover sealed crash-recovery chunks after an interrupted session. |
| Organize          | Keep sessions in an Inbox or named projects, search titles, notes, and transcripts, rename and move sessions, import existing audio or video, and use recoverable Trash rather than deleting work immediately.                                                                                                        |
| Work in a session | Play the mixed or source tracks with a cached waveform, write timestamped Markdown notes, review live captions, edit transcript segments, rename or merge speakers, and export a completed transcript as Markdown, text, JSON, SRT, or VTT.                                                                           |
| Transcribe        | Choose local whisper.cpp transcription after recording, OpenAI file transcription, or OpenAI live captions while recording. Jobs are durable, show progress and estimates where applicable, and can be retried without risking the recording.                                                                         |
| Dictate           | Use a compact dictation panel with its own optional global shortcut. A run snapshots its provider and delivery choices, transcribes after capture, and preserves a history for review.                                                                                                                                |
| Configure         | Guide first-time setup through library selection, audio sources, a transcription preference, and an optional ten-second audio test. Settings cover appearance, capture, keyboard shortcuts, storage, local-model downloads, OpenAI credentials, updates, and application reset controls.                              |

### Dictation and optional S1-mini cleanup

Choose a speech model in **Settings → Dictation**, then start from the Dictation
page or enable its global shortcut. Press the shortcut once to start and again
to stop and transcribe. Each new dictation places the compact panel on the
display containing your pointer. On macOS, it appears without taking focus
from the app you are typing in. The panel shows recording, transcription, and
delivery status. Cancel stops pending delivery; a failed run
offers Retry and keeps its audio until discarded, replaced, or the app restarts.

For English dictation, download **S1-mini by Superwhisper** (484 MB) under
**Clean up dictation**, then enable the toggle. S1-mini cleans the text produced
by your selected Whisper or OpenAI speech model, removing fillers and applying
spoken corrections. Cleanup runs locally and preserves the original transcript
in history. It is optional and supports up to 1,000 input tokens per dictation.
Using OpenAI for speech recognition still sends audio to OpenAI. S1-mini is a
text cleanup model, so it appears under **Clean up dictation**, not in the
**Speech recognition model** selector. For fully local dictation, install and
select a Whisper model, then optionally enable S1-mini.

Enable **Paste automatically** to paste into the original app and dismiss the
panel after transcription and optional cleanup finish. Keep that app active
until completion. On macOS, OpenTranscribe also needs **Accessibility** access.
If automatic paste is disabled, access is denied, or you switch apps, the text
is copied and the panel stays open so you can paste manually. Linux currently
uses copy only. If paste stops working after replacing an unsigned build, follow
the [permission reset steps](#macos-reset-permissions-after-reinstalling-an-unsigned-build).

### Screenshots

<p align="center">
  <img src="docs/images/home-workspace.png" alt="OpenTranscribe home workspace with recording controls and recent sessions" width="900" />
</p>

<p align="center">
  <img src="docs/images/session-workspace.png" alt="OpenTranscribe session workspace with transcript and notes" width="900" />
</p>

<p align="center">
  <img src="docs/images/settings-workspace.png" alt="OpenTranscribe settings workspace" width="900" />
</p>

## How your data stays yours

- Your selected library is the source of truth: sessions stay as readable JSON
  manifests, Markdown notes, PCM WAV tracks, and JSON transcripts. SQLite only
  accelerates search and can be rebuilt from those files.
- Audio never crosses the Tauri IPC boundary, and recording is finalized before
  local inference, uploads, waveform generation, indexing, or export begin.
- Local transcription runs in a supervised sidecar. A sidecar or provider
  failure leaves the completed recording intact for a manual retry.
- OpenAI is optional. Its API key is stored only in the native operating-system
  credential vault, and the interface states when audio will be sent to OpenAI.
- OpenTranscribe has no account requirement, telemetry, hosted sync, or remote
  crash reporting.

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

Microphone echo cancellation is an opt-in local path shared by the macOS,
Windows, and Linux capture adapters. It uses the captured system-output track
as a reference, so it does not rely on a platform-specific Voice Isolation
feature and never sends audio to a service. Physical validation remains
required on every supported operating system because acoustic suppression also
depends on the room, microphone, speaker, and Bluetooth latency. The full
release matrix is in [`docs/platform-validation.md`](docs/platform-validation.md).
OpenAI live transcription is implemented; local transcription runs after the
recording is durably finalized so model latency cannot interrupt capture.

Pull requests run the shared native tests on macOS, Windows, and Linux plus
compile checks for every supported release triple. Trusted `main` and nightly
runs also retain unsigned installers for seven days so current packages can be
smoke-tested before signing is configured.

## Installing a release

Download the installer for your platform from the releases page.

**macOS** ships a separate DMG per architecture: `aarch64` for Apple Silicon
and `x86_64` for Intel. Drag OpenTranscribe to Applications and open it from
there, not from the mounted disk image, so macOS does not run it from a
randomized read-only location that discards capture permissions. Grant
microphone access at the first prompt, then grant Screen & System Audio
Recording in System Settings to capture meeting audio.

Signed releases open with the ordinary confirmation for a downloaded
application. Unsigned preview builds are blocked instead: open System Settings,
go to Privacy & Security, and choose Open Anyway for OpenTranscribe, or clear
the quarantine attribute after moving the app into place.

```bash
/usr/bin/xattr -d -r com.apple.quarantine "/Applications/OpenTranscribe.app"
```

Use lowercase `-r` (recursive), not uppercase `-R`. The full `/usr/bin/xattr`
path selects Apple’s utility even if Python or another tool installed a different
`xattr` on your PATH. This removes the launch quarantine only; for microphone,
screen recording, or automatic-paste permissions, use the reset steps below.

**Windows** installers are unsigned, so SmartScreen shows a warning on first
run. Choose More info, then Run anyway. Nothing degrades on later updates.

**Linux** provides an AppImage and a `.deb`, built on Ubuntu 22.04 for systems
with glibc 2.35 or newer. Both need PipeWire 1.0 or newer running for
system-output capture. Mark the AppImage executable before running it.

## Updating a release

OpenTranscribe checks the signed release manifest on startup and only downloads
an update after you select **Update**. It waits until recordings, processing,
and model downloads finish.

- **macOS:** Run the copy in Applications. A mounted disk image is read-only,
  so it cannot replace itself.
- **Windows:** The updater starts the normal installer in passive mode;
  Windows can show its standard elevation prompt when needed.
- **Linux:** In-app updates apply only to the AppImage. Keep it in a writable
  folder. Update `.deb` installations through the system package manager.

### macOS: reset permissions after reinstalling an unsigned build

Replacing an unsigned or ad-hoc signed build can change the code identity macOS
uses for privacy permissions. An old OpenTranscribe entry may still look enabled
in System Settings even though the replacement cannot record audio or paste
dictation. If this happens after a reinstall or update:

1. Quit OpenTranscribe completely with **Quit** from its menu or tray. Closing
   its window only hides it. Stop any development instance too.
2. Install the replacement at `/Applications/OpenTranscribe.app` and eject the
   installer disk image. Keep the application closed for the reset.
3. Open Terminal and run:

   ```bash
   tccutil reset Microphone com.griffinboris.opentranscribe
   tccutil reset ScreenCapture com.griffinboris.opentranscribe
   tccutil reset Accessibility com.griffinboris.opentranscribe
   ```

   These reset only OpenTranscribe's microphone, Screen & System Audio Recording,
   and Accessibility decisions. Keep the bundle ID on each command so other
   applications' permissions are unaffected. No `sudo` is needed.

4. Open `/Applications/OpenTranscribe.app` again and start a recording to request
   capture access. In **System Settings → Privacy & Security**, allow
   **Microphone** and **Screen & System Audio Recording**. Enable
   **Accessibility** if you use dictation's automatic paste. If an old entry
   remains, remove it with the minus button where available and add the current
   application from Applications.
5. Quit and reopen OpenTranscribe after granting access, then test recording
   and dictation again.

The reset removes permission decisions; it does not grant access or delete
recordings, models, settings, or saved API keys. Clearing the quarantine
attribute with `xattr` only addresses launch blocking and does not reset these
permissions. See [Apple's permission reset documentation](https://developer.apple.com/documentation/xcode/resetting-access-to-protected-resources-in-macos)
and [macOS signing](docs/macos-signing.md) for details.

## Stack

- Tauri 2 and Rust for capture, storage, credentials, and providers
- Vue 3, TypeScript, Pinia, Vue Router, and PrimeVue for the interface
- SQLite FTS as a rebuildable index; JSON, Markdown, and audio files remain the
  durable source of truth
- A versioned MessagePack protocol for the local transcription sidecar

## Repository layout

- `apps/desktop/` is the complete desktop product entrypoint: Vue source,
  frontend tests and tooling, plus the Tauri crate under `src-tauri/`.
- `crates/` contains Rust libraries shared across native processes.
- `sidecars/` contains supervised companion executables such as the local
  transcriber.
- `scripts/`, `tasks/`, and `.github/` contain repository-level automation.
- `docs/` contains product, platform, credential, signing, and design-system
  references.

## Development

Requirements:

- Node.js 22 or newer
- Rust 1.92
- CMake 3.20 or newer for the bundled Whisper and S1-mini sidecars
- Tauri's operating-system prerequisites
- Linux builds also need PipeWire 1.0 or newer development headers
- Go Task is optional; every task delegates to npm or Cargo

```bash
task install
task dev
```

Useful checks:

```bash
task check
task build
task build:native
task build:dmg
APPLE_SIGNING_IDENTITY="Developer ID Application: …" task build:signed:macos
npm --prefix apps/desktop run icons
task types:generate
```

`task dev` is the single hot-reload entrypoint. Stop that process
before starting another instance, and quit any installed or release-bundle
copy of OpenTranscribe, so permission prompts, tray state, and the visible
window stay attached to the development process.

`npm --prefix apps/desktop run icons` regenerates every native application icon
and the dedicated monochrome system-tray icon from the canonical artwork in
`apps/desktop/src/assets/`.

`task types:generate` refreshes the frontend domain contracts from the Rust
domain crate. Commit generated contract changes with the Rust model change that
produced them.

### Local Linux CI

Use Docker to run the Linux CI gates locally before dispatching a GitHub
workflow:

```bash
task ci:linux:docker MODE=native-test
task ci:linux:docker MODE=quality
task ci:linux:docker MODE=release
```

The first command creates a cached Ubuntu 22.04 x86_64 image and Docker volumes
for Rust, npm, browser downloads, and build output. It validates the Linux
dependencies and checks without consuming GitHub Actions minutes.
`frontend-test` and `browser-test` are also available modes. `release` builds
the Linux binary and `.deb`; AppImage bundling still needs a native x86_64 Linux
host because `linuxdeploy` does not run reliably through Docker Desktop's x86
emulation on Apple Silicon. Docker cannot validate macOS or Windows-specific
behavior.

The signed macOS task requires an installed Developer ID Application identity,
builds the app and DMG with that identity, rejects an ad-hoc signature, and
rejects a bundle whose capture entitlements did not survive signing. macOS is
the only platform where signing changes runtime behavior: it binds capture
permissions and Keychain access to the signing identity, so unsigned updates
silently lose both. See [docs/macos-signing.md](docs/macos-signing.md) for
local and CI setup.

Without Task:

```bash
npm --prefix apps/desktop run format:check
npm --prefix apps/desktop run lint
npm --prefix apps/desktop run typecheck
npm --prefix apps/desktop run test
npm --prefix apps/desktop run build
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

## Data and privacy

OpenTranscribe does not require an OpenTranscribe account. A selected library
contains readable projects and sessions. Downloaded speech models are kept in a
separate configurable folder, defaulting to `OpenTranscribe/models` under the
documents directory. They are shared by every library, removable from Settings,
and fetched from pinned Whisper and Superwhisper S1-mini revisions on Hugging Face with a
SHA-256 verification step before use. OpenAI transcription is opt-in per
session. The API key is entered in the Settings webview, passed directly to the
native credential command, and never written to frontend persistence or the
library. See [PRIVACY.md](PRIVACY.md).

Recording laws and policies vary. Obtain the required participant consent
before recording.

## Licenses

Unless a file says otherwise, this project is available under either the
Apache License 2.0 or the MIT License, at your option.
