<script setup lang="ts">
import { computed, onMounted } from "vue";

import ApplicationShellView from "@/views/application/ApplicationShellView.vue";
import DictationPanelView from "@/views/dictation/DictationPanelView.vue";
import { subscribeToApplicationEvents } from "@/views/application/applicationEvents";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const isDictationPanel = computed(() =>
  new URLSearchParams(window.location.search).has("dictation"),
);

if (isDictationPanel.value) {
  document.documentElement.dataset.window = "dictation";
}

onMounted(async () => {
  if (isDictationPanel.value) {
    return;
  }

  await application.bootstrap();

  try {
    await subscribeToApplicationEvents();
  } catch (reason) {
    application.operationError =
      reason instanceof Error ? reason.message : String(reason);
  }
});
</script>

<template>
  <DictationPanelView v-if="isDictationPanel" />
  <ApplicationShellView v-else />
</template>
