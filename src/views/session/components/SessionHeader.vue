<script setup lang="ts">
import { Download, Pencil, RotateCcw, Search, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import type { ExportFormat, Session } from "@/types/domain";
import SessionTranscriptionActions from "@/views/session/components/SessionTranscriptionActions.vue";

defineProps<{
  session: Session | null;
  sessionId: string;
  projectOptions: Array<{ label: string; value: string }>;
  recording: boolean;
  movingSession: boolean;
  canTranscribe: boolean;
  hasTranscript: boolean;
  exportFormat: ExportFormat;
  exportOptions: Array<{ label: string; value: string }>;
  isExporting: boolean;
  recoverable: boolean;
  recovering: boolean;
}>();

const emit = defineEmits<{
  recover: [];
  rename: [];
  search: [];
  trash: [];
  export: [];
  "move-to-project": [projectId: string];
  "update:export-format": [value: string];
}>();

const { t } = useI18n();
</script>

<template>
  <header
    class="grid min-h-[82px] gap-3 overflow-hidden border-b border-[var(--divider)] px-6 py-[var(--space-4-5)]"
  >
    <div class="flex min-w-0 items-center gap-1 overflow-hidden">
      <div class="flex min-w-0 flex-1 items-center gap-1">
        <h1
          class="min-w-0 flex-1 overflow-hidden text-3xl leading-[var(--line-height-heading)] font-bold tracking-[-0.02em] text-ellipsis whitespace-nowrap"
        >
          {{ session?.title ?? t("session.untitledRecording") }}
        </h1>
        <AppButton
          class="shrink-0 p-[var(--space-1-5)]"
          size="small"
          variant="ghost"
          :aria-label="t('session.rename.action')"
          @click="emit('rename')"
        >
          <Pencil :size="15" />
        </AppButton>
      </div>
      <AppButton
        v-if="!recording"
        class="shrink-0"
        size="small"
        variant="ghost"
        :aria-label="t('session.trash.action')"
        @click="emit('trash')"
      >
        <Trash2 :size="16" />
      </AppButton>
    </div>
    <div
      class="flex min-w-0 items-center justify-between gap-[var(--space-4-5)] overflow-hidden max-[1200px]:flex-col max-[1200px]:items-stretch"
    >
      <div
        class="text-ink-muted flex min-h-[var(--control-height-medium)] min-w-0 items-center gap-2 text-sm max-[720px]:flex-wrap"
      >
        <span class="whitespace-nowrap">{{
          session
            ? new Date(session.created_at).toLocaleString()
            : t("session.justNow")
        }}</span>
        <span>·</span>
        <AppSelect
          class="w-[var(--control-width-project)] shrink-0"
          :model-value="session?.project_id ?? ''"
          :options="projectOptions"
          :accessible-label="t('session.moveProject.label')"
          :placeholder="t('navigation.inbox')"
          :disabled="recording || movingSession"
          @update:model-value="emit('move-to-project', $event)"
        />
        <StatusPill
          v-if="recording || recoverable"
          :tone="recording ? 'recording' : 'warning'"
        >
          {{
            recording ? t("session.recording") : t("session.recovery.needed")
          }}
        </StatusPill>
      </div>
      <div
        class="flex min-w-0 shrink-0 items-center justify-end gap-2 max-[1200px]:grid max-[1200px]:w-full max-[1200px]:grid-cols-2 max-[1200px]:items-stretch max-[720px]:grid-cols-1"
      >
        <AppButton
          v-if="recoverable"
          class="max-[1200px]:w-full"
          variant="primary"
          :loading="recovering"
          :disabled="recovering"
          @click="emit('recover')"
        >
          <RotateCcw :size="16" />
          {{ t("session.recovery.action") }}
        </AppButton>
        <AppButton
          class="max-[1200px]:w-full"
          variant="ghost"
          :aria-label="t('session.search.title')"
          @click="emit('search')"
        >
          <Search :size="16" />
          {{ t("session.search.action") }}
        </AppButton>
        <SessionTranscriptionActions
          class="max-[1200px]:w-full"
          :can-transcribe="canTranscribe"
          :session-id="sessionId"
        />
        <div
          class="flex items-center gap-2 max-[1200px]:w-full max-[720px]:grid max-[720px]:grid-cols-1"
        >
          <AppSelect
            class="w-32 min-w-0 flex-1"
            :model-value="exportFormat"
            :options="exportOptions"
            :accessible-label="t('session.exportFormat')"
            @update:model-value="emit('update:export-format', $event)"
          />
          <AppButton
            class="max-[720px]:w-full"
            variant="secondary"
            :disabled="!hasTranscript || isExporting"
            :loading="isExporting"
            @click="emit('export')"
          >
            <Download :size="15" />
            {{ isExporting ? t("session.exporting") : t("session.export") }}
          </AppButton>
        </div>
      </div>
    </div>
  </header>
</template>
