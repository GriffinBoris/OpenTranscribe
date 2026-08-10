<script setup lang="ts">
import { nextTick, onMounted } from "vue";
import {
  FolderOpen,
  HardDrive,
  Keyboard,
  KeyRound,
  MessageSquareText,
  Monitor,
  Settings2,
  Volume2,
} from "@lucide/vue";
import { useI18n } from "vue-i18n";

import ApplicationSettings from "@/views/settings/components/ApplicationSettings.vue";
import AppearanceSettings from "@/views/settings/components/AppearanceSettings.vue";
import DictationSettings from "@/views/settings/components/DictationSettings.vue";
import LocalModelsSettings from "@/views/settings/components/LocalModelsSettings.vue";
import OpenAiSettings from "@/views/settings/components/OpenAiSettings.vue";
import RecordingSettings from "@/views/settings/components/RecordingSettings.vue";
import ShortcutsSettings from "@/views/settings/components/ShortcutsSettings.vue";
import StorageSettings from "@/views/settings/components/StorageSettings.vue";

const { t } = useI18n();
const settingsNavLinkClass =
  "inline-flex w-full items-center justify-start gap-2.5 rounded-app-sm px-2.5 py-[9px] font-semibold text-ink hover:bg-canvas-subtle max-[900px]:w-auto";

onMounted(async () => {
  await nextTick();
  const section = document.getElementById(window.location.hash.slice(1));
  section?.scrollIntoView();
});
</script>

<template>
  <div
    class="page settings-page grid h-full min-h-0 w-full grid-rows-[auto_minmax(0,1fr)] overflow-hidden px-[var(--layout-page-gutter)] pt-[var(--space-13)] pb-[var(--space-14)]"
  >
    <header
      class="page-header page-header--compact mb-[var(--space-5-5)] flex items-start justify-between gap-6"
    >
      <h1
        class="text-display m-0 max-w-[730px] leading-[var(--line-height-tight)] font-bold tracking-[-0.035em]"
      >
        {{ t("settings.title") }}
      </h1>
    </header>

    <div
      class="settings-layout grid min-h-0 grid-cols-[190px_minmax(0,1fr)] items-stretch gap-6 max-[900px]:grid-cols-1 max-[900px]:grid-rows-[auto_minmax(0,1fr)] max-[900px]:gap-4"
    >
      <nav
        class="settings-nav grid min-h-0 content-start gap-1 overflow-auto pr-1 max-[900px]:auto-cols-max max-[900px]:grid-flow-col max-[900px]:overflow-x-auto max-[900px]:overflow-y-hidden max-[900px]:pr-0"
      >
        <a :class="settingsNavLinkClass" href="#dictation"
          ><MessageSquareText :size="16" />{{
            t("settings.navigation.dictation")
          }}</a
        >
        <a :class="settingsNavLinkClass" href="#recording"
          ><Volume2 :size="16" />{{ t("settings.navigation.recording") }}</a
        >
        <a :class="settingsNavLinkClass" href="#storage"
          ><FolderOpen :size="16" />{{ t("settings.navigation.storage") }}</a
        >
        <a :class="settingsNavLinkClass" href="#models"
          ><HardDrive :size="16" />{{ t("settings.navigation.localModels") }}</a
        >
        <a :class="settingsNavLinkClass" href="#openai"
          ><KeyRound :size="16" />{{ t("settings.navigation.openAi") }}</a
        >
        <a :class="settingsNavLinkClass" href="#shortcuts"
          ><Keyboard :size="16" />{{ t("settings.navigation.shortcuts") }}</a
        >
        <a :class="settingsNavLinkClass" href="#appearance"
          ><Monitor :size="16" />{{ t("settings.navigation.appearance") }}</a
        >
        <a :class="settingsNavLinkClass" href="#application"
          ><Settings2 :size="16" />{{ t("settings.navigation.application") }}</a
        >
      </nav>

      <div
        class="settings-content grid h-full min-h-0 auto-rows-max content-start gap-4 overflow-auto pr-1"
      >
        <DictationSettings />

        <RecordingSettings />

        <StorageSettings />

        <LocalModelsSettings />

        <OpenAiSettings />

        <ShortcutsSettings />

        <AppearanceSettings />

        <ApplicationSettings />
      </div>
    </div>
  </div>
</template>
