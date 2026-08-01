import type { NativeBridge } from "@/core/native/NativeBridge";
import { desktopOnly, previewState } from "@/core/native/preview/previewState";
import { i18n } from "@/i18n";
import type { Job, JobProgress } from "@/types/domain";

const { t } = i18n.global;

type TranscriptionBridge = Pick<
  NativeBridge,
  | "openAiCredentialStatus"
  | "saveOpenAiApiKey"
  | "removeOpenAiApiKey"
  | "testOpenAiConnection"
  | "enqueueTranscription"
  | "retryJob"
  | "cancelJob"
  | "localModelStatuses"
  | "localModelStoragePath"
  | "chooseLocalModelStorage"
  | "moveLocalModels"
  | "downloadLocalModel"
  | "removeLocalModel"
>;

export const previewTranscriptionBridge = {
  async openAiCredentialStatus() {
    return previewState.credential;
  },

  async saveOpenAiApiKey(apiKey: string) {
    previewState.credential = {
      configured: true,
      masked_key: `•••• ${apiKey.slice(-4)}`,
    };
    return previewState.credential;
  },

  async removeOpenAiApiKey() {
    previewState.credential = { configured: false, masked_key: null };
    return previewState.credential;
  },

  async testOpenAiConnection() {
    return {
      connected: previewState.credential.configured,
      message: previewState.credential.configured
        ? t("native.previewConnected")
        : t("native.addApiKey"),
    };
  },

  async enqueueTranscription(
    sessionId: string,
    provider: "local" | "open_ai",
  ): Promise<Job> {
    const timestamp = new Date().toISOString();
    return {
      id: crypto.randomUUID(),
      session_id: sessionId,
      kind: provider === "open_ai" ? "transcribe_open_ai" : "transcribe_local",
      state: "queued",
      progress: null,
      estimated_cost_usd: null,
      attempt: 1,
      error_message: null,
      created_at: timestamp,
      updated_at: timestamp,
    };
  },

  async retryJob() {
    return desktopOnly(t("native.localDesktopOnly"));
  },

  async cancelJob() {
    return desktopOnly(t("native.localDesktopOnly"));
  },

  async localModelStatuses() {
    return structuredClone(previewState.localModels);
  },

  async localModelStoragePath() {
    return previewState.localModelStoragePath;
  },

  async chooseLocalModelStorage() {
    return "/Users/you/Documents/OpenTranscribe models";
  },

  async moveLocalModels(path: string) {
    previewState.localModelStoragePath = path;
    return path;
  },

  async downloadLocalModel(
    modelId: string,
    onProgress: (progress: JobProgress) => void,
  ) {
    const model = previewState.localModels.find((item) => item.id === modelId);

    if (!model) {
      return desktopOnly(t("native.unknownLocalModel"));
    }

    for (const completionRatio of [0.08, 0.4, 0.72, 1]) {
      onProgress({
        stage: "downloading",
        completed_units: Math.round(model.byte_count * completionRatio),
        total_units: model.byte_count,
        unit: "bytes",
        message: t("models.downloading"),
      });

      if (completionRatio < 1) {
        await new Promise<void>((resolve) => setTimeout(resolve, 600));
      }
    }

    model.installed = true;
    return structuredClone(model);
  },

  async removeLocalModel(modelId: string) {
    const model = previewState.localModels.find((item) => item.id === modelId);

    if (!model) {
      return desktopOnly(t("native.unknownLocalModel"));
    }

    model.installed = false;
    return structuredClone(model);
  },
} satisfies TranscriptionBridge;
