import type { NativeBridge } from "@/core/native/NativeBridge";
import {
  desktopOnly,
  previewSnapshot,
  previewState,
} from "@/core/native/preview/previewState";
import { createPreviewSnapshot } from "@/core/native/preview/previewData";
import { i18n } from "@/i18n";
import type { ExportFormat } from "@/types/domain";

const { t } = i18n.global;

type SessionBridge = Pick<
  NativeBridge,
  | "updateTranscriptSegment"
  | "renameSpeaker"
  | "mergeSpeakers"
  | "exportSession"
  | "sessionWorkspace"
  | "sessionAudioSources"
  | "sessionWaveform"
  | "recoverRecording"
  | "revealSession"
  | "renameSession"
  | "moveSession"
  | "trashedSessions"
  | "trashSession"
  | "restoreSession"
  | "emptyTrash"
  | "saveNotes"
>;

export const previewSessionBridge = {
  async updateTranscriptSegment() {
    return desktopOnly(t("native.editingDesktopOnly"));
  },

  async renameSpeaker() {
    return desktopOnly(t("native.editingDesktopOnly"));
  },

  async mergeSpeakers() {
    return desktopOnly(t("native.editingDesktopOnly"));
  },

  async exportSession(_sessionId: string, format: ExportFormat) {
    return {
      format,
      path: t("native.previewExportPath", {
        extension: format === "markdown" ? "md" : format,
      }),
    };
  },

  async sessionWorkspace(sessionId: string) {
    if (previewState.trashedSessions.has(sessionId)) {
      throw new Error(t("native.requestedItemMissing"));
    }

    return null;
  },

  async sessionAudioSources() {
    return [];
  },

  async sessionWaveform() {
    return [];
  },

  async recoverRecording(sessionId: string) {
    const session = createPreviewSnapshot().recent_sessions.find(
      (candidate) => candidate.id === sessionId,
    );

    if (!session) {
      return desktopOnly(t("native.editingDesktopOnly"));
    }

    return {
      ...session,
      lifecycle: "recovered" as const,
      recovery_state: "recovered" as const,
    };
  },

  async revealSession() {},

  async renameSession(sessionId: string, title: string) {
    const session = previewSnapshot().recent_sessions.find(
      (candidate) => candidate.id === sessionId,
    );

    if (!session) {
      return desktopOnly(t("native.editingDesktopOnly"));
    }

    const normalizedTitle = title.trim();

    if (!normalizedTitle) {
      throw new Error("meeting title cannot be empty");
    }

    const renamed = {
      ...session,
      title: normalizedTitle,
      revision: session.revision + 1,
    };
    previewState.sessionTitles.set(sessionId, {
      title: renamed.title,
      revision: renamed.revision,
    });
    previewState.appEventListener?.({ type: "library_changed" });
    return renamed;
  },

  async moveSession(sessionId: string, projectId: string | null) {
    const session = previewSnapshot().recent_sessions.find(
      (candidate) => candidate.id === sessionId,
    );

    if (!session) {
      return desktopOnly(t("native.editingDesktopOnly"));
    }

    const moved = {
      ...session,
      project_id: projectId,
      revision: session.revision + 1,
    };
    previewState.sessionMoves.set(sessionId, {
      projectId,
      revision: moved.revision,
    });
    previewState.appEventListener?.({ type: "library_changed" });
    return moved;
  },

  async trashedSessions() {
    return [...previewState.trashedSessions.values()];
  },

  async trashSession(sessionId: string) {
    const session = previewSnapshot().recent_sessions.find(
      (candidate) => candidate.id === sessionId,
    );

    if (!session) {
      throw new Error(t("native.requestedItemMissing"));
    }

    const trashed = {
      ...session,
      lifecycle: "trashed" as const,
      revision: session.revision + 1,
    };
    previewState.trashedSessions.set(sessionId, trashed);
    previewState.appEventListener?.({ type: "library_changed" });
    return trashed;
  },

  async restoreSession(sessionId: string) {
    const session = previewState.trashedSessions.get(sessionId);

    if (!session) {
      throw new Error(t("native.requestedItemMissing"));
    }

    const restored = {
      ...session,
      lifecycle: "ready" as const,
      revision: session.revision + 1,
    };
    previewState.trashedSessions.delete(sessionId);
    previewState.sessionMoves.set(sessionId, {
      projectId: restored.project_id,
      revision: restored.revision,
    });
    previewState.appEventListener?.({ type: "library_changed" });
    return restored;
  },

  async emptyTrash() {
    previewState.trashedSessions.clear();
    previewState.appEventListener?.({ type: "library_changed" });
  },

  async saveNotes() {
    return null;
  },
} satisfies SessionBridge;
