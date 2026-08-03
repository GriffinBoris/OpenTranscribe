<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type {
  DictationProvider,
  OpenAiTranscriptionModel,
} from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useGlobalShortcutStore } from "@/views/application/globalShortcutStore";
import {
  formatGlobalShortcut,
  shortcutFromKeyboardEvent,
} from "@/views/application/globalShortcutPresets";
import { useLocalModelsStore } from "@/views/application/localModelsStore";
import { openAiModelOptions } from "@/views/application/openAiModels";

const application = useApplicationStore();
const globalShortcut = useGlobalShortcutStore();
const localModels = useLocalModelsStore();
const { t } = useI18n();
const bindingDialogOpen = ref(false);
const capturedShortcut = ref<string | null>(null);

const enabled = computed(
  () => application.settings?.dictation_shortcut_enabled ?? true,
);
const selectedShortcut = computed(
  () => application.settings?.dictation_shortcut ?? "Alt+Space",
);
const provider = computed(
  () => application.settings?.dictation_provider ?? "local",
);
const localModel = computed(
  () => application.settings?.dictation_local_model_id ?? "",
);
const openAiModel = computed(
  () => application.settings?.dictation_openai_model ?? "gpt_transcribe",
);
const autoPaste = computed(
  () => application.settings?.dictation_auto_paste ?? true,
);
const status = computed(() => {
  const shortcut = globalShortcut.shortcuts.dictation;

  if (shortcut.isConfiguring) {
    return {
      label: t("settings.shortcuts.updating"),
      tone: "neutral" as const,
    };
  }

  if (shortcut.errorMessage) {
    return {
      label: t("settings.shortcuts.unavailable"),
      tone: "warning" as const,
    };
  }

  return shortcut.isRegistered
    ? { label: t("settings.shortcuts.active"), tone: "success" as const }
    : { label: t("settings.dictation.disabled"), tone: "neutral" as const };
});
const providerOptions = computed(() => [
  { label: t("settings.dictation.local"), value: "local" },
  { label: t("settings.dictation.openAi"), value: "open_ai" },
]);
const localModelOptions = computed(() =>
  localModels.installedModels.map((model) => ({
    label: model.label,
    value: model.id,
  })),
);

async function updateEnabled(value: boolean) {
  await application.saveSettings({ dictation_shortcut_enabled: value });
}

async function updateProvider(value: string) {
  await application.saveSettings({
    dictation_provider: value as DictationProvider,
  });
}

async function updateLocalModel(value: string) {
  await application.saveSettings({
    dictation_local_model_id: value || null,
  });
}

async function updateOpenAiModel(value: string) {
  await application.saveSettings({
    dictation_openai_model: value as OpenAiTranscriptionModel,
  });
}

async function updateAutoPaste(value: boolean) {
  await application.saveSettings({ dictation_auto_paste: value });
}

async function openShortcutCapture() {
  capturedShortcut.value = null;
  if (!(await globalShortcut.beginShortcutCapture())) {
    return;
  }
  bindingDialogOpen.value = true;
}

