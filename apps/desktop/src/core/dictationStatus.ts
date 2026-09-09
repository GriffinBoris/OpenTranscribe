import type { DictationStatus, DictationPhase } from "@/types/domain";

export function emptyDictationStatus(): DictationStatus {
  return {
    id: null,
    phase: "idle",
    provider: null,
    text: null,
    error_message: null,
    elapsed_ms: 0,
    microphone_peak: 0,
    auto_pasted: false,
    approximate_cost_usd: null,
    can_retry: false,
    progress_percent: null,
  };
}

export function isDictationActive(phase: DictationPhase) {
  return ["recording", "transcribing", "cleaning"].includes(phase);
}
