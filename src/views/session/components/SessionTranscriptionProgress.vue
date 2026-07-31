<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import { jobEstimateDetails } from "@/views/application/jobEstimates";
import { useSessionStore } from "@/views/session/sessionStore";

const props = defineProps<{
  sessionId: string;
}>();

const sessionStore = useSessionStore();
const { t } = useI18n();
const activeJob = computed(() =>
  sessionStore.transcriptionJobForSession(props.sessionId),
);
const progressValue = computed(() => {
  const progress = activeJob.value?.progress;

  if (!progress?.total_units) {
    return undefined;
  }

  return (progress.completed_units / progress.total_units) * 100;
});
const estimateDetails = computed(() => {
  const job = activeJob.value;

  if (!job) {
    return [];
  }

  return jobEstimateDetails(job, (key, parameters) =>
    parameters ? t(key, parameters) : t(key),
  );
});
</script>

<template>
  <div
    v-if="activeJob"
    class="bg-canvas-subtle text-ink-muted grid grid-cols-[minmax(160px,320px)_auto] items-center gap-3 border-b border-[var(--divider)] px-6 py-[9px] text-sm max-[700px]:grid-cols-1"
    role="status"
    aria-live="polite"
  >
    <AppProgressBar
      :value="progressValue"
      :accessible-label="t('processing.progress')"
    />
    <span class="grid gap-0.5">
      <span>{{ activeJob.progress?.message ?? t("processing.waiting") }}</span>
      <small v-if="estimateDetails.length" class="text-xs">
        {{ estimateDetails.join(" · ") }}
      </small>
    </span>
  </div>
</template>
