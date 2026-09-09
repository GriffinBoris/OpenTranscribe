<script setup lang="ts">
import { CircleAlert, CircleCheck, CircleDashed } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type { Job } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { jobEstimateDetails } from "@/views/application/jobEstimates";

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

function progressSummary(job: Job) {
  if (!job.progress) {
    return jobState(job);
  }

  const stage = t(`processing.stages.${job.progress.stage}`);
  const progress = progressLabel(job);

  return progress === undefined ? stage : `${stage} · ${progress}%`;
}

function isCloudJob(job: Job) {
  return job.kind.includes("open_ai");
}

function jobProvider(job: Job) {
  if (job.kind === "finalize_recording") {
    return t("processing.onDevice");
  }

  return isCloudJob(job) ? t("processing.openAi") : t("processing.local");
}

function jobKind(job: Job) {
  if (job.kind === "finalize_recording") {
    return t("processing.recordingFinalization");
  }

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

function estimateDetails(job: Job) {
  return jobEstimateDetails(job, (key, parameters) =>
    parameters ? t(key, parameters) : t(key),
  ).join(" · ");
}
</script>

<template>
  <div
    class="page processing-page grid h-full min-h-0 w-full grid-rows-[auto_minmax(0,1fr)] overflow-hidden px-[var(--layout-page-gutter)] pt-[var(--space-13)] pb-[var(--space-14)]"
  >
    <header
      class="page-header page-header--compact mb-[var(--space-5-5)] flex items-start justify-between gap-6"
    >
      <h1
        class="text-display m-0 max-w-[730px] leading-[var(--line-height-tight)] font-bold tracking-[-0.035em]"
      >
        {{ t("processing.title") }}
      </h1>
    </header>

    <section
      class="processing-page__jobs grid min-h-0 grid-rows-[minmax(0,1fr)]"
    >
      <div class="processing-page__scroll min-h-0 overflow-auto">
        <template v-if="application.activeJobs.length">
          <div
            class="processing-table__header bg-canvas-subtle text-ink sticky top-0 z-[var(--layer-content)] grid grid-cols-[minmax(220px,1.4fr)_110px_minmax(200px,1fr)_150px] items-center gap-x-6 border-b border-[var(--divider)] px-[18px] py-[14px] text-xs font-bold tracking-[0.06em] uppercase max-[1100px]:hidden"
          >
            <span>{{ t("processing.job") }}</span
            ><span>{{ t("processing.provider") }}</span
            ><span>{{ t("processing.progress") }}</span
            ><span>{{ t("processing.status") }}</span>
          </div>
          <div
            v-for="job in application.activeJobs"
            :key="job.id"
            class="processing-row grid grid-cols-[minmax(220px,1.4fr)_110px_minmax(200px,1fr)_150px] items-center gap-x-6 border-b border-[var(--divider)] px-[18px] py-[16px] last:border-b-0 max-[1100px]:grid-cols-[minmax(0,1fr)_minmax(220px,0.8fr)] max-[1100px]:gap-x-5 max-[1100px]:gap-y-4 max-[700px]:grid-cols-1"
          >
            <div
              class="processing-row__title flex min-w-0 items-center gap-2.5 max-[1100px]:col-span-2 max-[700px]:col-span-1"
            >
              <CircleAlert
                v-if="job.state === 'failed'"
                :size="18"
                class="shrink-0 text-[var(--warning)]"
              />
              <CircleDashed
                v-else
                :size="18"
                class="shrink-0 animate-[spin_1.4s_linear_infinite]"
              />
              <span class="grid min-w-0 gap-1">
                <RouterLink
                  v-if="job.session_id"
                  :to="`/sessions/${job.session_id}`"
                  class="truncate font-semibold hover:underline"
                  :title="sessionTitle(job.session_id)"
                  >{{ sessionTitle(job.session_id) }}</RouterLink
                >
                <strong v-else>{{ sessionTitle(job.session_id) }}</strong>
                <small
                  :class="job.error_message ? 'text-[var(--warning)]' : ''"
                  >{{ job.error_message ?? jobKind(job) }}</small
                ></span
              >
            </div>
            <div
              class="processing-row__provider flex min-w-0 flex-col items-start gap-1.5"
            >
              <span
                class="text-ink-muted hidden text-xs font-bold tracking-[0.06em] uppercase max-[1100px]:block"
              >
                {{ t("processing.provider") }}
              </span>
              <StatusPill :tone="isCloudJob(job) ? 'cloud' : 'local'">
                {{ jobProvider(job) }}
              </StatusPill>
            </div>
            <div class="processing-row__progress min-w-0">
              <span
                class="text-ink-muted mb-1.5 hidden text-xs font-bold tracking-[0.06em] uppercase max-[1100px]:block"
              >
                {{ t("processing.progress") }}
              </span>
              <div
                class="progress-label text-ink mb-[7px] flex justify-between gap-2.5 text-xs"
              >
                <strong>{{ progressSummary(job) }}</strong>
              </div>
              <AppProgressBar
                :value="progressValue(job)"
                :accessible-label="t('processing.progress')"
              />
              <small
                v-if="estimateDetails(job)"
                class="text-ink mt-1.5 block text-xs"
              >
                {{ estimateDetails(job) }}
              </small>
            </div>
            <div
              class="processing-row__status flex min-w-0 items-center justify-end gap-2 max-[1100px]:col-span-2 max-[700px]:col-span-1 max-[700px]:justify-start"
            >
              <span
                class="text-ink-muted mr-auto text-xs font-bold tracking-[0.06em] uppercase min-[1101px]:hidden"
              >
                {{ t("processing.status") }}
              </span>
              <StatusPill
                :tone="job.state === 'failed' ? 'warning' : 'neutral'"
                >{{ jobState(job) }}</StatusPill
              >
              <AppButton
                v-if="job.state === 'failed'"
                size="small"
                variant="secondary"
                @click="application.retryJob(job.id)"
              >
                {{ t("processing.retry") }}
              </AppButton>
              <AppButton
                v-if="
                  job.kind !== 'finalize_recording' &&
                  ['queued', 'preparing', 'running', 'failed'].includes(
                    job.state,
                  )
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
    </section>
  </div>
</template>
