import type { Job } from "@/types/domain";

const ACTIVE_JOB_STATES: Job["state"][] = ["queued", "preparing", "running"];

export function findActiveTranscriptionJob(
  jobs: Job[],
  sessionId: string,
): Job | null {
  return (
    jobs.find(
      (job) =>
        job.session_id === sessionId &&
        job.kind.startsWith("transcribe_") &&
        ACTIVE_JOB_STATES.includes(job.state),
    ) ?? null
  );
}
