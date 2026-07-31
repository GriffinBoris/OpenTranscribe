<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

function startDragging() {
  void getCurrentWindow().startDragging();
}
</script>

<template>
  <div class="window-titlebar" aria-hidden="true">
    <div
      class="window-titlebar__drag-region"
      data-tauri-drag-region
      @mousedown.left.prevent="startDragging"
    >
      <span class="window-titlebar__handle"></span>
    </div>
  </div>
</template>
