import { ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import type {
  ExportFormat,
  ExportResult,
  SessionWorkspace,
  Transcript,
} from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { findActiveTranscriptionJob } from "@/views/session/sessionJobs";

export const useSessionStore = defineStore("session", () => {
  const application = useApplicationStore();
  const transcripts = ref<Record<string, Transcript>>({});
  const notesHashes = ref<Record<string, string>>({});

  function transcriptionJobForSession(sessionId: string) {
    return findActiveTranscriptionJob(application.activeJobs, sessionId);
  }

  async function transcribeWithOpenAi(sessionId: string) {
    application.operationError = null;
    try {
      const job = await native.enqueueTranscription(
        sessionId,
        "open_ai",
        "gpt-4o-transcribe-diarize",
      );
      application.upsertJob(job);
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function transcribeLocally(sessionId: string, modelId: string) {
    application.operationError = null;
    try {
      const job = await native.enqueueTranscription(
        sessionId,
        "local",
        modelId,
      );
      application.upsertJob(job);
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function loadWorkspace(
    sessionId: string,
  ): Promise<SessionWorkspace | null> {
    application.operationError = null;

    try {
      const workspace = await native.sessionWorkspace(sessionId);

      if (workspace) {
        if (workspace.transcript) {
          transcripts.value[sessionId] = workspace.transcript;
        }
      }

      return workspace;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    }
  }

  function startNotesEditing(sessionId: string, contentHash: string) {
    notesHashes.value[sessionId] = contentHash;
  }

  async function saveNotes(sessionId: string, markdown: string) {
    const expectedHash = notesHashes.value[sessionId];

    if (expectedHash === undefined) {
      return false;
    }

    application.operationError = null;

    try {
      const saved = await native.saveNotes(sessionId, markdown, expectedHash);

      if (saved) {
        notesHashes.value[sessionId] = saved.content_hash;
      }

      return Boolean(saved);
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function updateTranscriptSegment(
    sessionId: string,
    segmentId: string,
    text: string,
  ) {
    const transcript = transcripts.value[sessionId];

    if (!transcript) {
      return false;
    }

    application.operationError = null;

    try {
      transcripts.value[sessionId] = await native.updateTranscriptSegment(
        sessionId,
        segmentId,
        text,
        transcript.revision,
      );
      return true;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function renameSpeaker(
    sessionId: string,
    speakerId: string,
    displayName: string,
  ) {
    const transcript = transcripts.value[sessionId];

    if (!transcript) {
      return false;
    }

    application.operationError = null;

    try {
      transcripts.value[sessionId] = await native.renameSpeaker(
        sessionId,
        speakerId,
        displayName,
        transcript.revision,
      );
      return true;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function mergeSpeakers(
    sessionId: string,
    sourceSpeakerId: string,
    targetSpeakerId: string,
  ) {
    const transcript = transcripts.value[sessionId];

    if (!transcript) {
      return false;
    }

    application.operationError = null;

    try {
      transcripts.value[sessionId] = await native.mergeSpeakers(
        sessionId,
        sourceSpeakerId,
        targetSpeakerId,
        transcript.revision,
      );
      return true;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function trashSession(sessionId: string) {
    application.operationError = null;

    try {
      await native.trashSession(sessionId);
      await application.refreshLibrary();
      return true;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function moveSession(sessionId: string, projectId: string | null) {
    application.operationError = null;

    try {
      const session = await native.moveSession(sessionId, projectId);
      application.replaceRecentSession(session);
      return session;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    }
  }

  async function recoverRecording(sessionId: string) {
    application.operationError = null;

    try {
      const session = await native.recoverRecording(sessionId);
      application.replaceRecentSession(session);
      return session;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    }
  }

  async function exportSession(
    sessionId: string,
    format: ExportFormat,
  ): Promise<ExportResult | null> {
    application.operationError = null;

    try {
      return await native.exportSession(sessionId, format);
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return null;
    }
  }

  return {
    transcripts,
    transcriptionJobForSession,
    transcribeWithOpenAi,
    transcribeLocally,
    loadWorkspace,
    startNotesEditing,
    saveNotes,
    updateTranscriptSegment,
    renameSpeaker,
    mergeSpeakers,
    recoverRecording,
    moveSession,
    trashSession,
    exportSession,
  };
});
