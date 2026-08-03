import { reactive } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";

export const useGlobalShortcutStore = defineStore("globalShortcut", () => {
  const shortcuts = reactive({
    recording: {
      isConfiguring: false,
      isRegistered: false,
      errorMessage: null as string | null,
      accelerator: null as string | null,
      onTrigger: null as (() => void) | null,
    },
    dictation: {
      isConfiguring: false,
      isRegistered: false,
      errorMessage: null as string | null,
      accelerator: null as string | null,
      onTrigger: null as (() => void) | null,
    },
  });
  let captureDialogCount = 0;

  async function configure(
    shortcutId: "recording" | "dictation",
    shortcut: string | null,
    nextOnTrigger: () => void,
  ): Promise<boolean> {
    const shortcutState = shortcuts[shortcutId];
    shortcutState.isConfiguring = true;
    shortcutState.errorMessage = null;
    shortcutState.onTrigger = nextOnTrigger;

    if (captureDialogCount > 0) {
      shortcutState.accelerator = shortcut;
      shortcutState.isRegistered = shortcut !== null;
      shortcutState.isConfiguring = false;
      return true;
    }

    try {
      await native.configureGlobalShortcut(shortcutId, shortcut, nextOnTrigger);
      shortcutState.accelerator = shortcut;
      shortcutState.isRegistered = shortcut !== null;
      return true;
    } catch (reason) {
      shortcutState.errorMessage =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    } finally {
      shortcutState.isConfiguring = false;
    }
  }

  async function configureCandidate(
    shortcutId: "recording" | "dictation",
    shortcut: string,
  ): Promise<boolean> {
    const nextOnTrigger = shortcuts[shortcutId].onTrigger;

    if (!nextOnTrigger) {
      return false;
    }

    return configure(shortcutId, shortcut, nextOnTrigger);
  }

  async function beginShortcutCapture(): Promise<boolean> {
    captureDialogCount += 1;

    if (captureDialogCount > 1) {
      return true;
    }

    try {
      await Promise.all(
        (Object.keys(shortcuts) as Array<keyof typeof shortcuts>).map(
          async (shortcutId) => {
            const shortcut = shortcuts[shortcutId];

            if (shortcut.accelerator && shortcut.onTrigger) {
              await native.configureGlobalShortcut(
                shortcutId,
                null,
                shortcut.onTrigger,
              );
            }
          },
        ),
      );
      return true;
    } catch (reason) {
      captureDialogCount = 1;
      await endShortcutCapture();
      const message = reason instanceof Error ? reason.message : String(reason);

      for (const shortcut of Object.values(shortcuts)) {
        shortcut.errorMessage = message;
      }

      return false;
    }
  }

  async function endShortcutCapture() {
    captureDialogCount = Math.max(0, captureDialogCount - 1);

    if (captureDialogCount > 0) {
      return;
    }

    await Promise.all(
      (Object.keys(shortcuts) as Array<keyof typeof shortcuts>).map(
        async (shortcutId) => {
          const shortcut = shortcuts[shortcutId];

          if (!shortcut.accelerator || !shortcut.onTrigger) {
            return;
          }

          shortcut.isConfiguring = true;
          shortcut.errorMessage = null;

          try {
            await native.configureGlobalShortcut(
              shortcutId,
              shortcut.accelerator,
              shortcut.onTrigger,
            );
            shortcut.isRegistered = true;
          } catch (reason) {
            shortcut.isRegistered = false;
            shortcut.errorMessage =
              reason instanceof Error ? reason.message : String(reason);
          } finally {
            shortcut.isConfiguring = false;
          }
        },
      ),
    );
  }

  return {
    shortcuts,
    configure,
    configureCandidate,
    beginShortcutCapture,
    endShortcutCapture,
  };
});
