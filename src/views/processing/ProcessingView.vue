<script setup lang="ts">
import { CircleCheck, CircleDashed } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type { Job } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const { t } = useI18n();

function sessionTitle(sessionId: string | null) {
  return (
    application.recentSessions.find((session) => session.id === sessionId)
      ?.title ?? t("processing.transcriptionJob")
  );
}

function progressValue(job: Job) {
  if (!job.progress?.total_units) {
    return undefined;
  }

  return (job.progress.completed_units / job.progress.total_units) * 100;
}

function progressLabel(job: Job) {
  const value = progressValue(job);
  return value === undefined ? undefined : Math.round(value);
}

function isCloudJob(job: Job) {
  return job.kind.includes("open_ai");
}

function jobKind(job: Job) {
  if (job.kind === "transcribe_open_ai") {
    return t("processing.openAiTranscription");
  }

  if (job.kind === "transcribe_local") {
    return t("processing.localTranscription");
  }

  return job.kind.replaceAll("_", " ");
}

function jobState(job: Job) {
  return t(`processing.states.${job.state}`);
}

function progressMessage(job: Job) {
  if (!job.progress) {
    return t("processing.waiting");
  }

  const {
    completed_units: completed,
    total_units: total,
    stage,
  } = job.progress;

  if (stage === "transcribing" && total) {
    return t("processing.transcribingProgress", { completed, total });
  }

  return t(`processing.stages.${stage}`);
}
</script>

<template>
  <div class="page processing-page">
    <header class="page-header page-header--compact">
      <h1>{{ t("processing.title") }}</h1>
    </header>

    <AppSurface class="processing-page__jobs" :padded="false">
      <div class="processing-page__scroll">
        <template v-if="application.activeJobs.length">
          <div class="processing-table__header">
            <span>{{ t("processing.job") }}</span
            ><span>{{ t("processing.provider") }}</span
            ><span>{{ t("processing.progress") }}</span
            ><span>{{ t("processing.status") }}</span>
          </div>
          <div
            v-for="job in application.activeJobs"
            :key="job.id"
            class="processing-row"
          >
            <div class="processing-row__title">
              <CircleDashed :size="18" class="spin" />
              <span
                ><strong>{{ sessionTitle(job.session_id) }}</strong
                ><small>{{ jobKind(job) }}</small></span
              >
            </div>
            <StatusPill :tone="isCloudJob(job) ? 'cloud' : 'local'">
              {{
                isCloudJob(job) ? t("processing.openAi") : t("processing.local")
              }}
            </StatusPill>
            <div>
              <div class="progress-label">
                <span>{{ progressMessage(job) }}</span>
                <strong v-if="progressLabel(job) !== undefined">
                  {{ progressLabel(job) }}%
                </strong>
              </div>
              <AppProgressBar
                :value="progressValue(job)"
                :accessible-label="progressMessage(job)"
              />
            </div>
            <div class="processing-row__status">
              <StatusPill tone="neutral">{{ jobState(job) }}</StatusPill>
              <AppButton
                v-if="job.state === 'failed'"
                size="small"
                variant="secondary"
                @click="application.retryJob(job.id)"
              >
                {{ t("processing.retry") }}
              </AppButton>
              <AppButton
                v-else-if="
                  ['queued', 'preparing', 'running'].includes(job.state)
                "
                size="small"
                variant="ghost"
                @click="application.cancelJob(job.id)"
              >
                {{ t("processing.cancel") }}
              </AppButton>
            </div>
          </div>
        </template>
        <AppEmptyState
          v-else
          :title="t('processing.empty')"
          :message="t('processing.emptyDescription')"
        >
          <template #icon><CircleCheck :size="21" /></template>
        </AppEmptyState>
      </div>
    </AppSurface>
  </div>
</template>
