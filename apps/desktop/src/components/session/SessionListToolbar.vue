<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderInput } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import SessionMoveDialog from "@/components/session/SessionMoveDialog.vue";
import AppButton from "@/components/ui/AppButton.vue";
import AppCheckbox from "@/components/ui/AppCheckbox.vue";
import AppSearchInput from "@/components/ui/AppSearchInput.vue";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    searchPlaceholder: string;
    selectedCount: number;
    resultCount: number;
    allSelected: boolean;
    projectOptions: Array<{ label: string; value: string }>;
    moving?: boolean;
  }>(),
  {
    moving: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string | undefined];
  move: [projectId: string];
  "toggle-all": [];
}>();

const { t } = useI18n();
const moveOpen = ref(false);
const moveLabel = computed(() =>
  props.selectedCount
    ? t("session.moveProject.actionCount", { count: props.selectedCount })
    : t("session.moveProject.action"),
);
</script>

<template>
  <div class="library-toolbar grid min-w-0 gap-3">
    <div class="flex min-w-0 flex-wrap items-center gap-3">
      <AppSearchInput
        class="min-w-[220px] flex-1"
        full-width
        :model-value="modelValue"
        :placeholder="searchPlaceholder"
        @update:model-value="emit('update:modelValue', $event)"
      />
      <AppButton
        class="shrink-0"
        variant="secondary"
        :disabled="selectedCount === 0 || moving"
        @click="moveOpen = true"
      >
        <FolderInput :size="16" />
        {{ moveLabel }}
      </AppButton>
    </div>
    <div
      v-if="resultCount"
      class="text-ink-muted flex min-h-10 items-center gap-3 border-b border-[var(--divider)] px-[15px] text-sm"
    >
      <AppCheckbox
        :model-value="allSelected"
        :indeterminate="selectedCount > 0 && !allSelected"
        :accessible-label="t('session.selectAllMeetings')"
        @change="emit('toggle-all')"
      />
      <span>{{ t("library.sessionCount", resultCount) }}</span>
      <span v-if="selectedCount" class="ml-auto" role="status">{{
        t("library.selectedCount", { count: selectedCount })
      }}</span>
    </div>
  </div>

  <SessionMoveDialog
    v-model:open="moveOpen"
    :session-count="selectedCount"
    :project-options="projectOptions"
    :moving="moving"
    @move="emit('move', $event)"
  />
</template>
