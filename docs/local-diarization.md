# Local transcription and Nemotron 3

## Using it

In **Settings → Local models**, download **Nemotron 3 speaker recognition**
(400.5 MB). It is independent of the speech models:

- For a new local transcript, choose any installed Whisper model and enable
  **Identify speakers with Nemotron 3** before transcribing.
- For an existing local or cloud transcript, choose **Re-identify speakers**.
  Only Nemotron is required. Keep the original session audio available.
- Leave the checkbox off for ordinary local transcription. Automatic transcription
  after recording and dictation retain their existing defaults.

Re-identification replaces speaker labels and names, including manual renames or
merges. It preserves transcript text, edits, segment IDs, timestamps, word timing
data, and the original speech model/cost attribution. Previous and resulting
transcript snapshots are saved in the session folder. Speaker numbers can change
between runs; rename them after recognizing speakers. This feature processes
saved recordings and imports, rather than live captions or dictation.

## What the NVIDIA model does

[NVIDIA's model card](https://huggingface.co/nvidia/Nemotron-3-Diarization)
describes a 100M-parameter Sortformer model that detects activity for up to eight
anonymous speakers. It accepts 16 kHz mono audio and produces activity at 10 ms
resolution, with speaker channels ordered by first appearance. It does **not**
produce transcript text or identify people by name. Its streaming cache supports
long recordings without resetting speaker identity at each chunk. NVIDIA
provides NeMo and Transformers inference examples; neither runtime was already
part of this desktop application.

OpenTranscribe can pair this diarizer with any installed Whisper model. It uses the
[community ONNX export and Rust runtime](https://github.com/altunenes/parakeet-rs/tree/v0.3.8),
not the gated preview checkpoint or an official NVIDIA ONNX release. The runtime
is pinned to `parakeet-rs = 0.3.8`; the lockfile also pins ONNX Runtime bindings.
The default CPU execution provider avoids requiring CUDA, Python, or a running
server. Whisper continues to use Metal on macOS. ONNX Runtime is statically
linked. Windows also bundles its required DirectML DLL beside the executable;
users do not need a runtime installer. Intel Macs and Linux use checksum-pinned
ONNX Runtime 1.22 static archives through `scripts/prepare-onnx-runtime.mjs`.
Newer prebuilt distributions dropped Intel Mac support and require a newer
Linux GLIBC than our Ubuntu 22.04 baseline. The bindings explicitly target API 21
on every platform to avoid automatic device-selection policies unsupported by
the older static runtime. A one-line vendored `parakeet-rs` patch selects the
compatible `ORT_ENABLE_ALL` optimization level (see `vendor/README.md`). Use `scripts/prepare-sidecar.mjs` for these builds; direct
Cargo commands need `ORT_LIB_PATH` pointing at
`target/onnxruntime-1.22.0-<target-triple>/onnxruntime/lib` after preparation.
The script exports this path for subsequent GitHub Actions steps; local Docker
checks set it in `scripts/run-linux-ci.sh`. Other runtime downloads are handled
by `ort-sys` and verified against its pinned distribution manifest. ONNX telemetry is explicitly disabled
before creating any inference session. On Linux, one environment reference is
retained for the worker's process lifetime: ort's ELF finalizer otherwise calls
`ReleaseEnv` after the static runtime's C++ globals have been destroyed and
crashes at exit. Session and model allocations are still released after each run;
the OS reclaims the process-global environment when the worker exits.

The model comes from [this pinned revision](https://huggingface.co/altunenes/parakeet-rs/tree/4d2a8bc71f5c896ec40faa59732e6716295edaf2/nemotron-3-diarization).
The catalog verifies 400,506,656 bytes and SHA-256
`915e4fa23b0192ed9fadeb1cdd26847df986d50c92012d177be28d0343bbe03a`.
The bundled `resources/nemotron-3/LICENSE` and `NOTICE` retain the OpenMDW-1.1
license and model/conversion attribution. Model artifacts remain in the
ordinary configurable model folder and participate in move, removal, and reset.

## Repository data flow

1. `apps/desktop/src/views/session/components/SessionTranscriptionActions.vue`
   passes an optional diarization model ID alongside the speech model.
   `SessionDiarizationActions.vue` requests a separate job with the current
   transcript ID and revision. `localModelsStore` separates the speech and
   speaker-recognition model choices.
2. `apps/desktop/src-tauri/src/jobs.rs` owns enqueueing, retry, cancellation,
   and progress. Its `jobs/transcription.rs`, `jobs/diarization.rs`, and
   `jobs/finalization.rs` modules run the separate workflows. The repository
   prevents two active jobs against one session. Diarization jobs persist a full
   transcript snapshot, so retry after restart cannot silently target new text.
3. `apps/desktop/src-tauri/src/local_models/transcriber.rs` validates the
   independently selected artifacts; `diarizer.rs` requires only Nemotron.
   Both reuse the supervised runner in `local_models/sidecar.rs`.
4. Protocol version 5 in `crates/transcriber-protocol` carries an optional
   diarizer path in `FileTranscription`, and a separate `DiarizeFile` command
   returning timed `SpeakerTurn` events. Standalone diarization never loads a
   Whisper model and never emits replacement text. Rebuild both bundled workers
   when changing the protocol.
5. `sidecars/local-transcriber/src/diarization.rs` runs the CPU model on decoded
   16 kHz mono audio. With new transcription, Whisper uses word timestamps;
   words are assigned by total overlapping speaker activity, with stable ties,
   then merged into sentences without crossing speaker boundaries. Existing
   timestamp and native whitespace behavior is retained.
6. `apps/desktop/src-tauri/src/storage/repository/diarization.rs` assigns each
   existing segment to the speaker with greatest total overlap. Unmatched
   segments become **Unassigned**. Empty speaker output fails without replacing
   the transcript. Re-identification compares the full current transcript with
   the input snapshot, rejecting concurrent edits, external changes, or a
   replacement transcript. Canceled jobs cannot promote their results.
7. Each rerun writes `transcripts/diarization-runs/<id>/run.json`,
   `provider-response.json`, `input-transcript.json`, and `transcript.json`.
   These record the model hash, source speech run, revisions, raw timed turns,
   and before/after assignments. History is written before updating canonical
   `transcript.json`, Markdown, and search data; an interrupted promotion may
   leave an unapplied history folder. The speech run remains immutable. History
   is inspectable in the session folder; there is no in-app history restore UI.

Cancellation polls every 50 ms independently of worker output and kills the
worker even during silent ONNX inference. Errors, protocol mismatches, and
incomplete output fail the job and preserve the existing transcript. New
transcription splits stage progress between diarization and speech recognition;
standalone diarization shows an indeterminate progress indicator. Whisper debug
logging and ONNX telemetry are disabled.

## Limits and validation

Re-identifying existing transcripts preserves their segment boundaries: a long
segment containing several speakers still receives only one label. A new
transcription with speaker recognition uses word-level alignment for finer
boundaries.

Speaker IDs are local to one run. This is not voice enrollment or cross-meeting
identification. The canonical transcript has one speaker per segment, so
simultaneous speakers are resolved by overlap; this integration does not separate
overlapping voices or recover words Whisper missed. Whisper word timestamps are
estimated, not forced-aligned, and boundary words can receive the wrong speaker.
Nemotron's 10 ms output resolution does not guarantee 10 ms word accuracy.
The sidecar currently decodes the complete recording into memory, as it already
did for Whisper; cached diarization does not remove that existing memory cost.

Automated tests cover overlap alignment and ties, silence, all eight speaker
channels, sentence boundaries, silent cancellation, incomplete output, unchanged
text and timestamps, archived assignments, concurrent changes, restart snapshots,
independent downloads, and frontend availability without speech weights.

The opt-in inference test uses actual model files and a two-speaker WAV. No
weights or user recordings are committed. Run it with:

```bash
OPENTRANSCRIBE_TEST_WHISPER_MODEL=/absolute/path/to/ggml-large-v3-turbo-q5_0.bin \
OPENTRANSCRIBE_TEST_NEMOTRON_MODEL=/absolute/path/to/nemotron3_diar_v3.onnx \
OPENTRANSCRIBE_TEST_AUDIO=/absolute/path/to/two-speaker.wav \
cargo test -p opentranscribe-local-transcriber --test diarized_transcription --locked -- --ignored --nocapture
```

These tests validate standalone repeated diarization without a loaded speech
model, decoded events, nonempty text, ordered timestamps, multiple
speaker labels, and successful process exit. It is a wiring smoke test, not a
diarization accuracy benchmark. macOS validation can use synthesized voices so
no personal meeting audio is needed. Windows/Linux execution and installer
behavior still need their platform CI and packaged-app smoke tests.

### Platform validation

On 2026-09-24, macOS validation passed 189 Rust tests, 20 frontend unit tests,
83 browser tests, and both real-model inference tests. Clippy with warnings
denied, ESLint, type checking, formatting, production builds, and sidecar
preparation passed. Focused workspace tests also verify the speaker controls
at 800, 900, and 1200 pixel widths.

Actual inference is exercised on macOS Apple silicon using synthesized voices,
including a returning speaker. This is a wiring smoke test, not an accuracy
benchmark. No models or recordings are committed. Linux and Windows require
platform CI and packaged-app smoke tests. On Linux with Docker available, use
`task ci:linux:docker MODE=native-test`. On Windows, prepare both sidecars with
`node scripts/prepare-sidecar.mjs`, then run workspace tests and the inference
tests above. The existing S1-mini opt-in model test is unrelated to diarization.

The manually dispatched `Packaged Diarization Smoke` workflow takes a successful
`Native Compile` run ID and tests the worker extracted from that run's Linux
`.deb` on native Ubuntu 22.04. It verifies the pinned model hash and runs
speaker recognition twice without loading Whisper, using generated eSpeak
speech. This checks packaged runtime compatibility, not speaker accuracy.
