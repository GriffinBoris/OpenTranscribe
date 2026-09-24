import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, test, vi } from "vitest";

import { native } from "@/core/native";
import { createPreviewTranscript } from "@/core/native/preview/previewTranscript";
import type { Job } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useSessionStore } from "./sessionStore";

vi.mock("@/core/native", () => ({
  native: { enqueueDiarization: vi.fn(), enqueueTranscription: vi.fn() },
}));

const job: Job = {
  id: "labels",
  session_id: "session",
  kind: "diarize_local",
  state: "queued",
  progress: null,
  estimated_cost_usd: null,
  attempt: 1,
  error_message: null,
  created_at: "2026-09-24T12:00:00Z",
  updated_at: "2026-09-24T12:00:00Z",
};

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
});

test("queues speaker recognition against the current transcript revision without changing its text", async () => {
  vi.mocked(native.enqueueDiarization).mockResolvedValue(job);
  const store = useSessionStore();
  const transcript = createPreviewTranscript("session");
  store.transcripts.session = transcript;
  await store.diarize("session", "nemotron-3-diarization");
  expect(native.enqueueDiarization).toHaveBeenCalledWith(
    "session",
    "nemotron-3-diarization",
    transcript.id,
    transcript.revision,
  );
  expect(native.enqueueTranscription).not.toHaveBeenCalled();
  expect(store.transcripts.session).toEqual(transcript);
});

test("passes the separately selected diarizer with any speech model", async () => {
  vi.mocked(native.enqueueTranscription).mockResolvedValue({
    ...job,
    kind: "transcribe_local",
  });
  const store = useSessionStore();
  await store.transcribeLocally(
    "session",
    "whisper-small-q5_1",
    "nemotron-3-diarization",
  );
  expect(native.enqueueTranscription).toHaveBeenLastCalledWith(
    "session",
    "local",
    "whisper-small-q5_1",
    "nemotron-3-diarization",
  );
  await store.transcribeLocally("session", "whisper-medium-q5_0");
  expect(native.enqueueTranscription).toHaveBeenLastCalledWith(
    "session",
    "local",
    "whisper-medium-q5_0",
    undefined,
  );
});

test("surfaces a stale revision without replacing the displayed transcript", async () => {
  vi.mocked(native.enqueueDiarization).mockRejectedValue(
    new Error("Transcript changed"),
  );
  const store = useSessionStore();
  const transcript = createPreviewTranscript("session");
  store.transcripts.session = transcript;
  await store.diarize("session", "nemotron-3-diarization");
  expect(useApplicationStore().operationError).toBe("Transcript changed");
  expect(store.transcripts.session).toEqual(transcript);
});
