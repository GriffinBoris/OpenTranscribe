# Privacy

OpenTranscribe is designed to work without an OpenTranscribe account or cloud
sync.

## Stored on this device

The library folder selected by the user contains session manifests, notes,
audio artifacts, transcripts, and exports. A hidden `.opentranscribe` directory
contains a rebuildable SQLite search and job index.

Downloaded speech and text-cleanup models are kept separately from meeting libraries. By
default they live in `OpenTranscribe/models` under the documents directory, and
Settings shows the exact path. You can choose another folder; OpenTranscribe
offers to move the installed models with it. Models are not read by the library
index, are shared by every library, and can be deleted and refetched at any
time.

The catalog uses pinned revisions from ggerganov's `whisper.cpp` and
Superwhisper's S1-mini projects on Hugging Face. Every download is SHA-256 verified before use. A
synchronized model folder will treat models as ordinary files and can upload
several hundred megabytes of regenerable data, so choose a non-synced folder if
that matters to you.

## Dictation

Dictation uses a separate temporary audio cache and text history in application
data. Successful dictation deletes its temporary audio. A failed run keeps its
audio for Retry until it is discarded, replaced by a new dictation, or the app
restarts. Cancellation suppresses delivery; an already submitted cloud request
may still finish remotely before its local temporary files are removed.

Optional S1-mini by Superwhisper cleanup runs entirely on the device after speech
recognition. It supports English dictation and keeps the original transcript
alongside cleaned text in history. It does not send text to Superwhisper. Local
models remain loaded between dictations to reduce waiting; model changes and app
shutdown release their processes.

Dictation copies text to the clipboard. Automatic paste requires the original
application to remain active on macOS or the original window on Windows; when
that target cannot be verified, including Linux, the result remains available
for manual paste. Changing fields within the same application is not detected
on macOS. Clipboard contents are replaced when a result is delivered.

## OpenAI transcription

Cloud transcription is optional. When selected, OpenTranscribe sends only the
audio chunks needed for that transcription request to OpenAI. The interface
must label that boundary before a request is made. OpenAI API usage and billing
are associated with the key supplied by the user.

The API key is entered in the Settings webview and passed directly to the native
credential command. It is stored in the operating-system credential vault, is
not retained in frontend persistence, and is never written to the library.
Provider responses stored with a session do not contain the key.

## Telemetry

The current application does not implement product analytics or crash
telemetry. If telemetry is added later, it must be documented here and exposed
through an explicit setting before release.

## Recording consent

The user is responsible for following applicable laws, workplace policies, and
participant-consent requirements. OpenTranscribe must always show an obvious
recording state and will not provide a stealth-recording mode.
