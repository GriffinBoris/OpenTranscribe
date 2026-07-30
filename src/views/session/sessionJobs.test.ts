import type { Job } from "@/types/domain";
import { findActiveTranscriptionJob } from "@/views/session/sessionJobs";

const timestamp = "2026-07-30T12:00:00Z";

function createJob(
  id: string,
  sessionId: string,
  state: Job["state"],
  kind: Job["kind"] = "transcribe_local",
): Job {
  return {
    id,
    session_id: sessionId,
    kind,
    state,
    progress: null,
    attempt: 1,
    error_message: null,
    created_at: timestamp,
    updated_at: timestamp,
  };
}

test("selects the active transcription for the requested session", () => {
  const jobs = [
    createJob("other", "session-b", "running"),
    createJob("requested", "session-a", "preparing", "transcribe_open_ai"),
  ];

  expect(findActiveTranscriptionJob(jobs, "session-a")?.id).toBe("requested");
});

test("ignores failed and unrelated jobs", () => {
  const jobs = [
    createJob("failed", "session-a", "failed"),
    createJob("export", "session-a", "running", "export"),
  ];

  expect(findActiveTranscriptionJob(jobs, "session-a")).toBeNull();
});
