<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

const props = withDefaults(
  defineProps<{
    samples: number[];
    active?: boolean;
    dense?: boolean;
    loading?: boolean;
    progress?: number | null;
    minimumHeight?: number;
    maximumHeight?: number;
  }>(),
  {
    active: true,
    dense: false,
    loading: false,
    progress: null,
    minimumHeight: 4,
    maximumHeight: 42,
  },
);

const waveformElement = ref<HTMLElement>();
const waveformWidth = ref(0);

const displayedSamples = computed(() => {
  if (!props.dense || !props.samples.length || waveformWidth.value === 0) {
    return props.samples;
  }

  const barCount = Math.max(1, Math.floor(waveformWidth.value / 5));

  return Array.from({ length: barCount }, (_, index) => {
    const start = Math.floor((index * props.samples.length) / barCount);
    const end = Math.max(
      start + 1,
      Math.floor(((index + 1) * props.samples.length) / barCount),
    );
    const samples = props.samples.slice(start, end);

    return (
      samples.reduce((total, sample) => total + sample, 0) / samples.length
    );
  });
});

let resizeObserver: ResizeObserver | undefined;

onMounted(() => {
  const element = waveformElement.value;
  if (!element) {
    return;
  }

  resizeObserver = new ResizeObserver(([entry]) => {
    waveformWidth.value = entry.contentRect.width;
  });
  resizeObserver.observe(element);
});

onBeforeUnmount(() => resizeObserver?.disconnect());

function barHeight(sample: number) {
  return Math.round(
    props.minimumHeight +
      Math.min(1, Math.max(0, sample)) *
        (props.maximumHeight - props.minimumHeight),
  );
}

function isPlayed(index: number) {
  if (props.progress === null) {
    return false;
  }

  return (
    index / Math.max(1, displayedSamples.value.length - 1) <= props.progress
  );
}
</script>

<template>
  <div
    ref="waveformElement"
    class="flex size-full min-w-0 items-center"
    :class="
      dense ? 'justify-start gap-1 overflow-hidden' : 'justify-center gap-1'
    "
    aria-hidden="true"
  >
    <span
      v-for="(sample, index) in displayedSamples"
      :key="index"
      class="rounded-full motion-reduce:transition-none"
      :class="[
        dense ? 'w-px shrink-0' : 'max-w-[3px] min-w-px flex-1',
        active || loading ? 'opacity-100' : 'opacity-30',
        loading
          ? 'origin-center animate-[waveform-pulse_1100ms_ease-in-out_infinite]'
          : 'transition-[height,opacity,background-color] duration-75 ease-linear motion-reduce:transition-none',
        progress === null
          ? 'bg-lichen'
          : isPlayed(index)
            ? 'bg-accent'
            : 'bg-line-strong',
      ]"
      :style="{
        height: `${barHeight(sample)}px`,
        animationDelay: loading ? `${(index % 24) * -45}ms` : undefined,
      }"
    />
  </div>
</template>
