# Platform validation

Automated CI proves shared behavior, protocol compatibility, and native
compilation. A release candidate must also complete this checklist on physical
hardware because CI runners do not expose representative microphones, output
devices, desktop credential vaults, or sleep and device-interruption behavior.

Record the application version, installer type, operating-system version, audio
devices, and result for every run. Keep user audio and API credentials out of
the report.

## Supported targets

| Platform                   | Release level | Required package              |
| -------------------------- | ------------- | ----------------------------- |
| macOS 14+ on Apple Silicon | Supported     | Signed and notarized DMG      |
| macOS 14+ on Intel         | Supported     | Signed and notarized DMG      |
| Windows 10 22H2/11 x64     | Supported     | Signed MSI and NSIS installer |
| Linux x64 with PipeWire    | Beta          | AppImage and deb              |

## Installation and desktop integration

- Install from the packaged artifact, launch from the normal application
  location, and confirm only one OpenTranscribe instance is running.
- Confirm the application, sidebar, taskbar or Dock, installer, and tray icons
  use the same artwork and remain legible in light and dark system themes.
- Confirm close hides the window to the tray, tray reopen restores it, and Quit
  exits only when no recording is active.
- Confirm the custom macOS title bar leaves the traffic lights clickable.
- Confirm the native Windows and Linux title bars retain move, minimize,
  maximize, close, and Windows snap behavior.
- Enable each global-shortcut preset, verify it starts and stops one recording,
  and verify a registration conflict is reported without changing the saved
  preset.

## Credential vault

Use a temporary OpenAI project key and revoke it after validation.

1. Save the key, verify only its last four characters are shown, and test the
   connection.
2. Transcribe a short file, close OpenTranscribe, relaunch it, and transcribe
   again without re-entering the key.
3. Remove the key and verify cloud and live actions become unavailable.
4. Confirm the key never appears in the library, settings JSON, logs,
   diagnostics, screenshots, crash output, or process arguments.
5. macOS: use the same signed identity for every run and confirm Keychain does
   not request approval repeatedly.
6. Windows: confirm the entry is stored in Windows Credential Manager for the
   current user.
7. Linux: confirm save, relaunch, and removal with a Secret Service-compatible
   keyring both unlocked and locked. When unavailable, the application must
   explain the requirement and leave local transcription usable.

## Recording matrix

Run every row for at least ten seconds, then play the saved mix and individual
source tracks.

| Capture                      | Built-in | USB            | Bluetooth or wireless | Expected result                           |
| ---------------------------- | -------- | -------------- | --------------------- | ----------------------------------------- |
| Microphone only              | Required | Required       | Required              | One microphone track and a playable mix   |
| System output only           | Required | When available | Required              | One system track and a playable mix       |
| Microphone and system output | Required | Required       | Required              | Separate tracks plus an aligned mix       |
| Pause and resume             | Required | Required       | Required              | Paused time is absent from media duration |

Also verify:

- source labels identify microphone and computer audio correctly;
- meters remain responsive without changing recorded duration;
- changing the default output during capture reports an actionable error or
  continues on the explicitly selected device, never silently switches;
- unplugging the selected microphone preserves recovery audio and explains the
  failure;
- sleep and wake, screen lock, and Bluetooth disconnect do not corrupt an
  already finalized recording;
- low disk space fails visibly and preserves every complete recovery chunk;
- macOS permission denial identifies Screen & System Audio Recording and the
  recovery button opens the correct settings pane;
- PipeWire absence disables system output without disabling microphone capture.

## Transcription matrix

Use one short English fixture, one multilingual fixture, and one recording with
two speakers.

| Path                              | macOS arm64 | macOS x64 | Windows x64 | Linux x64 |
| --------------------------------- | ----------- | --------- | ----------- | --------- |
| Local Fast model                  | Required    | Required  | Required    | Required  |
| Local Balanced model              | Required    | Required  | Required    | Required  |
| Local Best model                  | Smoke       | Smoke     | Smoke       | Smoke     |
| OpenAI GPT Transcribe             | Required    | Required  | Required    | Required  |
| OpenAI diarization                | Required    | Required  | Required    | Required  |
| OpenAI live microphone            | Required    | Required  | Required    | Required  |
| OpenAI live microphone and system | Required    | Required  | Required    | Required  |

For each path, verify progress advances, cancellation leaves the recording
usable, retry creates a new immutable run, retranscription replaces the current
canonical transcript without deleting prior runs, timestamps seek correctly,
and Markdown, text, JSON, SRT, and VTT exports open successfully.

Disconnect the network during cloud transcription and live transcription. The
recording must continue and finalize even when the provider fails. Reconnect and
retry from the saved recording.

## Packaging and update gates

- Verify the packaged local-transcriber executable matches the application
  architecture and completes its protocol handshake.
- Download and integrity-check every local model from the packaged application.
- Confirm the default library resolves to the current user's
  `Documents/OpenTranscribe` directory and a custom removable or synced folder
  remains readable after relaunch.
- Verify signatures before installation: `codesign` and notarization on macOS,
  Authenticode on Windows, and updater signatures on every supported target.
- Install the previous release, apply the signed update, relaunch, and verify
  settings, credential access, projects, recordings, notes, and transcripts are
  preserved.
- Do not publish updater metadata until every intended artifact, checksum, and
  updater signature has been uploaded and validated.
