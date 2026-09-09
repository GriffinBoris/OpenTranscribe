<script setup lang="ts">
import { computed, nextTick, onMounted } from "vue";
import { useRoute } from "vue-router";
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
const route = useRoute();
const activeSection = computed(() => route.hash.slice(1) || "dictation");
const sections = [
  { id: "dictation", label: "dictation", icon: MessageSquareText },
  { id: "recording", label: "recording", icon: Volume2 },
  { id: "storage", label: "storage", icon: FolderOpen },
  { id: "models", label: "localModels", icon: HardDrive },
  { id: "openai", label: "openAi", icon: KeyRound },
  { id: "shortcuts", label: "shortcuts", icon: Keyboard },
  { id: "appearance", label: "appearance", icon: Monitor },
  { id: "application", label: "application", icon: Settings2 },
];

function selectSection(section: string) {
  window.location.hash = section;
}

onMounted(async () => {
  await nextTick();
  const hash = window.location.hash.slice(1);
  const section = sections.some((item) => item.id === hash)
    ? hash
    : "dictation";
  if (section !== hash) selectSection(section);
  document.getElementById(section)!.scrollIntoView();
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
      class="settings-layout grid min-h-0 grid-cols-[190px_minmax(0,1fr)] items-stretch gap-6 max-[1000px]:grid-cols-1 max-[1000px]:grid-rows-[auto_minmax(0,1fr)] max-[1000px]:gap-4"
    >
      <nav
        :aria-label="t('settings.sections')"
        class="settings-nav grid min-h-0 content-start gap-1 overflow-auto pr-1 max-[1000px]:auto-cols-max max-[1000px]:grid-flow-col max-[1000px]:overflow-x-auto max-[1000px]:overflow-y-hidden max-[1000px]:pr-0"
      >
        <a
          v-for="section in sections"
          :key="section.id"
          class="rounded-app-sm text-ink hover:bg-canvas-subtle [&.active]:bg-canvas-subtle inline-flex w-full items-center justify-start gap-2.5 px-2.5 py-[9px] font-semibold max-[1000px]:w-auto"
          :class="{ active: activeSection === section.id }"
          :aria-current="activeSection === section.id ? 'location' : undefined"
          :href="`#${section.id}`"
        >
          <component :is="section.icon" :size="16" aria-hidden="true" />
          {{ t(`settings.navigation.${section.label}`) }}
        </a>
      </nav>

      <div
        class="settings-content grid h-full min-h-0 auto-rows-max content-start gap-4 overflow-auto pr-1"
      >
        <DictationSettings @open-models="selectSection('models')" />

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
