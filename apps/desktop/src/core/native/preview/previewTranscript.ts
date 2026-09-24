import type { Transcript } from "@/types/domain";

export function createPreviewTranscript(sessionId: string): Transcript {
  return {
    schema_version: 1,
    id: "preview-transcript",
    session_id: sessionId,
    revision: 2,
    source_run_id: "preview-speech-run",
    detected_language: "en",
    language_hint: null,
    speakers: [{ id: "speaker-a", display_name: "Alice", source: "manual" }],
    segments: [
      {
        id: "segment-a",
        start_ms: 0,
        end_ms: 3000,
        text: "These corrected meeting notes should stay unchanged.",
        speaker_id: "speaker-a",
        source: "imported",
        edited: true,
        word_timings: null,
      },
    ],
    updated_at: "2026-09-24T12:00:00Z",
  };
}
