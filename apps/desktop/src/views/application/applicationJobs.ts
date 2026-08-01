import { computed, type Ref } from "vue";

import { native } from "@/core/native";
import type { AppSnapshot, Job, JobProgress } from "@/types/domain";

export function createApplicationJobs(
  snapshot: Ref<AppSnapshot | null>,
  operationError: Ref<string | null>,
) {
  const activeJobs = computed(() => snapshot.value?.active_jobs ?? []);
  const runningJobs = computed(() =>
    activeJobs.value.filter((job) =>
      ["queued", "preparing", "running"].includes(job.state),
    ),
  );

  function upsertJob(job: Job) {
    const jobs = snapshot.value?.active_jobs;

    if (!jobs) {
      return;
    }

    const index = jobs.findIndex((candidate) => candidate.id === job.id);

    if (["completed", "canceled"].includes(job.state)) {
      if (index >= 0) {
        jobs.splice(index, 1);
      }
      return;
    }

    if (index >= 0) {
      jobs[index] = job;
      return;
    }

    jobs.unshift(job);
  }

  function updateJobProgress(jobId: string, progress: JobProgress) {
    const job = snapshot.value?.active_jobs.find(
      (candidate) => candidate.id === jobId,
    );

    if (job) {
      job.progress = progress;
      job.updated_at = new Date().toISOString();
    }
  }

  async function retryJob(jobId: string) {
    operationError.value = null;

    try {
      upsertJob(await native.retryJob(jobId));
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function cancelJob(jobId: string) {
    operationError.value = null;

    try {
      upsertJob(await native.cancelJob(jobId));
    } catch (reason) {
      operationError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  return {
    activeJobs,
    runningJobs,
    upsertJob,
    updateJobProgress,
    retryJob,
    cancelJob,
  };
}
