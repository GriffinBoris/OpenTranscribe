<script setup lang="ts">
import { onMounted } from "vue";

import ApplicationShellView from "@/views/application/ApplicationShellView.vue";
import { subscribeToApplicationEvents } from "@/views/application/applicationEvents";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();

onMounted(async () => {
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
  <ApplicationShellView />
</template>
