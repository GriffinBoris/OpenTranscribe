<script setup lang="ts">
import {
  Check,
  FolderOpen,
  HardDrive,
  Headphones,
  Mic,
  Play,
} from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppToggleSwitch from "@/components/ui/AppToggleSwitch.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { useFirstRunSetup } from "@/views/home/useFirstRunSetup";
import SessionPlayback from "@/views/session/components/SessionPlayback.vue";

const { t } = useI18n();
const {
  application,
  phase,
  audioSources,
  waveform,
  sourceTestError,
  isChoosingLibrary,
  isFinishing,
  selectedMicrophone,
  recordingMode,
  captureSystemAudio,
  microphoneOptions,
  systemPermissionDescription,
  recordingModeOptions,
  providerReady,
  sourceTestProgress,
  secondsRemaining,
  canStartTest,
  chooseLibrary,
  updateMicrophone,
  updateSystemCapture,
  updateRecordingMode,
  startSourceTest,
  finishSetup,
  openTestSession,
  revealTestSession,
  openSettings,
} = useFirstRunSetup();
</script>

<template>
  <AppSurface class="first-run mb-7 w-full">
    <header
      class="first-run__header flex items-start justify-between gap-5 border-b border-[var(--divider)] pb-[18px]"
    >
      <div>
        <h2 class="mb-[5px] text-2xl">{{ t("firstRun.title") }}</h2>
        <p class="text-md text-ink-muted m-0">
          {{ t("firstRun.description") }}
        </p>
      </div>
      <StatusPill v-if="phase === 'review'" tone="success">
        <Check :size="13" />{{ t("firstRun.testReady") }}
      </StatusPill>
    </header>

    <div v-if="phase === 'configure'" class="first-run__steps grid">
      <section
        class="first-run__step grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3.5 border-b border-[var(--divider)] py-4"
      >
        <span
          class="first-run__step-icon rounded-app-md bg-lichen-soft text-lichen grid size-[34px] place-items-center"
          ><HardDrive :size="17"
        /></span>
        <div class="first-run__step-body grid min-w-0 gap-[var(--space-1-5)]">
          <strong>{{ t("firstRun.library") }}</strong>
          <small v-if="application.snapshot?.library" class="text-ink-muted">
            {{
              t("firstRun.libraryReady", {
                path: application.snapshot.library.path,
              })
            }}
          </small>
          <small v-else class="text-ink-muted">{{
            t("home.chooseLibraryPrompt")
          }}</small>
        </div>
        <StatusPill v-if="application.snapshot?.library" tone="success">
          <Check :size="13" />{{ t("settings.openAi.configured") }}
        </StatusPill>
        <AppButton
          v-else
          size="small"
          :loading="isChoosingLibrary"
          @click="chooseLibrary"
        >
          <FolderOpen :size="15" />{{ t("firstRun.chooseLibrary") }}
        </AppButton>
      </section>

      <section
        class="first-run__step first-run__step--sources grid grid-cols-[auto_minmax(0,1fr)_auto] items-start gap-3.5 border-b border-[var(--divider)] py-4"
      >
        <span
          class="first-run__step-icon rounded-app-md bg-lichen-soft text-lichen grid size-[34px] place-items-center"
          ><Mic :size="17"
        /></span>
        <div class="first-run__step-body grid min-w-0 gap-[var(--space-1-5)]">
          <strong>{{ t("firstRun.sources") }}</strong>
          <small class="text-ink-muted">{{
            t("firstRun.sourcesDescription")
          }}</small>
          <div class="first-run__fields mt-2 grid">
            <label
              class="setting-field m-0 grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3"
            >
              <span>{{ t("firstRun.microphone") }}</span>
              <AppSelect
                :model-value="selectedMicrophone"
                :options="microphoneOptions"
                :accessible-label="t('firstRun.microphone')"
                :placeholder="t('settings.recording.noMicrophone')"
                @update:model-value="updateMicrophone"
              />
            </label>
            <div
              class="setting-field setting-toggle-row grid grid-cols-[minmax(160px,1fr)_minmax(220px,1.3fr)] items-center gap-5 border-t border-[var(--divider)] py-3"
            >
              <span class="grid gap-1">
                <strong>{{ t("firstRun.systemOutput") }}</strong>
                <small class="text-ink-muted">{{
                  systemPermissionDescription
                }}</small>
                <AppButton
                  v-if="
                    application.audioDevices
                      ?.system_audio_permission_settings_available &&
                    application.audioDevices.system_audio_permission_granted ===
                      false
                  "
                  variant="ghost"
                  size="small"
                  @click="application.openSystemAudioPermissionSettings"
                >
                  {{ t("settings.recording.openSystemSettings") }}
                </AppButton>
              </span>
              <AppToggleSwitch
                :model-value="captureSystemAudio"
                :accessible-label="t('firstRun.systemOutput')"
                :disabled="!application.audioDevices?.system_audio_available"
                @update:model-value="updateSystemCapture"
              />
            </div>
          </div>
        </div>
      </section>

      <section
        class="first-run__step grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3.5 border-b border-[var(--divider)] py-4"
      >
        <span
          class="first-run__step-icon rounded-app-md bg-lichen-soft text-lichen grid size-[34px] place-items-center"
          ><Headphones :size="17"
        /></span>
        <div class="first-run__step-body grid min-w-0 gap-[var(--space-1-5)]">
          <strong>{{ t("firstRun.transcription") }}</strong>
          <AppSelect
            class="first-run__mode-select mt-1 w-full max-w-80"
            :model-value="recordingMode"
            :options="recordingModeOptions"
            :accessible-label="t('firstRun.transcription')"
            @update:model-value="updateRecordingMode"
          />
          <small
            v-if="recordingMode === 'local_after_recording'"
            class="text-ink-muted"
          >
            {{
              providerReady
                ? t("firstRun.localReady")
                : t("firstRun.localNeeded")
            }}
          </small>
          <small v-if="recordingMode === 'open_ai_live'" class="text-ink-muted">
            {{
              providerReady
                ? t("firstRun.openAiReady")
                : t("firstRun.openAiNeeded")
            }}
          </small>
        </div>
        <AppButton
          v-if="recordingMode === 'local_after_recording' && !providerReady"
          @click="openSettings('models')"
        >
          {{ t("firstRun.configureLocal") }}
        </AppButton>
        <AppButton
          v-if="recordingMode === 'open_ai_live' && !providerReady"
          @click="openSettings('openai')"
        >
          {{ t("firstRun.configureOpenAi") }}
        </AppButton>
      </section>

      <section
        class="first-run__step first-run__step--test grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3.5 py-4 last:border-b-0 last:pb-0"
      >
        <span
          class="first-run__step-icon rounded-app-md bg-lichen-soft text-lichen grid size-[34px] place-items-center"
          ><Play :size="17"
        /></span>
        <div class="first-run__step-body grid min-w-0 gap-[var(--space-1-5)]">
          <strong>{{ t("firstRun.sourceTest") }}</strong>
          <small class="text-ink-muted">{{
            t("firstRun.sourceTestDescription")
          }}</small>
          <small class="first-run__consent text-warning">{{
            t("firstRun.consent")
          }}</small>
        </div>
        <AppButton
          variant="primary"
          :disabled="!canStartTest"
          @click="startSourceTest"
        >
          <span
            class="text-accent-contrast size-[9px] rounded-full bg-current"
          ></span
          >{{ t("firstRun.startTest") }}
        </AppButton>
      </section>
    </div>

    <div
      v-else-if="phase === 'running' || phase === 'stopping'"
      class="first-run__test-running grid min-h-[150px] grid-cols-[auto_minmax(0,1fr)] items-center gap-3.5"
    >
      <span
        class="recording-dot bg-accent size-3 animate-[pulse_1.25s_ease-in-out_infinite] rounded-full"
      ></span>
      <div class="grid gap-1">
        <strong>
          {{
            phase === "stopping"
              ? t("recordingDock.stop")
              : t("firstRun.testRunning", { seconds: secondsRemaining })
          }}
        </strong>
        <small class="text-ink-muted">{{
          t("firstRun.sourceTestDescription")
        }}</small>
      </div>
      <AppProgressBar
        class="col-span-full"
        :value="sourceTestProgress"
        :accessible-label="t('firstRun.testProgress')"
      />
    </div>

    <div v-else class="first-run__review grid gap-4 pt-[18px]">
      <div>
        <h3 class="mb-[5px] text-2xl">{{ t("firstRun.testReady") }}</h3>
        <p class="text-md text-ink-muted m-0">
          {{ t("firstRun.testReadyDescription") }}
        </p>
      </div>
      <SessionPlayback
        v-if="audioSources.length"
        class="rounded-app-md border border-[var(--border)]"
        :sources="audioSources"
        :waveform="waveform"
        @reveal="revealTestSession"
      />
      <p v-else class="quiet-state text-md text-ink-muted m-0 py-[18px]">
        {{ t("firstRun.noPlayback") }}
      </p>
      <div class="first-run__review-actions flex justify-end gap-2">
        <AppButton variant="ghost" @click="openTestSession">
          {{ t("firstRun.openSession") }}
        </AppButton>
        <AppButton
          variant="primary"
          :loading="isFinishing"
          @click="finishSetup"
        >
          {{ t("firstRun.finish") }}
        </AppButton>
      </div>
    </div>

    <p
      v-if="sourceTestError || application.operationError"
      class="text-accent grid h-full min-h-[220px] place-content-center justify-items-center gap-2 text-center"
      role="alert"
    >
      {{ sourceTestError ?? application.operationError }}
    </p>
  </AppSurface>
</template>
