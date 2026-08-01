<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

function startDragging() {
  void getCurrentWindow().startDragging();
}
</script>

<template>
  <div
    class="window-titlebar relative z-[var(--layer-titlebar)] col-span-full row-start-1 grid h-[var(--layout-titlebar-height)] w-full place-items-center select-none"
    aria-hidden="true"
  >
    <div
      class="window-titlebar__drag-region group absolute inset-y-0 right-[120px] left-[88px] grid cursor-grab place-items-center active:cursor-grabbing"
      data-tauri-drag-region
      @mousedown.left.prevent="startDragging"
    >
      <span
        class="window-titlebar__handle h-1 w-[42px] rounded-full bg-[color-mix(in_srgb,var(--text-muted)_35%,transparent)] transition-colors duration-[var(--duration-fast)] ease-[var(--easing-standard)] group-hover:bg-[color-mix(in_srgb,var(--text-muted)_60%,transparent)]"
      ></span>
    </div>
  </div>
</template>
