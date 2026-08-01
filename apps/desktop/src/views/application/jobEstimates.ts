import type { Job } from "@/types/domain";

export function estimatedRemainingMilliseconds(
  job: Job,
  now = Date.now(),
): number | null {
  const completed = job.progress?.completed_units ?? 0;
  const total = job.progress?.total_units ?? 0;

  if (completed <= 0 || total <= completed) {
    return null;
  }

  const elapsed = now - new Date(job.created_at).getTime();
  return Math.round((elapsed * (total - completed)) / completed);
}

export function formatUsd(amount: number): string {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: amount < 0.1 ? 4 : 2,
  }).format(amount);
}

export function jobEstimateDetails(
  job: Job,
  translate: (
    key: string,
    parameters?: Record<string, string | number>,
  ) => string,
): string[] {
  const details: string[] = [];
  const remaining = estimatedRemainingMilliseconds(job);

  if (remaining !== null) {
    details.push(
      remaining < 60_000
        ? translate("processing.lessThanMinuteRemaining")
        : translate("processing.aboutMinutesRemaining", {
            minutes: Math.max(1, Math.round(remaining / 60_000)),
          }),
    );
  }

  if (job.kind.includes("open_ai")) {
    details.push(
      job.estimated_cost_usd === null
        ? translate("processing.costUnavailable")
        : translate("processing.estimatedCost", {
            cost: formatUsd(job.estimated_cost_usd),
          }),
    );
  }

  return details;
}
