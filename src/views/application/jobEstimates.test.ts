import { describe, expect, it } from "vitest";

import type { Job } from "@/types/domain";
import {
  estimatedRemainingMilliseconds,
  formatUsd,
  jobEstimateDetails,
} from "@/views/application/jobEstimates";

function job(completed: number, total: number | null): Job {
  return {
    id: "job",
    session_id: "session",
    kind: "transcribe_open_ai",
    state: "running",
    progress: {
      stage: "transcribing",
      completed_units: completed,
      total_units: total,
      unit: "items",
      message: "Transcribing",
    },
    estimated_cost_usd: 0.0045,
    attempt: 1,
    error_message: null,
    created_at: "2026-07-31T12:00:00.000Z",
    updated_at: "2026-07-31T12:00:30.000Z",
  };
}

describe("estimatedRemainingMilliseconds", () => {
  it("estimates remaining time from completed work and elapsed time", () => {
    expect(
      estimatedRemainingMilliseconds(
        job(1, 4),
        Date.parse("2026-07-31T12:01:00.000Z"),
      ),
    ).toBe(180_000);
  });

  it("waits until measurable progress is available", () => {
    expect(estimatedRemainingMilliseconds(job(0, 4))).toBeNull();
    expect(estimatedRemainingMilliseconds(job(1, null))).toBeNull();
    expect(estimatedRemainingMilliseconds(job(4, 4))).toBeNull();
  });
});

describe("formatUsd", () => {
  it("keeps small transcription estimates useful", () => {
    expect(formatUsd(0.0045)).toBe("$0.0045");
    expect(formatUsd(0.42)).toBe("$0.42");
  });
});

describe("jobEstimateDetails", () => {
  it("includes cloud cost without adding it to local jobs", () => {
    const translate = (
      key: string,
      parameters?: Record<string, string | number>,
    ) => `${key}:${parameters?.cost ?? ""}`;
    const cloudJob = job(0, 4);
    const localJob = { ...cloudJob, kind: "transcribe_local" };

    expect(jobEstimateDetails(cloudJob, translate)).toContain(
      "processing.estimatedCost:$0.0045",
    );
    expect(jobEstimateDetails(localJob, translate)).toEqual([]);
  });
});
