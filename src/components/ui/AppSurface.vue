<script setup lang="ts">
withDefaults(
  defineProps<{
    interactive?: boolean;
    padded?: boolean;
  }>(),
  {
    interactive: false,
    padded: true,
  },
);
</script>

<template>
  <section
    class="app-surface"
    :class="{
      'app-surface--interactive': interactive,
      'app-surface--padded': padded,
    }"
  >
    <slot />
  </section>
</template>
