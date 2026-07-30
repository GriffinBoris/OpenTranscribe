<script setup lang="ts">
import { onMounted } from "vue";

import ApplicationShellView from "@/views/application/ApplicationShellView.vue";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();

onMounted(() => application.bootstrap());
</script>

<template>
  <ApplicationShellView />
</template>
