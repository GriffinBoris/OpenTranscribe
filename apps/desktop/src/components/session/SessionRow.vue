<script setup lang="ts">
import { AlertCircle, FileAudio, FolderInput, Mic2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppCheckbox from "@/components/ui/AppCheckbox.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type { Session } from "@/types/domain";

defineProps<{
  session: Session;
  selectable?: boolean;
  selected?: boolean;
}>();
const emit = defineEmits<{
  move: [];
  "selection-change": [event: Event];
}>();
const { t } = useI18n();

function formatDuration(durationMs: number) {
  const minutes = Math.floor(durationMs / 60_000);
  const seconds = Math.floor((durationMs % 60_000) / 1000);
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
</script>

<template>
  <div
    class="session-row hover:bg-canvas-subtle flex min-w-0 items-center gap-3 border-b border-[var(--divider)] px-[15px] py-[13px] last:border-b-0"
    :class="{ 'bg-canvas-subtle': selected }"
  >
    <AppCheckbox
      v-if="selectable"
      :model-value="Boolean(selected)"
      :accessible-label="t('session.selectMeeting', { title: session.title })"
      @change="emit('selection-change', $event)"
    />
    <RouterLink
      :to="`/sessions/${session.id}`"
      class="session-row__link flex min-w-0 flex-1 items-center gap-3"
      :aria-label="session.title"
    >
      <span
        class="session-row__icon rounded-app-md bg-lichen-soft text-lichen grid size-[35px] shrink-0 place-items-center"
      >
        <FileAudio v-if="session.source === 'import'" :size="18" />
        <Mic2 v-else :size="18" />
      </span>
      <span class="session-row__body grid min-w-0 flex-1 gap-1">
        <strong class="truncate">{{ session.title }}</strong>
        <small class="text-ink-muted block">
          {{ new Date(session.created_at).toLocaleDateString() }} ·
          {{ formatDuration(session.duration_ms) }}
        </small>
      </span>
      <StatusPill
        v-if="session.recovery_state === 'recoverable'"
        tone="warning"
      >
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
    </RouterLink>
    <AppButton
      v-if="!selectable"
      size="small"
      variant="ghost"
      :aria-label="
        t('session.moveProject.recordingLabel', { title: session.title })
      "
      @click="emit('move')"
    >
      <FolderInput :size="15" />{{ t("session.moveProject.action") }}
    </AppButton>
  </div>
</template>
