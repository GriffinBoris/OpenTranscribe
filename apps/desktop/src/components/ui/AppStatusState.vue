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
    class="text-ink-muted grid h-full min-h-[220px] place-content-center justify-items-center gap-2 text-center"
    :class="{ 'text-accent': error }"
    :role="error ? 'alert' : 'status'"
  >
    <LoaderCircle
      v-if="loading"
      class="animate-[spin_1.4s_linear_infinite]"
      :size="22"
      aria-hidden="true"
    />
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
