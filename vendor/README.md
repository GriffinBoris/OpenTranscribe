# Patched dependencies

`parakeet-rs` is the published 0.3.8 crate (source, manifests, README and
licenses), with one compatibility change in `src/execution.rs`:
`GraphOptimizationLevel::Level3` becomes `GraphOptimizationLevel::All`.
The newer ort bindings map Level3 to ORT_ENABLE_LAYOUT (3), which ONNX Runtime
1.22 rejects. All maps to ORT_ENABLE_ALL (99), supported by both our 1.22 and
1.28 runtimes. Retain this patch until upstream supports our runtime baseline.

Bindings use API 21 so ort does not enable the API 22 automatic device
selection policy, which crashes in the older static runtime. CPU remains the
execution provider.

The runtime compatibility is exercised by the local transcriber's opt-in
`diarized_transcription` tests against both runtime versions. The vendored
source remains otherwise unchanged to keep the upstream difference reviewable.
