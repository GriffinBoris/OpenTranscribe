<script setup lang="ts">
import {
  Check,
  FolderOpen,
  HardDrive,
  Headphones,
  Mic,
  Play,
} from "@lucide/vue";
import { computed } from "vue";
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
const setupSteps = computed(() => [
  t("firstRun.library"),
  t("firstRun.sources"),
  t("firstRun.transcription"),
  t("firstRun.sourceTest"),
]);
const {
  application,
  phase,
  setupStep,
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
  isFirstSetupStep,
  isLastSetupStep,
  chooseLibrary,
  updateMicrophone,
  updateSystemCapture,
  updateRecordingMode,
  previousSetupStep,
  nextSetupStep,
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
      class="flex items-start justify-between gap-5 border-b border-[var(--divider)] pb-[18px]"
    >
      <div>
        <h2 class="mb-[5px] text-2xl">{{ t("firstRun.title") }}</h2>
        <p class="text-md text-ink-muted m-0">
          {{ t("firstRun.description") }}
        </p>
      </div>
      <AppButton
        v-if="phase === 'configure'"
        variant="ghost"
        size="small"
        :loading="isFinishing"
        @click="finishSetup"
      >
        {{ t("firstRun.skip") }}
      </AppButton>
      <StatusPill v-else-if="phase === 'review'" tone="success">
        <Check :size="13" />{{ t("firstRun.testReady") }}
      </StatusPill>
    </header>

    <div v-if="phase === 'configure'" class="grid gap-6 pt-5">
      <ol class="flex items-start" :aria-label="t('firstRun.setupProgress')">
        <li
          v-for="(step, index) in setupSteps"
          :key="step"
          class="grid min-w-0 flex-1 grid-cols-[auto_minmax(8px,1fr)] items-start last:flex-none"
          :aria-current="index + 1 === setupStep ? 'step' : undefined"
        >
          <div class="grid min-w-0 justify-items-center gap-1.5">
            <span
              class="grid size-7 place-items-center rounded-full border text-sm font-semibold transition-[background-color,border-color,color] duration-[var(--duration-standard)] ease-[var(--easing-standard)]"
              :class="
                index + 1 <= setupStep
                  ? 'border-accent bg-accent text-accent-contrast'
                  : 'border-line bg-surface text-ink-muted'
              "
            >
              <Check v-if="index + 1 < setupStep" :size="14" />
              <span v-else>{{ index + 1 }}</span>
            </span>
            <span
              class="text-ink-muted max-w-[88px] truncate text-center text-xs font-medium transition-colors duration-[var(--duration-standard)] ease-[var(--easing-standard)] max-[620px]:hidden"
              :class="{ 'text-ink': index + 1 === setupStep }"
              >{{ step }}</span
            >
          </div>
          <span
            v-if="index < setupSteps.length - 1"
            class="mt-[13px] h-px min-w-2 flex-1 transition-colors duration-[var(--duration-standard)] ease-[var(--easing-standard)]"
            :class="index + 1 < setupStep ? 'bg-accent' : 'bg-[var(--divider)]'"
          ></span>
        </li>
      </ol>

      <Transition name="setup-step" mode="out-in">
        <section v-if="setupStep === 1" key="library" class="grid gap-5">
          <div class="flex items-start gap-3.5">
            <span
              class="rounded-app-md bg-lichen-soft text-lichen grid size-[34px] shrink-0 place-items-center"
            >
              <HardDrive :size="17" />
            </span>
            <div class="grid gap-1">
              <h3 class="m-0 text-xl">{{ t("firstRun.library") }}</h3>
              <p class="text-ink-muted m-0 text-sm">
                {{
                  application.snapshot?.library
                    ? t("firstRun.libraryReady", {
                        path: application.snapshot.library.path,
                      })
                    : t("home.chooseLibraryPrompt")
                }}
              </p>
            </div>
          </div>
          <div
            class="rounded-app-md bg-canvas-subtle flex items-center justify-between gap-3 border border-[var(--border)] px-4 py-3 max-[600px]:flex-col max-[600px]:items-start"
          >
            <StatusPill v-if="application.snapshot?.library" tone="success">
              <Check :size="13" />{{ t("firstRun.ready") }}
            </StatusPill>
            <AppButton
              size="small"
              :loading="isChoosingLibrary"
              @click="chooseLibrary"
            >
              <FolderOpen :size="15" />{{
                application.snapshot?.library
                  ? t("firstRun.changeLibrary")
                  : t("firstRun.chooseLibrary")
              }}
            </AppButton>
          </div>
        </section>

        <section v-else-if="setupStep === 2" key="audio" class="grid gap-5">
          <div class="flex items-start gap-3.5">
            <span
              class="rounded-app-md bg-lichen-soft text-lichen grid size-[34px] shrink-0 place-items-center"
            >
              <Mic :size="17" />
            </span>
            <div class="grid gap-1">
              <h3 class="m-0 text-xl">{{ t("firstRun.sources") }}</h3>
              <p class="text-ink-muted m-0 text-sm">
                {{ t("firstRun.sourcesDescription") }}
              </p>
            </div>
          </div>
          <div
            class="rounded-app-md overflow-hidden border border-[var(--border)]"
          >
            <label
              class="grid grid-cols-[minmax(140px,1fr)_minmax(220px,1.3fr)] items-center gap-5 px-4 py-3 max-[620px]:grid-cols-1"
            >
              <span class="font-semibold">{{ t("firstRun.microphone") }}</span>
              <AppSelect
                :model-value="selectedMicrophone"
                :options="microphoneOptions"
                :accessible-label="t('firstRun.microphone')"
                :placeholder="t('settings.recording.noMicrophone')"
                @update:model-value="updateMicrophone"
              />
            </label>
            <div
              class="grid grid-cols-[minmax(140px,1fr)_auto] items-center gap-5 border-t border-[var(--divider)] px-4 py-3 max-[620px]:grid-cols-[minmax(0,1fr)_auto]"
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
                  class="justify-self-start"
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
        </section>

        <section
          v-else-if="setupStep === 3"
          key="transcription"
          class="grid gap-5"
        >
          <div class="flex items-start gap-3.5">
            <span
              class="rounded-app-md bg-lichen-soft text-lichen grid size-[34px] shrink-0 place-items-center"
            >
              <Headphones :size="17" />
            </span>
            <div class="grid gap-1">
              <h3 class="m-0 text-xl">{{ t("firstRun.transcription") }}</h3>
              <p class="text-ink-muted m-0 text-sm">
                {{ t("firstRun.decideLater") }}
              </p>
            </div>
          </div>
          <div
            class="rounded-app-md grid gap-3 border border-[var(--border)] p-4"
          >
            <AppSelect
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
            <small
              v-if="recordingMode === 'open_ai_live'"
              class="text-ink-muted"
            >
              {{
                providerReady
                  ? t("firstRun.openAiReady")
                  : t("firstRun.openAiNeeded")
              }}
            </small>
            <AppButton
              v-if="recordingMode === 'local_after_recording' && !providerReady"
              class="justify-self-start"
              size="small"
              @click="openSettings('models')"
            >
              {{ t("firstRun.configureLocal") }}
            </AppButton>
            <AppButton
              v-if="recordingMode === 'open_ai_live' && !providerReady"
              class="justify-self-start"
              size="small"
              @click="openSettings('openai')"
            >
              {{ t("firstRun.configureOpenAi") }}
            </AppButton>
          </div>
        </section>

        <section v-else-if="isLastSetupStep" key="test" class="grid gap-5">
          <div class="flex items-start gap-3.5">
            <span
              class="rounded-app-md bg-lichen-soft text-lichen grid size-[34px] shrink-0 place-items-center"
            >
              <Play :size="17" />
            </span>
            <div class="grid gap-1">
              <span class="flex items-center gap-2">
                <h3 class="m-0 text-xl">{{ t("firstRun.sourceTest") }}</h3>
                <StatusPill tone="neutral">{{
                  t("firstRun.sourceTestOptional")
                }}</StatusPill>
              </span>
              <p class="text-ink-muted m-0 text-sm">
                {{ t("firstRun.sourceTestDescription") }}
              </p>
            </div>
          </div>
          <p class="text-warning m-0 text-sm">{{ t("firstRun.consent") }}</p>
        </section>
      </Transition>

      <footer
        class="flex items-center justify-between gap-3 border-t border-[var(--divider)] pt-4"
      >
        <AppButton
          v-if="!isFirstSetupStep"
          variant="ghost"
          @click="previousSetupStep"
        >
          {{ t("firstRun.back") }}
        </AppButton>
        <span v-else></span>
        <div class="flex items-center gap-2">
          <AppButton
            v-if="isLastSetupStep"
            variant="secondary"
            :disabled="!canStartTest"
            @click="startSourceTest"
          >
            {{ t("firstRun.startTest") }}
          </AppButton>
          <AppButton
            v-if="isLastSetupStep"
            variant="primary"
            :loading="isFinishing"
            @click="finishSetup"
          >
            {{ t("firstRun.continue") }}
          </AppButton>
          <AppButton v-else variant="primary" @click="nextSetupStep">
            {{ t("firstRun.next") }}
          </AppButton>
        </div>
      </footer>
    </div>

    <div
      v-else-if="phase === 'running' || phase === 'stopping'"
      class="grid min-h-[150px] grid-cols-[auto_minmax(0,1fr)] items-center gap-3.5 pt-5"
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

    <div v-else class="grid gap-4 pt-5">
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
      <div class="flex justify-end gap-2">
        <AppButton variant="ghost" @click="openTestSession">
          {{ t("firstRun.openSession") }}
        </AppButton>
        <AppButton
          variant="primary"
          :loading="isFinishing"
          @click="finishSetup"
        >
          {{ t("firstRun.continue") }}
        </AppButton>
      </div>
    </div>

    <p
      v-if="sourceTestError || application.operationError"
      class="text-accent grid min-h-[120px] place-content-center justify-items-center gap-2 text-center"
      role="alert"
    >
      {{ sourceTestError ?? application.operationError }}
    </p>
  </AppSurface>
</template>

<style scoped>
.setup-step-enter-active,
.setup-step-leave-active {
  transition:
    opacity var(--duration-standard) var(--easing-standard),
    transform var(--duration-standard) var(--easing-standard);
}

.setup-step-enter-from {
  opacity: 0;
  transform: translateX(8px);
}

.setup-step-leave-to {
  opacity: 0;
  transform: translateX(-6px);
}
</style>
