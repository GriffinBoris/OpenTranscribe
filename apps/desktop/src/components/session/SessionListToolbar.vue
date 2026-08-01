<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderInput } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import SessionMoveDialog from "@/components/session/SessionMoveDialog.vue";
import AppButton from "@/components/ui/AppButton.vue";
import AppSearchInput from "@/components/ui/AppSearchInput.vue";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    searchPlaceholder: string;
    selectedCount: number;
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
  <div class="library-toolbar mb-4 flex min-w-0 flex-wrap items-center gap-3">
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

  <SessionMoveDialog
    v-model:open="moveOpen"
    :session-count="selectedCount"
    :project-options="projectOptions"
    :moving="moving"
    @move="emit('move', $event)"
  />
</template>
