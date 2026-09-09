import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { native } from "@/core/native";
import {
  emptyDictationStatus,
  isDictationActive,
} from "@/core/dictationStatus";
import type { DictationStatus } from "@/types/domain";

export const useDictationStore = defineStore("dictation", () => {
  const status = ref(emptyDictationStatus());
  const historyRevision = ref(0);
  const isActive = computed(() => isDictationActive(status.value.phase));

  function apply(next: DictationStatus) {
    if (
      next.phase !== status.value.phase &&
      ["completed", "failed"].includes(next.phase)
    ) {
      historyRevision.value += 1;
    }
    status.value = next;
  }

  async function refresh() {
    apply(await native.dictationStatus());
  }
  return { status, historyRevision, isActive, apply, refresh };
});
