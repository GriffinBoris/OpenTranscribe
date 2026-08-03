<script setup lang="ts">
import { nextTick, onMounted, ref } from "vue";
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

import AppButton from "@/components/ui/AppButton.vue";
import ApplicationSettings from "@/views/settings/components/ApplicationSettings.vue";
import AppearanceSettings from "@/views/settings/components/AppearanceSettings.vue";
import DictationSettings from "@/views/settings/components/DictationSettings.vue";
import LocalModelsSettings from "@/views/settings/components/LocalModelsSettings.vue";
import OpenAiSettings from "@/views/settings/components/OpenAiSettings.vue";
import RecordingSettings from "@/views/settings/components/RecordingSettings.vue";
import ShortcutsSettings from "@/views/settings/components/ShortcutsSettings.vue";
import StorageSettings from "@/views/settings/components/StorageSettings.vue";

const { t } = useI18n();
const activeSection = ref("dictation");
const settingsContent = ref<HTMLElement | null>(null);
const isContentScrolled = ref(false);
const settingsNavButtonClass =
  "w-full justify-start gap-2.5 rounded-app-sm px-2.5 py-[9px] text-ink hover:bg-canvas-subtle [&.active]:bg-canvas-subtle [&.active]:text-ink max-[900px]:w-auto";

function selectSection(section: string) {
  activeSection.value = section;
  window.history.replaceState({ ...window.history.state }, "", `#${section}`);

  const scrollPane = settingsContent.value!;

  if (section === "recording") {
    isContentScrolled.value = false;
    scrollPane.scrollTo({ top: 0, behavior: "smooth" });
    return;
  }

  const target = document.getElementById(section)!;
  const offset = Number.parseFloat(
    getComputedStyle(scrollPane).getPropertyValue("--space-5"),
  );

  scrollPane.scrollTo({
    behavior: "smooth",
    top:
      scrollPane.scrollTop +
      target.getBoundingClientRect().top -
      scrollPane.getBoundingClientRect().top -
      offset,
  });
}

function updateContentScroll(event: Event) {
  isContentScrolled.value = (event.currentTarget as HTMLElement).scrollTop > 0;
}

onMounted(() => {
  activeSection.value = window.location.hash.slice(1) || "dictation";
  void nextTick(() => selectSection(activeSection.value));
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
        <AppButton
          variant="ghost"
          :class="[
            settingsNavButtonClass,
            { active: activeSection === 'dictation' },
          ]"
          @click="selectSection('dictation')"
          ><MessageSquareText :size="16" />{{
            t("settings.navigation.dictation")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="[
            settingsNavButtonClass,
            { active: activeSection === 'recording' },
          ]"
          @click="selectSection('recording')"
          ><Volume2 :size="16" />{{
            t("settings.navigation.recording")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="[
            settingsNavButtonClass,
            { active: activeSection === 'storage' },
          ]"
          @click="selectSection('storage')"
          ><FolderOpen :size="16" />{{
            t("settings.navigation.storage")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="[
            settingsNavButtonClass,
            { active: activeSection === 'models' },
          ]"
          @click="selectSection('models')"
          ><HardDrive :size="16" />{{
            t("settings.navigation.localModels")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="[
            settingsNavButtonClass,
            { active: activeSection === 'openai' },
          ]"
          @click="selectSection('openai')"
          ><KeyRound :size="16" />{{
            t("settings.navigation.openAi")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="[
            settingsNavButtonClass,
            { active: activeSection === 'shortcuts' },
          ]"
          @click="selectSection('shortcuts')"
          ><Keyboard :size="16" />{{
            t("settings.navigation.shortcuts")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="[
            settingsNavButtonClass,
            { active: activeSection === 'appearance' },
          ]"
          @click="selectSection('appearance')"
          ><Monitor :size="16" />{{
            t("settings.navigation.appearance")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="[
            settingsNavButtonClass,
            { active: activeSection === 'application' },
          ]"
          @click="selectSection('application')"
          ><Settings2 :size="16" />{{
            t("settings.navigation.application")
          }}</AppButton
        >
      </nav>

      <div
        class="settings-content-frame relative min-h-0"
        :class="{ 'settings-content-frame--scrolled': isContentScrolled }"
      >
        <div
          ref="settingsContent"
          class="settings-content grid h-full min-h-0 auto-rows-max content-start gap-4 overflow-auto pr-1"
          @scroll="updateContentScroll"
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
  </div>
</template>
