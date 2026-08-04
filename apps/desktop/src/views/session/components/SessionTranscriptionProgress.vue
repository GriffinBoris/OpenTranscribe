<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import AppButton from "@/components/ui/AppButton.vue";
import { jobEstimateDetails } from "@/views/application/jobEstimates";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useSessionStore } from "@/views/session/sessionStore";

const props = defineProps<{
  sessionId: string;
}>();

const sessionStore = useSessionStore();
const application = useApplicationStore();
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
const progressPercent = computed(() =>
  progressValue.value === undefined ? null : Math.round(progressValue.value),
);
const isCanceling = ref(false);
const streamedTranscript = computed(
  () => application.transcriptionPreviews[props.sessionId] ?? "",
);
const estimateDetails = computed(() => {
  const job = activeJob.value;

  if (!job) {
    return [];
  }

  return jobEstimateDetails(job, (key, parameters) =>
    parameters ? t(key, parameters) : t(key),
  );
});

async function cancelTranscription() {
  if (!activeJob.value) {
    return;
  }

  isCanceling.value = true;

  try {
    await application.cancelJob(activeJob.value.id);
  } finally {
    isCanceling.value = false;
  }
}
</script>

<template>
  <div
    v-if="activeJob"
    class="bg-canvas-subtle text-ink-muted grid grid-cols-[minmax(160px,320px)_auto_auto] items-center gap-3 border-b border-[var(--divider)] px-6 py-[9px] text-sm max-[700px]:grid-cols-1"
    role="status"
    aria-live="polite"
  >
    <AppProgressBar
      :value="progressValue"
      :accessible-label="t('processing.progress')"
    />
    <span class="grid gap-0.5">
      <strong v-if="progressPercent !== null">{{ progressPercent }}%</strong>
      <span v-else>{{ t("processing.waiting") }}</span>
      <small v-if="estimateDetails.length" class="text-xs">
        {{ estimateDetails.join(" · ") }}
      </small>
    </span>
    <AppButton
      variant="ghost"
      size="small"
      :disabled="isCanceling"
      @click="cancelTranscription"
    >
      {{ t("processing.cancel") }}
    </AppButton>
    <p
      v-if="streamedTranscript"
      class="text-ink col-span-full max-h-20 overflow-auto border-t border-[var(--divider)] pt-2 text-sm leading-relaxed whitespace-pre-wrap"
    >
      {{ streamedTranscript }}
    </p>
  </div>
</template>
