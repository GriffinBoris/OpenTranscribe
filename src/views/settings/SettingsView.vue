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
  document.getElementById(section)?.scrollIntoView({ behavior: "smooth" });
}

onMounted(async () => {
  activeSection.value = window.location.hash.slice(1) || "recording";
  void openAi.loadCredential();
});
</script>

<template>
  <div class="page settings-page">
    <header class="page-header page-header--compact">
      <h1>{{ t("settings.title") }}</h1>
    </header>

    <div class="settings-layout">
      <nav class="settings-nav">
        <AppButton
          variant="ghost"
          :class="{ active: activeSection === 'recording' }"
          @click="selectSection('recording')"
          ><Volume2 :size="16" />{{
            t("settings.navigation.recording")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="{ active: activeSection === 'storage' }"
          @click="selectSection('storage')"
          ><FolderOpen :size="16" />{{
            t("settings.navigation.storage")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="{ active: activeSection === 'models' }"
          @click="selectSection('models')"
          ><HardDrive :size="16" />{{
            t("settings.navigation.localModels")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="{ active: activeSection === 'openai' }"
          @click="selectSection('openai')"
          ><KeyRound :size="16" />{{
            t("settings.navigation.openAi")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="{ active: activeSection === 'shortcuts' }"
          @click="selectSection('shortcuts')"
          ><Keyboard :size="16" />{{
            t("settings.navigation.shortcuts")
          }}</AppButton
        >
        <AppButton
          variant="ghost"
          :class="{ active: activeSection === 'appearance' }"
          @click="selectSection('appearance')"
          ><Monitor :size="16" />{{
            t("settings.navigation.appearance")
          }}</AppButton
        >
      </nav>

      <div class="settings-content">
        <RecordingSettings />

        <StorageSettings />

        <LocalModelsSettings />

        <AppSurface id="openai">
          <div class="section-heading">
            <div>
              <h2>{{ t("settings.navigation.openAi") }}</h2>
              <p class="setting-description">
                {{ t("settings.openAi.description") }}
              </p>
            </div>
            <StatusPill v-if="openAi.credential?.configured" tone="success">
              {{ t("settings.openAi.configured") }}
            </StatusPill>
          </div>
          <div v-if="openAi.credential?.configured" class="model-row">
            <KeyRound :size="22" />
            <span
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
            class="setting-field openai-key-form"
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
            <AppButton type="submit" size="small">
              {{ t("settings.openAi.save") }}
            </AppButton>
          </form>
          <p v-if="openAi.connectionMessage" role="status">
            {{ openAi.connectionMessage }}
          </p>
        </AppSurface>

        <ShortcutsSettings />

        <AppSurface id="appearance">
          <div class="section-heading">
            <h2>{{ t("settings.navigation.appearance") }}</h2>
          </div>
          <label class="setting-field">
            <span>{{ t("settings.appearance.theme") }}</span>
            <AppSelect
              :model-value="theme"
              :options="themeOptions"
              :accessible-label="t('settings.appearance.theme')"
              @update:model-value="updateTheme"
            />
          </label>
          <div class="setting-field setting-toggle-row">
            <span>
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
      </div>
    </div>
  </div>
</template>
