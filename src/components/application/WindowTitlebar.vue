<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

function startDragging() {
  void getCurrentWindow().startDragging();
}
</script>

<template>
  <div
    class="window-titlebar"
    data-tauri-drag-region
    aria-hidden="true"
    @mousedown.left.prevent="startDragging"
  >
    <span class="window-titlebar__handle" data-tauri-drag-region></span>
  </div>
</template>
