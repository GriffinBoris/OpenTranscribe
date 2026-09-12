<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useGlobalShortcutStore } from "@/views/application/globalShortcutStore";
import {
  formatGlobalShortcut,
  shortcutFromKeyboardEvent,
} from "@/views/application/globalShortcutPresets";

const application = useApplicationStore();
const globalShortcut = useGlobalShortcutStore();
const { t } = useI18n();
const bindingDialogOpen = ref(false);
const capturedShortcut = ref<string | null>(null);

const globalShortcutEnabled = computed(
  () => application.settings?.global_shortcut_enabled ?? false,
);
const selectedGlobalShortcut = computed(
  () => application.settings?.global_shortcut ?? "CommandOrControl+Shift+R",
);
const globalShortcutStatus = computed(() => {
  if (globalShortcut.shortcuts.recording.isConfiguring) {
    return {
      label: t("settings.shortcuts.updating"),
      tone: "neutral" as const,
    };
  }

  if (globalShortcut.shortcuts.recording.errorMessage) {
    return {
      label: t("settings.shortcuts.unavailable"),
      tone: "warning" as const,
    };
  }

  if (globalShortcut.shortcuts.recording.isRegistered) {
    return {
      label: t("settings.shortcuts.active"),
      tone: "success" as const,
    };
  }

  return {
    label: t("settings.shortcuts.off"),
    tone: "neutral" as const,
  };
});
const shortcuts = computed(() => [
  { label: t("settings.shortcuts.searchLibrary"), keys: "⌘/Ctrl K" },
  { label: t("settings.shortcuts.importMedia"), keys: "⌘/Ctrl O" },
  { label: t("settings.shortcuts.searchSession"), keys: "⌘/Ctrl F" },
  { label: t("settings.shortcuts.addTimestamp"), keys: "⌘/Ctrl ⇧ M" },
]);

async function updateGlobalShortcutEnabled(value: boolean) {
  await application.saveSettings({ global_shortcut_enabled: value });
}

async function openShortcutCapture() {
  capturedShortcut.value = null;
  if (!(await globalShortcut.beginShortcutCapture())) {
    return;
  }
  bindingDialogOpen.value = true;
}

function captureShortcut(event: KeyboardEvent) {
  const shortcut = shortcutFromKeyboardEvent(event);

  if (!shortcut) {
    return;
  }

  event.preventDefault();
  capturedShortcut.value = shortcut;
}

async function saveCapturedShortcut() {
  if (!capturedShortcut.value) {
    return;
  }

  const previousShortcut = selectedGlobalShortcut.value;

  if (!globalShortcutEnabled.value) {
    if (
      await application.saveSettings({
        global_shortcut: capturedShortcut.value,
      })
    ) {
      bindingDialogOpen.value = false;
    }

    return;
  }

  if (
    !(await globalShortcut.configureCandidate(
      "recording",
      capturedShortcut.value,
    ))
  ) {
    return;
  }

  if (
    !(await application.saveSettings({
      global_shortcut: capturedShortcut.value,
    }))
  ) {
    await globalShortcut.configureCandidate("recording", previousShortcut);
    return;
  }

  bindingDialogOpen.value = false;
}

watch(bindingDialogOpen, (isOpen) => {
  if (isOpen) {
    window.addEventListener("keydown", captureShortcut);
    return;
  }

  window.removeEventListener("keydown", captureShortcut);
  void globalShortcut.endShortcutCapture();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", captureShortcut);

  if (bindingDialogOpen.value) {
    void globalShortcut.endShortcutCapture();
  }
});
</script>

<template>
  <AppSurface id="settings-shortcuts">
    <div
      class="section-heading flex items-start justify-between gap-4 border-b border-[var(--divider)] pb-3.5"
    >
      <div>
        <h2 class="mb-[5px] text-2xl">
          {{ t("settings.navigation.shortcuts") }}
        </h2>
        <p class="text-ink mt-[3px] leading-[var(--line-height-body)]">
          {{ t("settings.shortcuts.description") }}
        </p>
      </div>
      <StatusPill :tone="globalShortcutStatus.tone" aria-live="polite">
        {{ globalShortcutStatus.label }}
      </StatusPill>
    </div>
    <div
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid gap-1">
        <strong>{{ t("settings.shortcuts.globalRecording") }}</strong>
        <small>{{ t("settings.shortcuts.globalDescription") }}</small>
      </span>
      <AppToggleSwitch
        :model-value="globalShortcutEnabled"
        :accessible-label="t('settings.shortcuts.globalRecording')"
        :disabled="globalShortcut.shortcuts.recording.isConfiguring"
        @update:model-value="updateGlobalShortcutEnabled"
      />
    </div>
    <div
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid gap-1">
        <strong>{{ t("settings.shortcuts.globalShortcut") }}</strong>
        <small
          v-if="globalShortcut.shortcuts.recording.errorMessage"
          class="text-accent"
          role="alert"
        >
          {{
            t("settings.shortcuts.registrationError", {
              message: globalShortcut.shortcuts.recording.errorMessage,
            })
          }}
        </small>
      </span>
      <AppButton
        variant="secondary"
        :aria-label="t('settings.shortcuts.globalShortcut')"
        :disabled="globalShortcut.shortcuts.recording.isConfiguring"
        @click="openShortcutCapture"
      >
        {{ formatGlobalShortcut(selectedGlobalShortcut) }}
      </AppButton>
    </div>
    <div
      v-for="shortcut in shortcuts"
      :key="shortcut.label"
      class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-t border-[var(--divider)] py-[11px]"
    >
      <span>{{ shortcut.label }}</span>
      <kbd
        class="rounded-app-xs border-line bg-canvas-subtle text-ink-muted px-[7px] py-1 text-sm"
        >{{ shortcut.keys }}</kbd
      >
    </div>
    <AppDialog
      :open="bindingDialogOpen"
      :title="t('settings.shortcuts.captureTitle')"
      @update:open="bindingDialogOpen = $event"
    >
      <div class="grid gap-3">
        <p class="m-0">
          {{ t("settings.shortcuts.captureDescription") }}
        </p>
        <p
          class="rounded-app-sm border-line bg-canvas-subtle m-0 px-3 py-2 text-center text-lg font-semibold"
          aria-live="polite"
        >
          {{
            capturedShortcut
              ? formatGlobalShortcut(capturedShortcut)
              : t("settings.shortcuts.captureWaiting")
          }}
        </p>
      </div>
      <template #footer>
        <AppButton variant="ghost" @click="bindingDialogOpen = false">
          {{ t("settings.shortcuts.cancel") }}
        </AppButton>
        <AppButton
          :disabled="
            !capturedShortcut ||
            globalShortcut.shortcuts.recording.isConfiguring
          "
          :loading="globalShortcut.shortcuts.recording.isConfiguring"
          @click="saveCapturedShortcut"
        >
          {{ t("settings.shortcuts.save") }}
        </AppButton>
      </template>
    </AppDialog>
  </AppSurface>
</template>
