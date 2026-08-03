import type { NativeBridge } from "@/core/native/NativeBridge";
import {
  currentRecordingStatus,
  previewSnapshot,
  previewState,
} from "@/core/native/preview/previewState";
import { i18n } from "@/i18n";
import type {
  AppSettings,
  AudioDevices,
  DictationHistoryEntry,
  DictationStatus,
  SearchFilters,
} from "@/types/domain";

const { t } = i18n.global;

type ApplicationBridge = Pick<
  NativeBridge,
  | "bootstrap"
  | "saveSettings"
  | "resetApplicationSettings"
  | "deleteAllApplicationData"
  | "initializeLibrary"
  | "chooseLibrary"
  | "createProject"
  | "searchLibrary"
  | "importMedia"
  | "audioDevices"
  | "openSystemAudioPermissionSettings"
  | "configureGlobalShortcut"
  | "toggleDictation"
  | "dictationStatus"
  | "dictationShortcut"
  | "dictationHistory"
  | "clearDictationHistory"
  | "cancelDictation"
  | "dismissDictation"
  | "subscribeDictationStatus"
  | "subscribe"
>;

export const previewApplicationBridge = {
  async bootstrap() {
    return previewSnapshot();
  },

  async saveSettings(updates: Partial<AppSettings>) {
    const settings: AppSettings = {
      ...previewSnapshot().settings,
      ...updates,
      revision: previewSnapshot().settings.revision + 1,
    };
    previewState.settings = settings;
    return settings;
  },

  async resetApplicationSettings() {
    const settings: AppSettings = {
      ...previewSnapshot().settings,
      setup_completed: false,
      recording_mode: "record_only",
      microphone_device_id: null,
      capture_system_audio: false,
      recording_project_selection: { kind: "automatic" },
      global_shortcut_enabled: false,
      dictation_shortcut_enabled: true,
      dictation_shortcut: "Alt+Space",
      dictation_provider: "local",
      dictation_local_model_id: null,
      dictation_openai_model: "gpt_transcribe",
      dictation_auto_paste: true,
      appearance: { theme: "system", reduced_motion: false },
    };
    previewState.settings = settings;
    return settings;
  },

  async deleteAllApplicationData() {
    window.location.assign("/?firstRun=1&resetReady=1");
  },

  async initializeLibrary(path: string) {
    return previewSnapshot(path);
  },

  async chooseLibrary() {
    return "/Users/you/Documents/OpenTranscribe";
  },

  async createProject(name: string) {
    const timestamp = new Date().toISOString();
    return {
      schema_version: 1,
      id: crypto.randomUUID(),
      name,
      revision: 1,
      glossary: [],
      openai_profile_id: null,
      language_hint: null,
      created_at: timestamp,
      updated_at: timestamp,
    };
  },

  async searchLibrary(query: string, filters: SearchFilters) {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    const sessions = previewSnapshot().recent_sessions;

    return {
      results: sessions
        .filter(
          (session) =>
            session.title.toLocaleLowerCase().includes(normalizedQuery) &&
            (!filters.project_id ||
              session.project_id === filters.project_id) &&
            (!filters.content_kinds.length ||
              filters.content_kinds.includes("session")),
        )
        .map((session) => ({
          session_id: session.id,
          session_title: session.title,
          kind: "session" as const,
          excerpt: session.title,
          timestamp_ms: null,
        })),
      next_cursor: null,
    };
  },

  async importMedia() {
    return null;
  },

  async audioDevices(): Promise<AudioDevices> {
    const requestedPermission = new URLSearchParams(window.location.search).get(
      "systemAudioPermission",
    );

    if (
      requestedPermission === "granted" ||
      requestedPermission === "required"
    ) {
      previewState.systemAudioPermission = requestedPermission;
    }

    const permissionState = previewState.systemAudioPermission;
    const permissionSettingsAvailable = permissionState !== null;

    return {
      microphones: [
        {
          id: "preview-microphone",
          label: t("native.defaultMicrophone"),
          is_default: true,
        },
      ],
      system_audio_available: permissionSettingsAvailable,
      system_audio_permission_granted:
        permissionState === "granted"
          ? true
          : permissionState === "required"
            ? false
            : null,
      system_audio_permission_settings_available: permissionSettingsAvailable,
    };
  },

  async openSystemAudioPermissionSettings() {},

  async configureGlobalShortcut(shortcutId, shortcut, onTrigger) {
    const existingListener =
      previewState.globalShortcutListeners.get(shortcutId);
    const eventName = `opentranscribe:preview-global-shortcut:${shortcutId}`;

    if (!shortcut) {
      if (existingListener) {
        window.removeEventListener(eventName, existingListener);
        previewState.globalShortcutListeners.delete(shortcutId);
      }

      return;
    }

    if (
      new URLSearchParams(window.location.search).has("globalShortcutError")
    ) {
      throw new Error(t("native.previewGlobalShortcutUnavailable"));
    }

    if (existingListener === onTrigger) {
      return;
    }

    previewState.globalShortcutListeners.set(shortcutId, onTrigger);
    window.addEventListener(eventName, onTrigger);

    if (existingListener) {
      window.removeEventListener(eventName, existingListener);
    }
  },

  async toggleDictation(): Promise<DictationStatus> {
    const isRecording = previewState.dictationStatus.phase === "recording";
    previewState.dictationStatus = isRecording
      ? {
          ...previewState.dictationStatus,
          phase: "completed",
          text: "Preview dictation.",
          auto_pasted: true,
          approximate_cost_usd: null,
        }
      : {
          id: crypto.randomUUID(),
          phase: "recording",
          provider: previewState.settings.dictation_provider ?? "local",
          text: null,
          error_message: null,
          elapsed_ms: 0,
          microphone_peak: 0.58,
          auto_pasted: false,
          approximate_cost_usd: null,
        };
    previewState.dictationStatusListener?.(previewState.dictationStatus);
    return previewState.dictationStatus;
  },

  async dictationStatus(): Promise<DictationStatus> {
    return previewState.dictationStatus;
  },

  async dictationShortcut() {
    return previewState.settings.dictation_shortcut ?? "Alt+Space";
  },

  async dictationHistory(): Promise<DictationHistoryEntry[]> {
    return [];
  },

  async clearDictationHistory() {},

  async cancelDictation(): Promise<DictationStatus> {
    previewState.dictationStatus = {
      id: null,
      phase: "idle",
      provider: null,
      text: null,
      error_message: null,
      elapsed_ms: 0,
      microphone_peak: 0,
      auto_pasted: false,
      approximate_cost_usd: null,
    };
    previewState.dictationStatusListener?.(previewState.dictationStatus);
    return previewState.dictationStatus;
  },

  async dismissDictation() {},

  async subscribeDictationStatus(onStatus) {
    previewState.dictationStatusListener = onStatus;
  },

  async subscribe(onEvent) {
    previewState.appEventListener = onEvent;
    window.setInterval(() => {
      const status = currentRecordingStatus();

      if (status) {
        onEvent({ type: "recording_levels", payload: status });
      }
    }, 50);
  },
} satisfies ApplicationBridge;
