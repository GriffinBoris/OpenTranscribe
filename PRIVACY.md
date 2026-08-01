# Privacy

OpenTranscribe is designed to work without an OpenTranscribe account or cloud
sync.

## Stored on this device

The library folder selected by the user contains session manifests, notes,
audio artifacts, transcripts, and exports. A hidden `.opentranscribe` directory
contains a rebuildable SQLite search and job index.

Downloaded speech models are kept separately in `OpenTranscribe/models` under
the documents directory, so a large download stays visible and removable
without knowing platform-specific data paths. Models are not part of any
meeting library: they are not read by the library index, they are shared by
every library, and switching libraries does not download them again. They
contain no meeting content and can be deleted and refetched at any time.

Because that location sits under the documents directory, a synchronized
documents folder will treat models as ordinary files and upload them. Remove
the models from the application, or exclude `OpenTranscribe/models` from
synchronization, to keep several hundred megabytes of regenerable data out of
a backup or cloud quota.

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
