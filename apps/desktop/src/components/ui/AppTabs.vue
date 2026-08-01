<script setup lang="ts">
import Tab from "primevue/tab";
import TabList from "primevue/tablist";
import Tabs from "primevue/tabs";

interface AppTabOption {
  label: string;
  value: string;
}

defineProps<{
  modelValue: string;
  options: AppTabOption[];
}>();

defineEmits<{
  "update:modelValue": [value: string];
}>();

const tabListParts = {
  root: "app-tabs__root",
  content: "app-tabs__content overflow-hidden",
  tabList: "app-tabs__list relative flex gap-1",
  activeBar: "app-tabs__active-bar hidden",
  prevButton: "app-tabs__navigator",
  nextButton: "app-tabs__navigator",
};
</script>

<template>
  <Tabs
    unstyled
    class="app-tabs"
    :value="modelValue"
    @update:value="$emit('update:modelValue', String($event))"
  >
    <TabList unstyled :pt="tabListParts">
      <Tab
        v-for="option in options"
        :key="option.value"
        unstyled
        class="app-tabs__tab rounded-app-xs text-ink-muted data-[p-active=true]:bg-canvas-subtle data-[p-active=true]:text-ink relative z-[var(--layer-content)] border-0 bg-transparent px-3 py-[7px]"
        :value="option.value"
      >
        {{ option.label }}
      </Tab>
    </TabList>
  </Tabs>
</template>
