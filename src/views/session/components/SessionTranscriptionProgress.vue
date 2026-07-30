<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import AppProgressBar from "@/components/ui/AppProgressBar.vue";
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
</script>

<template>
  <div
    v-if="activeJob"
    class="session-progress"
    role="status"
    aria-live="polite"
  >
    <AppProgressBar
      :value="progressValue"
      :accessible-label="t('processing.progress')"
    />
    <span>{{ activeJob.progress?.message ?? t("processing.waiting") }}</span>
  </div>
</template>
