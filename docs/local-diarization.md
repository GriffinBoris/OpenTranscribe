# Local transcription and Nemotron 3

## Using it

In **Settings → Local models**, download **Whisper + Nemotron 3**. This installs
the 400.5 MB Nemotron ONNX artifact and the shared 574.0 MB Whisper Best artifact
(974.5 MB total). An already installed Best model is reused. In a saved session's
transcription menu, choose **Whisper + Nemotron 3**. Audio stays on the computer.
The resulting transcript supports the existing speaker renaming, seeking,
editing, and export workflows.

Removing this option removes Nemotron only. Removing Best makes the combination
unavailable; Settings keeps Nemotron removable and offers Download to restore
the missing dependency. Existing transcription defaults stay unchanged. This
option is for saved meetings and imports, not live captions or dictation.

## What the NVIDIA model does

[NVIDIA's model card](https://huggingface.co/nvidia/Nemotron-3-Diarization)
describes a 100M-parameter Sortformer model that detects activity for up to eight
anonymous speakers. It accepts 16 kHz mono audio and produces activity at 10 ms
resolution, with speaker channels ordered by first appearance. It does **not**
produce transcript text or identify people by name. Its streaming cache supports
long recordings without resetting speaker identity at each chunk. NVIDIA
provides NeMo and Transformers inference examples; neither runtime was already
part of this desktop application.

OpenTranscribe pairs this diarizer with Whisper Large v3 Turbo Q5. It uses the
[community ONNX export and Rust runtime](https://github.com/altunenes/parakeet-rs/tree/v0.3.8),
not the gated preview checkpoint or an official NVIDIA ONNX release. The runtime
is pinned to `parakeet-rs = 0.3.8`; the lockfile also pins ONNX Runtime bindings.
The default CPU execution provider avoids requiring CUDA, Python, or a running
server. Whisper continues to use Metal on macOS. ONNX Runtime is statically
linked by its default build, so sidecar packaging does not require a new runtime
installer. Build-time binary downloads are handled by `ort-sys` and verified
against its pinned distribution manifest. ONNX telemetry is explicitly disabled
before creating any inference session.

The model comes from [this pinned revision](https://huggingface.co/altunenes/parakeet-rs/tree/4d2a8bc71f5c896ec40faa59732e6716295edaf2/nemotron-3-diarization).
The catalog verifies 400,506,656 bytes and SHA-256
`915e4fa23b0192ed9fadeb1cdd26847df986d50c92012d177be28d0343bbe03a`.
The bundled `resources/nemotron-3/LICENSE` and `NOTICE` retain the OpenMDW-1.1
license and model/conversion attribution. Both model artifacts remain in the
ordinary configurable model folder and participate in move, removal, and reset.

## Repository data flow

1. The Vue session view uses `localModelsStore` to list installed, usable models
   and sends the selected model ID through the typed native bridge. Settings
   gets model metadata from the Rust catalog. Dictation filters out the combined
   meeting profile, with a matching native validation check.
2. Native jobs resolve finalized audio through the library repository and call
   `LocalTranscriptionService`. The service validates both installed artifacts,
   resolves the combination to its Whisper and Nemotron paths, and launches the
   supervised `local-transcriber` executable.
3. Protocol version 4 carries the optional diarizer path in `FileTranscription`
   and an optional speaker label on each segment. The desktop and both bundled
   workers use the shared protocol crate and must be built together.
4. The sidecar decodes the selected audio once, downmixes it to mono, and resamples
   to 16 kHz with the existing audio decoder. Nemotron runs first, using the
   runtime's offline profile and persistent speaker cache within the recording.
   The application still performs this work only after recording finalization.
5. Whisper enables token timestamps and word splitting for the combined profile.
   The alignment module assigns each word to the speaker with the greatest total
   overlapping activity. Ties choose the earlier-arriving speaker. Words without
   matching activity remain unassigned. Sentence merging stops at speaker changes,
   punctuation, long pauses, and the existing duration limit.
6. The desktop preserves model IDs, the diarizer hash, and timed/labeled segments
   in the provider response. `build_bundle` creates stable speaker identities for
   that immutable run, then the existing repository persists the run and promotes
   the canonical transcript. No library schema migration is needed: speakers and
   integer-millisecond segment times already exist in the domain model.

The sidecar's output reader polls cancellation even when model loading/inference
is silent. Errors, protocol mismatches, and incomplete output fail the job and
terminate the child. A failure does not persist a partially completed transcript
or affect the saved recording. Progress allocates half to diarization and half
to Whisper; this is stage progress, not a prediction of elapsed processing time.
Whisper's native debug logging is disabled because it can expose recognized text;
inference errors are delivered through the protocol.

## Limits and validation

Speaker IDs are local to one run. This is not voice enrollment or cross-meeting
identification. The canonical transcript has one speaker per segment, so
simultaneous speakers are resolved by overlap; this integration does not separate
overlapping voices or recover words Whisper missed. Whisper word timestamps are
estimated, not forced-aligned, and boundary words can receive the wrong speaker.
Nemotron's 10 ms output resolution does not guarantee 10 ms word accuracy.
The sidecar currently decodes the complete recording into memory, as it already
did for Whisper; cached diarization does not remove that existing memory cost.

Automated tests cover alignment, overlapping turns and ties, silence, all eight
speaker channels, sentence boundaries, protocol transport, silent cancellation,
incomplete completion, catalog dependencies, and UI availability after dependency
removal. The browser test exercises combined installation and repair.

The opt-in inference test uses actual model files and a two-speaker WAV. No
weights or user recordings are committed. Run it with:

```bash
OPENTRANSCRIBE_TEST_WHISPER_MODEL=/absolute/path/to/ggml-large-v3-turbo-q5_0.bin \
OPENTRANSCRIBE_TEST_NEMOTRON_MODEL=/absolute/path/to/nemotron3_diar_v3.onnx \
OPENTRANSCRIBE_TEST_AUDIO=/absolute/path/to/two-speaker.wav \
cargo test -p opentranscribe-local-transcriber --test diarized_transcription --locked -- --ignored --nocapture
```

This test validates decoded events, nonempty text, ordered timestamps, multiple
speaker labels, and successful process exit. It is a wiring smoke test, not a
diarization accuracy benchmark. macOS validation can use synthesized voices so
no personal meeting audio is needed. Windows/Linux execution and installer
behavior still need their platform CI and packaged-app smoke tests.

### Implementation validation — 2026-09-24

On macOS Apple silicon: 182 native tests, 16 frontend unit tests, and 82 browser
tests passed, including light/dark accessibility checks and the model dependency
repair flow at 900 × 640. Formatting, Clippy with warnings denied, ESLint, Vue
type checking, the frontend production build, `scripts/prepare-sidecar.mjs`, and
`cargo build --workspace --locked` passed. Both downloaded artifact hashes matched
the catalog. The opt-in inference test passed with Whisper Best and Nemotron on
roughly 33 seconds of synthesized two-voice audio, including a returning speaker.
The normal native suite skips two model-dependent tests; the Nemotron test was
run separately with the downloaded weights. The existing S1-mini model test was
not run.

Linux native validation was unavailable because the local Docker daemon was not
running. With Docker available, run `task ci:linux:docker MODE=native-test`.
On Windows, prepare the sidecars with `node scripts/prepare-sidecar.mjs`, then run
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked`, and the opt-in inference test above. These
checks do not replace a signed/packaged application smoke test on each platform.
