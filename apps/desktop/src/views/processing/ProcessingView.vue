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

    <AppSurface
      class="processing-page__jobs grid min-h-0 grid-rows-[minmax(0,1fr)]"
      :padded="false"
    >
      <div class="processing-page__scroll min-h-0 overflow-auto">
        <template v-if="application.activeJobs.length">
          <div
            class="processing-table__header bg-canvas-subtle text-ink-muted sticky top-0 z-[var(--layer-content)] grid grid-cols-[minmax(0,1.2fr)_max-content_minmax(220px,1fr)_max-content] items-center gap-5 border-b border-[var(--divider)] px-[18px] py-[14px] text-xs font-bold tracking-[0.06em] uppercase max-[1100px]:hidden"
          >
            <span>{{ t("processing.job") }}</span
            ><span>{{ t("processing.provider") }}</span
            ><span>{{ t("processing.progress") }}</span
            ><span>{{ t("processing.status") }}</span>
          </div>
          <div
            v-for="job in application.activeJobs"
            :key="job.id"
            class="processing-row grid grid-cols-[minmax(0,1.2fr)_max-content_minmax(220px,1fr)_max-content] items-center gap-5 border-b border-[var(--divider)] px-[18px] py-[14px] last:border-b-0 max-[1100px]:grid-cols-[minmax(0,1fr)_max-content] max-[1100px]:gap-x-4 max-[1100px]:gap-y-3 max-[700px]:grid-cols-1"
          >
            <div class="processing-row__title flex items-center gap-2.5">
              <CircleDashed
                :size="18"
                class="animate-[spin_1.4s_linear_infinite]"
              />
              <span class="grid min-w-0 gap-1"
                ><strong>{{ sessionTitle(job.session_id) }}</strong
                ><small>{{ jobKind(job) }}</small></span
              >
            </div>
            <StatusPill :tone="isCloudJob(job) ? 'cloud' : 'local'">
              {{
                isCloudJob(job) ? t("processing.openAi") : t("processing.local")
              }}
            </StatusPill>
            <div class="min-w-0">
              <div
                class="progress-label text-ink-muted mb-[7px] flex justify-between gap-2.5 text-xs"
              >
                <strong v-if="progressLabel(job) !== undefined">
                  {{ progressLabel(job) }}%
                </strong>
              </div>
              <AppProgressBar
                :value="progressValue(job)"
                :accessible-label="t('processing.progress')"
              />
              <small
                v-if="estimateDetails(job)"
                class="text-ink-muted mt-1.5 block text-xs"
              >
                {{ estimateDetails(job) }}
              </small>
            </div>
            <div
              class="processing-row__status flex items-center gap-1.5 justify-self-end max-[700px]:justify-self-start"
            >
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
