<script setup lang="ts">
import { useDictationStore } from "@/views/application/dictationStore";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import WindowTitlebar from "@/views/application/components/WindowTitlebar.vue";
import AppButton from "@/components/ui/AppButton.vue";
import AppStatusState from "@/components/ui/AppStatusState.vue";
import AppSidebar from "@/views/application/components/AppSidebar.vue";
import LocalModelDownloadStatus from "@/views/application/components/LocalModelDownloadStatus.vue";
import RecordingDock from "@/views/application/components/RecordingDock.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { native } from "@/core/native";
import { useGlobalShortcutStore } from "@/views/application/globalShortcutStore";
import { useLocalModelsStore } from "@/views/application/localModelsStore";
import { useRecordingStore } from "@/views/application/recordingStore";
import { useUpdateStore } from "@/views/application/updateStore";

const dictation = useDictationStore();
const application = useApplicationStore();
const globalShortcut = useGlobalShortcutStore();
const localModels = useLocalModelsStore();
const recording = useRecordingStore();
const updater = useUpdateStore();
const { t } = useI18n();
const router = useRouter();
const isHandlingGlobalShortcut = ref(false);
const isHandlingDictationShortcut = ref(false);
const isStopping = ref(false);
const hasActiveWork = computed(
  () =>
    Boolean(recording.activeRecording) ||
    dictation.isActive ||
    application.runningJobs.length > 0 ||
    localModels.downloadingModelId !== null,
);
const globalShortcutAccelerator = computed(() => {
  const settings = application.settings;

  if (!settings?.global_shortcut_enabled) {
    return null;
  }

  return settings.global_shortcut;
});
const dictationShortcutAccelerator = computed(() => {
  const settings = application.settings;

  if (!settings?.dictation_shortcut_enabled) {
    return null;
  }

  return settings.dictation_shortcut;
});

function refreshApplicationOnFocus() {
  void dictation.refresh();
  if (application.isLoading) {
    return;
  }

  if (!recording.activeRecording) {
    void application.loadAudioDevices();
  }

  if (recording.activeRecording || application.runningJobs.length > 0) {
    return;
  }

  void application.refreshLibrary();
}

onMounted(() => {
  void dictation.refresh();
  window.addEventListener("focus", refreshApplicationOnFocus);
});
onBeforeUnmount(() => {
  window.removeEventListener("focus", refreshApplicationOnFocus);
  void globalShortcut.configure(
    "recording",
    null,
    toggleRecordingFromGlobalShortcut,
  );
  void globalShortcut.configure(
    "dictation",
    null,
    toggleDictationFromGlobalShortcut,
  );
});

watch(
  () => application.importRequestRevision,
  async () => {
    const session = await application.importMedia();

    if (session) {
      await router.push(`/sessions/${session.id}`);
    }
  },
);

async function stopRecording() {
  isStopping.value = true;

  try {
    await recording.stop();
  } catch (reason) {
    application.operationError =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isStopping.value = false;
  }
}

async function installUpdate() {
  if (hasActiveWork.value) {
    return;
  }

  await updater.installUpdate();
}

async function toggleRecordingFromGlobalShortcut() {
  if (isHandlingGlobalShortcut.value || isStopping.value) {
    return;
  }

  isHandlingGlobalShortcut.value = true;

  try {
    if (recording.activeRecording) {
      await stopRecording();
      return;
    }

    const session = await recording.createSession(
      application.settings?.recording_mode ?? "record_only",
    );

    if (session) {
      await router.push(`/sessions/${session.id}`);
    }
  } finally {
    isHandlingGlobalShortcut.value = false;
  }
}

async function toggleDictationFromGlobalShortcut() {
  if (isHandlingDictationShortcut.value) {
    return;
  }

  isHandlingDictationShortcut.value = true;

  try {
    await native.toggleDictation();
  } catch (reason) {
    application.operationError =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isHandlingDictationShortcut.value = false;
  }
}

watch(
  globalShortcutAccelerator,
  (shortcut) => {
    void globalShortcut.configure(
      "recording",
      shortcut,
      toggleRecordingFromGlobalShortcut,
    );
  },
  { immediate: true },
);

watch(
  dictationShortcutAccelerator,
  (shortcut) => {
    void globalShortcut.configure(
      "dictation",
      shortcut,
      toggleDictationFromGlobalShortcut,
    );
  },
  { immediate: true },
);
</script>

<template>
  <div
    class="application-shell bg-sidebar relative grid h-full grid-cols-[var(--layout-sidebar-width)_minmax(0,1fr)] grid-rows-[var(--layout-titlebar-height)_minmax(0,1fr)] gap-x-2 pr-2 pb-2 max-[900px]:grid-cols-[78px_minmax(0,1fr)]"
  >
    <WindowTitlebar />
    <AppSidebar />

    <main
      class="application-main rounded-app-xl bg-surface shadow-app-pane col-start-2 row-start-2 flex min-h-0 min-w-0 flex-col overflow-hidden border border-[color-mix(in_srgb,var(--border)_82%,transparent)]"
    >
      <AppStatusState
        v-if="application.isLoading"
        :title="t('shell.openingLibrary')"
        loading
      />
      <AppStatusState
        v-else-if="application.error"
        :title="t('shell.openingFailed')"
        :message="application.error"
        :retry-label="t('shell.retry')"
        error
        @retry="application.bootstrap"
      />
      <div
        v-else
        class="application-main__content flex min-h-0 flex-1 flex-col"
      >
        <p
          v-if="application.operationError"
          class="shell-state shell-state--error shell-operation-error rounded-app-sm mx-5 mt-3.5 block h-auto min-h-0 border border-[color-mix(in_srgb,var(--accent)_28%,var(--border))] bg-[color-mix(in_srgb,var(--accent)_7%,var(--surface))] px-3 py-2.5 text-left"
          role="alert"
        >
          {{ application.operationError }}
        </p>
        <div
          v-if="updater.availableUpdate"
          class="shell-state rounded-app-sm bg-canvas-subtle mx-5 mt-3.5 flex items-center justify-between gap-3 border border-[var(--border)] px-3 py-2.5 max-[600px]:items-start"
        >
          <span class="grid gap-1">
            <span class="text-sm font-medium">
              {{
                t("settings.application.updates.available", {
                  version: updater.availableUpdate.version,
                })
              }}
            </span>
            <small v-if="updater.error" class="text-accent" role="alert">
              {{ updater.error }}
            </small>
            <small v-else-if="hasActiveWork" class="text-warning">
              {{ t("settings.application.updates.busy") }}
            </small>
          </span>
          <AppButton
            variant="ghost"
            size="small"
            :disabled="hasActiveWork"
            :loading="updater.isInstalling"
            @click="installUpdate"
          >
            {{ t("settings.application.updates.update") }}
          </AppButton>
        </div>
        <LocalModelDownloadStatus />
        <div class="application-main__route min-h-0 flex-1">
          <RouterView v-slot="{ Component }">
            <Transition name="workspace-fade" mode="out-in">
              <component :is="Component" :key="$route.path" />
            </Transition>
          </RouterView>
        </div>
      </div>
    </main>

    <RecordingDock
      v-if="recording.activeRecording"
      :stopping="isStopping"
      @stop="stopRecording"
    />
  </div>
</template>
