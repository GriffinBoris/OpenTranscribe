<script setup lang="ts">
import { LoaderCircle } from "@lucide/vue";

import AppButton from "@/components/ui/AppButton.vue";

withDefaults(
  defineProps<{
    title: string;
    message?: string;
    loading?: boolean;
    error?: boolean;
    retryLabel?: string;
  }>(),
  {
    message: "",
    loading: false,
    error: false,
    retryLabel: "",
  },
);

const emit = defineEmits<{
  retry: [];
}>();
</script>

<template>
  <div
    class="shell-state"
    :class="{ 'shell-state--error': error }"
    :role="error ? 'alert' : 'status'"
  >
    <LoaderCircle v-if="loading" class="spin" :size="22" aria-hidden="true" />
    <strong>{{ title }}</strong>
    <span v-if="message">{{ message }}</span>
    <AppButton
      v-if="retryLabel"
      size="small"
      variant="secondary"
      @click="emit('retry')"
    >
      {{ retryLabel }}
    </AppButton>
  </div>
</template>
