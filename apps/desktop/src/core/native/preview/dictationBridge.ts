import type { NativeBridge } from "@/core/native/NativeBridge";
import {
  emptyDictationStatus,
  isDictationActive,
} from "@/core/dictationStatus";
import {
  previewSnapshot,
  previewState,
} from "@/core/native/preview/previewState";
import type { DictationHistoryEntry, DictationStatus } from "@/types/domain";

let history: DictationHistoryEntry[] = [];
let completion: ReturnType<typeof setTimeout> | null = null;
let startedAt = 0;

function publish(status: DictationStatus) {
  previewState.dictationStatus = status;
  previewState.dictationStatusListener?.(status);
  previewState.appEventListener?.({
    type: "dictation_state_changed",
    payload: status,
  });
  return status;
}

function finish(id: string) {
  if (previewState.dictationStatus.id !== id) return;
  const settings = previewSnapshot().settings;
  const failed =
    new URLSearchParams(window.location.search).get("dictationResult") ===
    "failed";
  if (failed) {
    publish({
      ...previewState.dictationStatus,
      phase: "failed",
      can_retry: true,
      error_message: "Transcription could not finish. Try again.",
    });
    return;
  }
  const text = settings.dictation_cleanup_enabled
    ? "Send the draft on Tuesday."
    : "Um send the draft on Monday, no Tuesday.";
  history = [
    {
      id,
      text,
      raw_text: settings.dictation_cleanup_enabled
        ? "Um send the draft on Monday, no Tuesday."
        : null,
      cleanup_model_id: settings.dictation_cleanup_enabled
        ? "s1-mini-q4_k_m"
        : null,
      created_at: new Date().toISOString(),
      provider: settings.dictation_provider,
      model_id: settings.dictation_local_model_id ?? "gpt-transcribe",
      usage: null,
      approximate_cost_usd: null,
    },
    ...history.filter((entry) => entry.id !== id),
  ];
  publish({
    ...previewState.dictationStatus,
    phase: "completed",
    text,
    auto_pasted: false,
    can_retry: false,
    error_message: null,
    progress_percent: null,
  });
}

function process() {
  const id = previewState.dictationStatus.id!;
  publish({
    ...previewState.dictationStatus,
    phase: "transcribing",
    error_message: null,
    can_retry: false,
    elapsed_ms: Date.now() - startedAt,
  });
  completion = setTimeout(() => {
    if (previewState.dictationStatus.id !== id) return;
    if (previewSnapshot().settings.dictation_cleanup_enabled) {
      publish({ ...previewState.dictationStatus, phase: "cleaning" });
      completion = setTimeout(() => finish(id), 400);
    } else finish(id);
  }, 650);
  return previewState.dictationStatus;
}

export const previewDictationBridge: Pick<
  NativeBridge,
  | "toggleDictation"
  | "retryDictation"
  | "dictationSettings"
  | "dictationStatus"
  | "dictationHistory"
  | "clearDictationHistory"
  | "cancelDictation"
  | "dismissDictation"
  | "subscribeDictationStatus"
> = {
  async toggleDictation() {
    if (previewState.dictationStatus.phase === "recording") return process();
    if (isDictationActive(previewState.dictationStatus.phase))
      throw new Error("Cancel or wait for this dictation to finish.");
    const settings = previewSnapshot().settings;
    const fixture = new URLSearchParams(window.location.search).has(
      "dictation",
    );
    if (
      !fixture &&
      settings.dictation_provider === "local" &&
      !previewState.localModels.some(
        (model) =>
          model.installed &&
          model.id === settings.dictation_local_model_id &&
          model.preset !== "cleanup",
      )
    )
      throw new Error(
        "Choose an installed speech model in Settings → Dictation.",
      );
    startedAt = Date.now();
    return publish({
      ...emptyDictationStatus(),
      id: crypto.randomUUID(),
      phase: "recording",
      provider: settings.dictation_provider,
      microphone_peak: 0.58,
    });
  },
  async retryDictation() {
    if (!previewState.dictationStatus.can_retry)
      throw new Error("There is no failed dictation to retry.");
    return process();
  },
  async dictationSettings() {
    return previewSnapshot().settings;
  },
  async dictationStatus() {
    return previewState.dictationStatus;
  },
  async dictationHistory() {
    return history;
  },
  async clearDictationHistory() {
    history = [];
  },
  async cancelDictation() {
    if (completion !== null) clearTimeout(completion);
    return publish(emptyDictationStatus());
  },
  async dismissDictation() {
    await previewDictationBridge.cancelDictation();
  },
  async subscribeDictationStatus(listener) {
    previewState.dictationStatusListener = listener;
  },
};
