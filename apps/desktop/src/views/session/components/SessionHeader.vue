<script setup lang="ts">
import { computed, ref } from "vue";
import {
  Download,
  FolderInput,
  Pencil,
  RotateCcw,
  Search,
  Trash2,
} from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import SessionMoveDialog from "@/components/session/SessionMoveDialog.vue";
import type { ExportFormat, Session, TranscriptRun } from "@/types/domain";
import { formatUsd } from "@/views/application/jobEstimates";
import SessionTranscriptionActions from "@/views/session/components/SessionTranscriptionActions.vue";

const props = defineProps<{
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
  transcriptRun: TranscriptRun | null;
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
const exportOpen = ref(false);
const moveOpen = ref(false);
const cloudCost = computed(() => {
  if (props.transcriptRun?.source !== "open_ai") {
    return null;
  }

  return props.transcriptRun.approximate_cost_usd === null
    ? t("processing.costUnavailable")
    : t("processing.totalEstimatedCost", {
        cost: formatUsd(props.transcriptRun.approximate_cost_usd),
      });
});
const projectLabel = computed(
  () =>
    props.projectOptions.find(
      (option) => option.value === (props.session?.project_id ?? ""),
    )?.label ?? t("navigation.inbox"),
);
</script>

<template>
  <header
    class="session-header grid gap-4 border-b border-[var(--divider)] px-6 py-[var(--space-4-5)]"
  >
    <div class="flex min-w-0 items-center gap-3 overflow-hidden">
      <h1
        class="min-w-0 flex-1 overflow-hidden text-3xl leading-[var(--line-height-heading)] font-bold tracking-[-0.02em] text-ellipsis whitespace-nowrap"
      >
        {{ session?.title ?? t("session.untitledRecording") }}
      </h1>
      <div class="flex shrink-0 items-center gap-1">
        <AppButton
          class="shrink-0 p-[var(--space-1-5)]"
          size="small"
          variant="ghost"
          :aria-label="t('session.rename.action')"
          @click="emit('rename')"
        >
          <Pencil :size="15" />
        </AppButton>
        <AppButton
          class="shrink-0"
          size="small"
          variant="ghost"
          :aria-label="t('session.moveProject.action')"
          :disabled="recording || movingSession"
          @click="moveOpen = true"
        >
          <FolderInput :size="16" />
        </AppButton>
        <AppButton
          class="shrink-0"
          size="small"
          variant="ghost"
          :aria-label="t('session.export')"
          @click="exportOpen = true"
        >
          <Download :size="16" />
        </AppButton>
        <span
          v-if="!recording"
          class="mx-1 h-5 w-px bg-[var(--divider)]"
          aria-hidden="true"
        ></span>
        <AppButton
          v-if="!recording"
          class="shrink-0"
          size="small"
          variant="danger"
          :aria-label="t('session.trash.action')"
          @click="emit('trash')"
        >
          <Trash2 :size="16" />
        </AppButton>
      </div>
    </div>
    <div
      class="session-header__content flex min-w-0 items-center gap-4 max-[900px]:flex-col max-[900px]:items-stretch max-[900px]:gap-3"
    >
      <div
        class="session-header__metadata text-ink-muted flex min-h-[var(--control-height-medium)] min-w-0 flex-1 flex-wrap items-center gap-2 text-sm"
      >
        <span class="whitespace-nowrap">{{
          session
            ? new Date(session.created_at).toLocaleString()
            : t("session.justNow")
        }}</span>
        <span>·</span>
        <span class="text-ink-faint whitespace-nowrap">{{ projectLabel }}</span>
        <StatusPill
          v-if="recording || recoverable"
          :tone="recording ? 'recording' : 'warning'"
        >
          {{
            recording ? t("session.recording") : t("session.recovery.needed")
          }}
        </StatusPill>
        <span
          v-if="cloudCost"
          class="text-ink-muted text-xs whitespace-nowrap"
          :title="t('processing.pricingDisclaimer')"
        >
          {{ cloudCost }}
        </span>
      </div>
      <div
        class="session-header__controls ml-auto flex min-w-0 shrink-0 flex-wrap items-center justify-end gap-2 max-[900px]:ml-0 max-[900px]:justify-start"
      >
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
        <SessionTranscriptionActions
          class="max-w-full"
          :can-transcribe="canTranscribe"
          :has-transcript="hasTranscript"
          :session-id="sessionId"
        />
      </div>
    </div>
  </header>

  <SessionMoveDialog
    v-model:open="moveOpen"
    :session-count="1"
    :project-options="projectOptions"
    :moving="movingSession"
    @move="emit('move-to-project', $event)"
  />

  <AppDialog
    :open="exportOpen"
    :title="t('session.export')"
    @update:open="exportOpen = $event"
  >
    <div class="grid gap-3">
      <p v-if="!hasTranscript" class="text-ink-muted m-0 text-sm">
        {{ t("session.exportUnavailable") }}
      </p>
      <label class="grid gap-2">
        <span class="text-sm font-semibold">{{
          t("session.exportFormat")
        }}</span>
        <AppSelect
          class="w-full"
          :model-value="exportFormat"
          :options="exportOptions"
          :accessible-label="t('session.exportFormat')"
          @update:model-value="emit('update:export-format', $event)"
        />
      </label>
    </div>
    <template #footer>
      <AppButton variant="secondary" @click="exportOpen = false">
        {{ t("session.cancel") }}
      </AppButton>
      <AppButton
        :disabled="!hasTranscript || isExporting"
        :loading="isExporting"
        @click="
          exportOpen = false;
          emit('export');
        "
      >
        <Download :size="15" />
        {{ t("session.export") }}
      </AppButton>
    </template>
  </AppDialog>
</template>
