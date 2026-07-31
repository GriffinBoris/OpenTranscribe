import { ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";

export const useGlobalShortcutStore = defineStore("globalShortcut", () => {
  const isConfiguring = ref(false);
  const isRegistered = ref(false);
  const errorMessage = ref<string | null>(null);

  async function configure(
    shortcut: string | null,
    onTrigger: () => void,
  ): Promise<void> {
    isConfiguring.value = true;
    errorMessage.value = null;

    try {
      await native.configureGlobalShortcut(shortcut, onTrigger);
      isRegistered.value = shortcut !== null;
    } catch (reason) {
      isRegistered.value = false;
      errorMessage.value =
        reason instanceof Error ? reason.message : String(reason);
    } finally {
      isConfiguring.value = false;
    }
  }

  return {
    isConfiguring,
    isRegistered,
    errorMessage,
    configure,
  };
});
