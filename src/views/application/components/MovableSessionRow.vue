<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

import SessionRow from "@/components/session/SessionRow.vue";
import type { Session } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

const props = defineProps<{
  session: Session;
}>();

const application = useApplicationStore();
const { t } = useI18n();
const moving = ref(false);
const projectOptions = computed(() => [
  { label: t("navigation.inbox"), value: "" },
  ...application.projects.map((project) => ({
    label: project.name,
    value: project.id,
  })),
]);

async function moveToProject(projectId: string) {
  if (projectId === (props.session.project_id ?? "")) {
    return;
  }

  moving.value = true;
  await application.moveSession(props.session.id, projectId || null);
  moving.value = false;
}
</script>

<template>
  <SessionRow
    :session="session"
    :project-options="projectOptions"
    :moving="moving"
    @move-to-project="moveToProject"
  />
</template>
