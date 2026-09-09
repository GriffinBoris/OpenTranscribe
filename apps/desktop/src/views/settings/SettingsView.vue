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
    behavior:
      document.documentElement.dataset.reducedMotion === "true" ||
      window.matchMedia("(prefers-reduced-motion: reduce)").matches
        ? "auto"
        : "smooth",
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
  const section = window.location.hash.slice(1);
  activeSection.value = sections.some((item) => item.id === section)
    ? section
    : "dictation";
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
      class="settings-layout grid min-h-0 grid-cols-[190px_minmax(0,1fr)] items-stretch gap-6 max-[1000px]:grid-cols-1 max-[1000px]:grid-rows-[auto_minmax(0,1fr)] max-[1000px]:gap-4"
    >
      <nav
        :aria-label="t('settings.sections')"
        class="settings-nav grid min-h-0 content-start gap-1 overflow-auto pr-1 max-[1000px]:auto-cols-max max-[1000px]:grid-flow-col max-[1000px]:overflow-x-auto max-[1000px]:overflow-y-hidden max-[1000px]:pr-0"
      >
        <AppButton
          v-for="section in sections"
          :key="section.id"
          variant="ghost"
          class="rounded-app-sm text-ink hover:bg-canvas-subtle [&.active]:bg-canvas-subtle w-full justify-start gap-2.5 px-2.5 py-[9px] max-[1000px]:w-auto"
          :class="{ active: activeSection === section.id }"
          :aria-current="activeSection === section.id ? 'location' : undefined"
          @click="selectSection(section.id)"
        >
          <component :is="section.icon" :size="16" aria-hidden="true" />
          {{ t(`settings.navigation.${section.label}`) }}
        </AppButton>
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
  </div>
</template>
