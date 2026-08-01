import { ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";

export const useGlobalShortcutStore = defineStore("globalShortcut", () => {
  const isConfiguring = ref(false);
  const isRegistered = ref(false);
  const errorMessage = ref<string | null>(null);
  let onTrigger: (() => void) | null = null;

  async function configure(
    shortcut: string | null,
    nextOnTrigger: () => void,
  ): Promise<boolean> {
    isConfiguring.value = true;
    errorMessage.value = null;
    onTrigger = nextOnTrigger;

    try {
      await native.configureGlobalShortcut(shortcut, nextOnTrigger);
      isRegistered.value = shortcut !== null;
      return true;
    } catch (reason) {
      errorMessage.value =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    } finally {
      isConfiguring.value = false;
    }
  }

  async function configureCandidate(shortcut: string): Promise<boolean> {
    if (!onTrigger) {
      return false;
    }

    return configure(shortcut, onTrigger);
  }

  return {
    isConfiguring,
    isRegistered,
    errorMessage,
    configure,
    configureCandidate,
  };
});
