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
  <header class="session-header">
    <div class="session-title-row">
      <h1 class="session-title">
        {{ session?.title ?? t("session.untitledRecording") }}
      </h1>
      <AppButton
        class="session-title__edit"
        size="small"
        variant="ghost"
        :aria-label="t('session.rename.action')"
        @click="emit('rename')"
      >
        <Pencil :size="15" />
      </AppButton>
    </div>
    <div class="session-header__controls">
      <div class="session-meta">
        <span class="session-meta__date">{{
          session
            ? new Date(session.created_at).toLocaleString()
            : t("session.justNow")
        }}</span>
        <span>·</span>
        <AppSelect
          class="session-project-select"
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
      <div class="session-header__actions">
        <AppButton
          v-if="recoverable"
          variant="primary"
          :loading="recovering"
          :disabled="recovering"
          @click="emit('recover')"
        >
          <RotateCcw :size="16" />
          {{ t("session.recovery.action") }}
        </AppButton>
        <AppButton
          variant="ghost"
          :aria-label="t('session.search.title')"
          @click="emit('search')"
        >
          <Search :size="16" />
          {{ t("session.search.action") }}
        </AppButton>
        <AppButton
          v-if="!recording"
          variant="ghost"
          :aria-label="t('session.trash.action')"
          @click="emit('trash')"
        >
          <Trash2 :size="16" />
        </AppButton>
        <SessionTranscriptionActions
          :can-transcribe="canTranscribe"
          :session-id="sessionId"
        />
        <div class="export-actions">
          <AppSelect
            :model-value="exportFormat"
            :options="exportOptions"
            :accessible-label="t('session.exportFormat')"
            @update:model-value="emit('update:export-format', $event)"
          />
          <AppButton
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
