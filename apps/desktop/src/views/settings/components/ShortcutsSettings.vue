<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AppSelect from "@/components/ui/AppSelect.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type { GlobalShortcutPreset } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useGlobalShortcutStore } from "@/views/application/globalShortcutStore";

const application = useApplicationStore();
const globalShortcut = useGlobalShortcutStore();
const { t } = useI18n();

const globalShortcutEnabled = computed(
  () => application.settings?.global_shortcut_enabled ?? false,
);
const selectedGlobalShortcut = computed(
  () => application.settings?.global_shortcut ?? "command_or_control_shift_r",
);
const globalShortcutOptions = computed(() => [
  {
    label: t("settings.shortcuts.commandOrControlShiftR"),
    value: "command_or_control_shift_r",
  },
  {
    label: t("settings.shortcuts.commandOrControlShiftSpace"),
    value: "command_or_control_shift_space",
  },
  {
    label: t("settings.shortcuts.altShiftR"),
    value: "alt_shift_r",
  },
]);
const globalShortcutStatus = computed(() => {
  if (globalShortcut.isConfiguring) {
    return {
      label: t("settings.shortcuts.updating"),
      tone: "neutral" as const,
    };
  }

  if (globalShortcut.errorMessage) {
    return {
      label: t("settings.shortcuts.unavailable"),
      tone: "warning" as const,
    };
  }

  if (globalShortcut.isRegistered) {
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

async function updateGlobalShortcut(value: string) {
  await application.saveSettings({
    global_shortcut: value as GlobalShortcutPreset,
  });
}
</script>

<template>
  <AppSurface id="shortcuts">
    <div
      class="section-heading flex items-start justify-between gap-4 border-b border-[var(--divider)] pb-3.5"
    >
      <div>
        <h2 class="mb-[5px] text-2xl">
          {{ t("settings.navigation.shortcuts") }}
        </h2>
        <p class="text-ink-muted mt-[3px] leading-[var(--line-height-body)]">
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
        :disabled="globalShortcut.isConfiguring"
        @update:model-value="updateGlobalShortcutEnabled"
      />
    </div>
    <label
      v-if="globalShortcutEnabled"
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid gap-1">
        <strong>{{ t("settings.shortcuts.globalShortcut") }}</strong>
        <small
          v-if="globalShortcut.errorMessage"
          class="text-accent"
          role="alert"
        >
          {{
            t("settings.shortcuts.registrationError", {
              message: globalShortcut.errorMessage,
            })
          }}
        </small>
      </span>
      <AppSelect
        :model-value="selectedGlobalShortcut"
        :options="globalShortcutOptions"
        :accessible-label="t('settings.shortcuts.globalShortcut')"
        :disabled="globalShortcut.isConfiguring"
        @update:model-value="updateGlobalShortcut"
      />
    </label>
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
  </AppSurface>
</template>
