<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppSelect from "@/components/ui/AppSelect.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import type { ThemePreference } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const { t } = useI18n();
const theme = ref<ThemePreference>(
  application.settings?.appearance.theme ?? "system",
);
const reducedMotion = ref(
  application.settings?.appearance.reduced_motion ?? false,
);

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
</script>

<template>
  <AppSurface id="settings-appearance">
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
        <small>{{ t("settings.appearance.reducedMotionDescription") }}</small>
      </span>
      <AppToggleSwitch
        :model-value="reducedMotion"
        :accessible-label="t('settings.appearance.reducedMotion')"
        @update:model-value="updateReducedMotion"
      />
    </div>
  </AppSurface>
</template>
