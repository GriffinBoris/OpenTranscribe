<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { FolderInput } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppSelect from "@/components/ui/AppSelect.vue";

const props = defineProps<{
  open: boolean;
  sessionCount: number;
  projectOptions: Array<{ label: string; value: string }>;
  moving?: boolean;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  move: [projectId: string];
}>();

const { t } = useI18n();
const projectId = ref("");
const title = computed(() =>
  t(
    props.sessionCount === 1
      ? "session.moveProject.dialogTitleSingle"
      : "session.moveProject.dialogTitleMultiple",
    { count: props.sessionCount },
  ),
);

watch(
  () => props.open,
  (open) => {
    if (open) {
      projectId.value = "";
    }
  },
);

function move() {
  emit("update:open", false);
  emit("move", projectId.value);
}
</script>

<template>
  <AppDialog
    :open="open"
    :title="title"
    @update:open="emit('update:open', $event)"
  >
    <div class="grid gap-4">
      <p class="text-ink-muted m-0 text-sm">
        {{
          t(
            sessionCount === 1
              ? "session.moveProject.dialogDescriptionSingle"
              : "session.moveProject.dialogDescriptionMultiple",
            { count: sessionCount },
          )
        }}
      </p>
      <label class="grid gap-2">
        <span class="text-sm font-semibold">{{
          t("session.moveProject.destination")
        }}</span>
        <AppSelect
          class="w-full"
          :model-value="projectId"
          :options="projectOptions"
          :accessible-label="t('session.moveProject.destination')"
          :placeholder="t('navigation.inbox')"
          :disabled="moving"
          @update:model-value="projectId = $event"
        />
      </label>
    </div>
    <template #footer>
      <AppButton
        variant="secondary"
        :disabled="moving"
        @click="emit('update:open', false)"
      >
        {{ t("session.cancel") }}
      </AppButton>
      <AppButton :loading="moving" :disabled="moving" @click="move">
        <FolderInput :size="15" />
        {{ t("session.moveProject.confirm") }}
      </AppButton>
    </template>
  </AppDialog>
</template>
