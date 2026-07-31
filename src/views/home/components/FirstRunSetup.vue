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
import { useFirstRunSetup } from "@/views/home/components/useFirstRunSetup";
import SessionPlayback from "@/views/session/SessionPlayback.vue";

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
  <AppSurface class="first-run">
    <header class="first-run__header">
      <div>
        <h2>{{ t("firstRun.title") }}</h2>
        <p>{{ t("firstRun.description") }}</p>
      </div>
      <StatusPill v-if="phase === 'review'" tone="success">
        <Check :size="13" />{{ t("firstRun.testReady") }}
      </StatusPill>
    </header>

    <div v-if="phase === 'configure'" class="first-run__steps">
      <section class="first-run__step">
        <span class="first-run__step-icon"><HardDrive :size="17" /></span>
        <div class="first-run__step-body">
          <strong>{{ t("firstRun.library") }}</strong>
          <small v-if="application.snapshot?.library">
            {{
              t("firstRun.libraryReady", {
                path: application.snapshot.library.path,
              })
            }}
          </small>
          <small v-else>{{ t("home.chooseLibraryPrompt") }}</small>
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

      <section class="first-run__step first-run__step--sources">
        <span class="first-run__step-icon"><Mic :size="17" /></span>
        <div class="first-run__step-body">
          <strong>{{ t("firstRun.sources") }}</strong>
          <small>{{ t("firstRun.sourcesDescription") }}</small>
          <div class="first-run__fields">
            <label class="setting-field">
              <span>{{ t("firstRun.microphone") }}</span>
              <AppSelect
                :model-value="selectedMicrophone"
                :options="microphoneOptions"
                :accessible-label="t('firstRun.microphone')"
                :placeholder="t('settings.recording.noMicrophone')"
                @update:model-value="updateMicrophone"
              />
            </label>
            <div class="setting-field setting-toggle-row">
              <span>
                <strong>{{ t("firstRun.systemOutput") }}</strong>
                <small>{{ systemPermissionDescription }}</small>
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

      <section class="first-run__step">
        <span class="first-run__step-icon"><Headphones :size="17" /></span>
        <div class="first-run__step-body">
          <strong>{{ t("firstRun.transcription") }}</strong>
          <AppSelect
            class="first-run__mode-select"
            :model-value="recordingMode"
            :options="recordingModeOptions"
            :accessible-label="t('firstRun.transcription')"
            @update:model-value="updateRecordingMode"
          />
          <small v-if="recordingMode === 'local_live'">
            {{
              providerReady
                ? t("firstRun.localReady")
                : t("firstRun.localNeeded")
            }}
          </small>
          <small v-if="recordingMode === 'open_ai_live'">
            {{
              providerReady
                ? t("firstRun.openAiReady")
                : t("firstRun.openAiNeeded")
            }}
          </small>
        </div>
        <AppButton
          v-if="recordingMode === 'local_live' && !providerReady"
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

      <section class="first-run__step first-run__step--test">
        <span class="first-run__step-icon"><Play :size="17" /></span>
        <div class="first-run__step-body">
          <strong>{{ t("firstRun.sourceTest") }}</strong>
          <small>{{ t("firstRun.sourceTestDescription") }}</small>
          <small class="first-run__consent">{{ t("firstRun.consent") }}</small>
        </div>
        <AppButton
          variant="primary"
          :disabled="!canStartTest"
          @click="startSourceTest"
        >
          <span class="button-record-dot"></span>{{ t("firstRun.startTest") }}
        </AppButton>
      </section>
    </div>

    <div
      v-else-if="phase === 'running' || phase === 'stopping'"
      class="first-run__test-running"
    >
      <span class="recording-dot"></span>
      <div>
        <strong>
          {{
            phase === "stopping"
              ? t("recordingDock.stop")
              : t("firstRun.testRunning", { seconds: secondsRemaining })
          }}
        </strong>
        <small>{{ t("firstRun.sourceTestDescription") }}</small>
      </div>
      <AppProgressBar
        :value="sourceTestProgress"
        :accessible-label="t('firstRun.testProgress')"
      />
    </div>

    <div v-else class="first-run__review">
      <div>
        <h3>{{ t("firstRun.testReady") }}</h3>
        <p>{{ t("firstRun.testReadyDescription") }}</p>
      </div>
      <SessionPlayback
        v-if="audioSources.length"
        :sources="audioSources"
        :waveform="waveform"
        @reveal="revealTestSession"
      />
      <p v-else class="quiet-state">{{ t("firstRun.noPlayback") }}</p>
      <div class="first-run__review-actions">
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
      class="shell-state shell-state--error"
      role="alert"
    >
      {{ sourceTestError ?? application.operationError }}
    </p>
  </AppSurface>
</template>
