import { ref, type Ref } from "vue";

import { native } from "@/core/native";
import type { SessionAudioSource } from "@/types/domain";
import { useSessionStore } from "@/views/session/sessionStore";

interface PlaybackController {
  seek(seconds: number): void;
}

export function useSessionMedia(sessionId: Ref<string>) {
  const sessionStore = useSessionStore();
  const audioSources = ref<SessionAudioSource[]>([]);
  const waveform = ref<number[]>([]);
  const playback = ref<PlaybackController | null>(null);
  const playbackPosition = ref(0);
  const sessionError = ref<string | null>(null);
  const isRecovering = ref(false);

  async function loadMedia() {
    sessionError.value = null;

    try {
      [audioSources.value, waveform.value] = await Promise.all([
        native.sessionAudioSources(sessionId.value),
        native.sessionWaveform(sessionId.value).catch(() => []),
      ]);
    } catch (reason) {
      sessionError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function revealSession() {
    sessionError.value = null;

    try {
      await native.revealSession(sessionId.value);
    } catch (reason) {
      sessionError.value =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function recoverRecording() {
    isRecovering.value = true;
    sessionError.value = null;

    try {
      if (await sessionStore.recoverRecording(sessionId.value)) {
        await loadMedia();
      }
    } catch (reason) {
      sessionError.value =
        reason instanceof Error ? reason.message : String(reason);
    } finally {
      isRecovering.value = false;
    }
  }

  return {
    audioSources,
    waveform,
    playback,
    playbackPosition,
    sessionError,
    isRecovering,
    loadMedia,
    revealSession,
    recoverRecording,
  };
}
