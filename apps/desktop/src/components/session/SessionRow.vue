<script setup lang="ts">
import { AlertCircle, FileAudio, Mic2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppSelect from "@/components/ui/AppSelect.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type { Session } from "@/types/domain";

defineProps<{
  session: Session;
  projectOptions: Array<{ label: string; value: string }>;
  moving: boolean;
}>();
const emit = defineEmits<{
  "move-to-project": [projectId: string];
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
  >
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
      <StatusPill v-else tone="neutral">
        {{
          session.source === "import"
            ? t("session.imported")
            : t("session.recording")
        }}
      </StatusPill>
    </RouterLink>
    <AppSelect
      class="session-row__project-select w-[var(--control-width-project)] shrink-0 text-sm"
      :model-value="session.project_id ?? ''"
      :options="projectOptions"
      :accessible-label="
        t('session.moveProject.recordingLabel', { title: session.title })
      "
      :placeholder="t('navigation.inbox')"
      :disabled="moving"
      @update:model-value="emit('move-to-project', $event)"
    />
  </div>
</template>
