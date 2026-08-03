import { onBeforeUnmount, ref } from "vue";

const COPIED_FEEDBACK_DURATION_MS = 1_000;

export function useClipboard() {
  const copiedId = ref<string | null>(null);
  let copiedFeedbackTimeout: ReturnType<typeof setTimeout> | null = null;

  async function copy(text: string, id = text) {
    await navigator.clipboard.writeText(text);
    copiedId.value = id;

    if (copiedFeedbackTimeout) {
      clearTimeout(copiedFeedbackTimeout);
    }

    copiedFeedbackTimeout = setTimeout(() => {
      copiedId.value = null;
      copiedFeedbackTimeout = null;
    }, COPIED_FEEDBACK_DURATION_MS);
  }

  onBeforeUnmount(() => {
    if (copiedFeedbackTimeout) {
      clearTimeout(copiedFeedbackTimeout);
    }
  });

  return { copiedId, copy };
}
