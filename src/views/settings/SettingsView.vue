<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  FolderOpen,
  HardDrive,
  Keyboard,
  KeyRound,
  Monitor,
  Volume2,
} from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type { ThemePreference } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useOpenAiStore } from "@/views/application/openAiStore";
import LocalModelsSettings from "@/views/settings/components/LocalModelsSettings.vue";
import RecordingSettings from "@/views/settings/components/RecordingSettings.vue";
import ShortcutsSettings from "@/views/settings/components/ShortcutsSettings.vue";
import StorageSettings from "@/views/settings/components/StorageSettings.vue";

const application = useApplicationStore();
const openAi = useOpenAiStore();
const { t } = useI18n();
const theme = ref<ThemePreference>(
  application.settings?.appearance.theme ?? "system",
);
const reducedMotion = ref(
  application.settings?.appearance.reduced_motion ?? false,
);
const apiKey = ref("");
const activeSection = ref("recording");
const settingsContent = ref<HTMLElement | null>(null);
const isContentScrolled = ref(false);
const settingsNavButtonClass =
  "w-full justify-start gap-2.5 rounded-app-sm px-2.5 py-[9px] text-ink-muted hover:bg-canvas-subtle hover:text-ink [&.active]:bg-canvas-subtle [&.active]:text-ink max-[900px]:w-auto";

const themeOptions = computed(() => [
  { label: t("settings.appearance.system"), value: "system" },
  { label: t("settings.appearance.light"), value: "light" },
  { label: t("settings.appearance.dark"), value: "dark" },
]);

async function updateTheme(value: string) {
  theme.value = value as ThemePreference;
  application.applyTheme(theme.value);
  await application.saveSettings({
    appearance: {
      ...application.settings!.appearance,
      theme: theme.value,
    },
  });
}

async function updateReducedMotion(value: boolean) {
  reducedMotion.value = value;
  application.applyReducedMotion(value);
  await application.saveSettings({
    appearance: {
      ...application.settings!.appearance,
      reduced_motion: value,
    },
  });
}

async function saveApiKey() {
  if (await openAi.saveApiKey(apiKey.value)) {
    apiKey.value = "";
  }
}

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

onMounted(async () => {
  activeSection.value = window.location.hash.slice(1) || "recording";
  void openAi.loadCredential();
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
          <RecordingSettings />

          <StorageSettings />

          <LocalModelsSettings />

          <AppSurface id="openai">
            <div
              class="section-heading flex items-start justify-between gap-4 border-b border-[var(--divider)] pb-3.5"
            >
              <div>
                <h2 class="mb-[5px] text-2xl">
                  {{ t("settings.navigation.openAi") }}
                </h2>
                <p
                  class="text-ink-muted mt-[3px] leading-[var(--line-height-body)]"
                >
                  {{ t("settings.openAi.description") }}
                </p>
              </div>
              <StatusPill v-if="openAi.credential?.configured" tone="success">
                {{ t("settings.openAi.configured") }}
              </StatusPill>
            </div>
            <div
              v-if="openAi.credential?.configured"
              class="grid grid-cols-[minmax(220px,1fr)_auto_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
            >
              <KeyRound :size="22" />
              <span class="grid gap-1"
                ><strong>{{ t("settings.openAi.defaultProfile") }}</strong
                ><small>{{ openAi.credential.masked_key }}</small></span
              >
              <AppButton size="small" @click="openAi.testConnection">
                {{ t("settings.openAi.testConnection") }}
              </AppButton>
              <AppButton
                size="small"
                variant="danger"
                @click="openAi.removeApiKey"
              >
                {{ t("settings.openAi.remove") }}
              </AppButton>
            </div>
            <form
              v-else
              class="openai-key-form grid grid-cols-[minmax(120px,0.5fr)_minmax(240px,1.3fr)_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
              @submit.prevent="saveApiKey"
            >
              <span>{{ t("settings.openAi.apiKey") }}</span>
              <AppInputText
                v-model="apiKey"
                type="password"
                autocomplete="off"
                placeholder="sk-…"
                required
              />
              <AppButton type="submit">
                {{ t("settings.openAi.save") }}
              </AppButton>
            </form>
            <p v-if="openAi.connectionMessage" role="status">
              {{ openAi.connectionMessage }}
            </p>
          </AppSurface>

          <ShortcutsSettings />

          <AppSurface id="appearance">
            <div class="border-b border-[var(--divider)] pb-3.5">
              <h2 class="mb-[5px] text-2xl">
                {{ t("settings.navigation.appearance") }}
              </h2>
            </div>
            <label
              class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
            >
              <span>{{ t("settings.appearance.theme") }}</span>
              <AppSelect
                :model-value="theme"
                :options="themeOptions"
                :accessible-label="t('settings.appearance.theme')"
                @update:model-value="updateTheme"
              />
            </label>
            <div
              class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
            >
              <span class="grid gap-1">
                <strong>{{ t("settings.appearance.reducedMotion") }}</strong>
                <small>{{
                  t("settings.appearance.reducedMotionDescription")
                }}</small>
              </span>
              <AppToggleSwitch
                :model-value="reducedMotion"
                :accessible-label="t('settings.appearance.reducedMotion')"
                @update:model-value="updateReducedMotion"
              />
            </div>
          </AppSurface>

          <div
            class="h-[calc(100dvh-var(--layout-titlebar-height))]"
            aria-hidden="true"
          />
        </div>
      </div>
    </div>
  </div>
</template>