function captureShortcut(event: KeyboardEvent) {
  const shortcut = shortcutFromKeyboardEvent(event, {
    requireCommandOrControl: false,
  });

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

  const previousShortcut = selectedShortcut.value;

  if (!enabled.value) {
    if (
      await application.saveSettings({
        dictation_shortcut: capturedShortcut.value,
      })
    ) {
      bindingDialogOpen.value = false;
    }
    return;
  }

  if (
    !(await globalShortcut.configureCandidate(
      "dictation",
      capturedShortcut.value,
    ))
  ) {
    return;
  }

  if (
    !(await application.saveSettings({
      dictation_shortcut: capturedShortcut.value,
    }))
  ) {
    await globalShortcut.configureCandidate("dictation", previousShortcut);
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

onMounted(() => void localModels.load());
onBeforeUnmount(() => {
  window.removeEventListener("keydown", captureShortcut);

  if (bindingDialogOpen.value) {
    void globalShortcut.endShortcutCapture();
  }
});
</script>

<template>
  <AppSurface id="dictation">
    <div
      class="flex items-start justify-between gap-4 border-b border-[var(--divider)] pb-3.5"
    >
      <div>
        <h2 class="mb-[5px] text-2xl">{{ t("settings.dictation.title") }}</h2>
        <p class="text-ink mt-[3px] leading-[var(--line-height-body)]">
          {{ t("settings.dictation.description") }}
        </p>
      </div>
      <StatusPill :tone="status.tone" aria-live="polite">{{
        status.label
      }}</StatusPill>
    </div>

    <div
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid gap-1">
        <strong>{{ t("settings.dictation.enabled") }}</strong>
        <small>{{ t("settings.dictation.enabledDescription") }}</small>
      </span>
      <AppToggleSwitch
        :model-value="enabled"
        :accessible-label="t('settings.dictation.enabled')"
        :disabled="globalShortcut.shortcuts.dictation.isConfiguring"
        @update:model-value="updateEnabled"
      />
    </div>

    <div
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid gap-1">
        <strong>{{ t("settings.dictation.shortcut") }}</strong>
        <small
          v-if="globalShortcut.shortcuts.dictation.errorMessage"
          class="text-accent"
          role="alert"
        >
          {{
            t("settings.dictation.registrationError", {
              message: globalShortcut.shortcuts.dictation.errorMessage,
            })
          }}
        </small>
      </span>
      <AppButton
        variant="secondary"
        :disabled="globalShortcut.shortcuts.dictation.isConfiguring"
        @click="openShortcutCapture"
      >
        {{ formatGlobalShortcut(selectedShortcut) }}
      </AppButton>
    </div>

    <label
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span>{{ t("settings.dictation.provider") }}</span>
      <AppSelect
        :model-value="provider"
        :options="providerOptions"
        :accessible-label="t('settings.dictation.provider')"
        @update:model-value="updateProvider"
      />
    </label>

    <label
      v-if="provider === 'local'"
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span>{{ t("settings.dictation.localModel") }}</span>
      <AppSelect
        :model-value="localModel"
        :options="localModelOptions"
        :accessible-label="t('settings.dictation.localModel')"
        :placeholder="t('settings.dictation.chooseLocalModel')"
        @update:model-value="updateLocalModel"
      />
    </label>

    <label
      v-else
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span>{{ t("settings.dictation.openAiModel") }}</span>
      <AppSelect
        :model-value="openAiModel"
        :options="openAiModelOptions(t)"
        :accessible-label="t('settings.dictation.openAiModel')"
        @update:model-value="updateOpenAiModel"
      />
    </label>

    <div
      class="grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid gap-1">
        <strong>{{ t("settings.dictation.autoPaste") }}</strong>
        <small>{{ t("settings.dictation.autoPasteDescription") }}</small>
      </span>
      <AppToggleSwitch
        :model-value="autoPaste"
        :accessible-label="t('settings.dictation.autoPaste')"
        @update:model-value="updateAutoPaste"
      />
    </div>

    <AppDialog
      :open="bindingDialogOpen"
      :title="t('settings.dictation.captureTitle')"
      @update:open="bindingDialogOpen = $event"
    >
      <div class="grid gap-3">
        <p class="m-0">{{ t("settings.dictation.captureDescription") }}</p>
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
        <AppButton variant="ghost" @click="bindingDialogOpen = false">{{
          t("settings.shortcuts.cancel")
        }}</AppButton>
        <AppButton
          :disabled="
            !capturedShortcut ||
            globalShortcut.shortcuts.dictation.isConfiguring
          "
          :loading="globalShortcut.shortcuts.dictation.isConfiguring"
          @click="saveCapturedShortcut"
        >
          {{ t("settings.shortcuts.save") }}
        </AppButton>
      </template>
    </AppDialog>
  </AppSurface>
</template>
