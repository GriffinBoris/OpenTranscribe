<script setup lang="ts">
import { AlertCircle, FileAudio, Mic2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import StatusPill from "@/components/ui/StatusPill.vue";
import type { Session } from "@/types/domain";

defineProps<{
  session: Session;
}>();
const { t } = useI18n();

function formatDuration(durationMs: number) {
  const minutes = Math.floor(durationMs / 60_000);
  const seconds = Math.floor((durationMs % 60_000) / 1000);
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
</script>

<template>
  <RouterLink :to="`/sessions/${session.id}`" class="session-row">
    <span class="session-row__icon">
      <FileAudio v-if="session.source === 'import'" :size="18" />
      <Mic2 v-else :size="18" />
    </span>
    <span class="session-row__body">
      <strong>{{ session.title }}</strong>
      <small>
        {{ new Date(session.created_at).toLocaleDateString() }} ·
        {{ formatDuration(session.duration_ms) }}
      </small>
    </span>
    <StatusPill v-if="session.recovery_state === 'recoverable'" tone="warning">
      <AlertCircle :size="12" />{{ t("session.recovery.needed") }}
    </StatusPill>
    <StatusPill
      v-else-if="session.lifecycle === 'needs_attention'"
      tone="warning"
    >
      <AlertCircle :size="12" />{{ t("session.needsAttention") }}
    </StatusPill>
    <StatusPill
      v-else-if="session.recovery_state === 'recovered'"
      tone="success"
    >
      {{ t("session.recovery.recovered") }}
    </StatusPill>
    <StatusPill v-else tone="neutral">
      {{
        session.source === "import"
          ? t("session.imported")
          : t("session.recording")
      }}
    </StatusPill>
  </RouterLink>
</template>
