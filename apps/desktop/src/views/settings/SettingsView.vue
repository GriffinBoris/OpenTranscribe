<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";
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
let scrollEndTimer: number | null = null;
let pendingNavigation: { section: string; top: number } | null = null;
const settingsNavButtonClass =
  "w-full justify-start gap-2.5 rounded-app-sm px-2.5 py-[9px] text-ink hover:bg-canvas-subtle [&.active]:bg-canvas-subtle [&.active]:text-ink max-[900px]:w-auto";

function updateActiveSection(section: string) {
  activeSection.value = section;
  window.history.replaceState({ ...window.history.state }, "", `#${section}`);
}

function selectSection(section: string) {
  updateActiveSection(section);

  const scrollPane = settingsContent.value!;
  const target = document.getElementById(section)!;
  const offset = Number.parseFloat(
    getComputedStyle(scrollPane).getPropertyValue("--space-5"),
  );
  const top = Math.max(
    0,
    Math.min(
      scrollPane.scrollHeight - scrollPane.clientHeight,
      scrollPane.scrollTop +
        target.getBoundingClientRect().top -
        scrollPane.getBoundingClientRect().top -
        offset,
    ),
  );

  pendingNavigation = { section, top };

  scrollPane.scrollTo({
    behavior: "smooth",
    top,
  });
}

function updateContentScroll(event: Event) {
  const scrollPane = event.currentTarget as HTMLElement;
  isContentScrolled.value = scrollPane.scrollTop > 0;

  if (scrollEndTimer !== null) {
    window.clearTimeout(scrollEndTimer);
  }

  scrollEndTimer = window.setTimeout(() => {
    const navigation = pendingNavigation;
    pendingNavigation = null;

    if (navigation && Math.abs(scrollPane.scrollTop - navigation.top) <= 2) {
      updateActiveSection(navigation.section);
      return;
    }

    const activeTarget = Array.from(
      scrollPane.children,
    ).reduce<HTMLElement | null>((closestTarget, child) => {
      if (!(child instanceof HTMLElement)) {
        return closestTarget;
      }

      if (!closestTarget) {
        return child;
      }

      const contentTop = scrollPane.getBoundingClientRect().top;
      const currentDistance = Math.abs(
        child.getBoundingClientRect().top - contentTop,
      );
      const closestDistance = Math.abs(
        closestTarget.getBoundingClientRect().top - contentTop,
      );

      return currentDistance < closestDistance ? child : closestTarget;
    }, null);

    if (activeTarget?.id) {
      updateActiveSection(activeTarget.id);
    }
  }, 100);
}

onMounted(() => {
  activeSection.value = window.location.hash.slice(1) || "dictation";
  void nextTick(() => selectSection(activeSection.value));
});

onBeforeUnmount(() => {
  if (scrollEndTimer !== null) {
    window.clearTimeout(scrollEndTimer);
  }
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
