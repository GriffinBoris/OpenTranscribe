import { computed, ref, watch, type Ref } from "vue";

import type { Session } from "@/types/domain";

export function useSessionSelection(sessions: Ref<Session[]>) {
  const selectedIds = ref(new Set<string>());
  const anchorId = ref<string | null>(null);
  const selectedSessions = computed(() =>
    sessions.value.filter((session) => selectedIds.value.has(session.id)),
  );
  const allSelected = computed(
    () =>
      sessions.value.length > 0 &&
      sessions.value.every((session) => selectedIds.value.has(session.id)),
  );

  watch(
    sessions,
    (visibleSessions) => {
      const visibleIds = new Set(visibleSessions.map((session) => session.id));
      selectedIds.value = new Set(
        [...selectedIds.value].filter((id) => visibleIds.has(id)),
      );

      if (anchorId.value && !visibleIds.has(anchorId.value)) {
        anchorId.value = null;
      }
    },
    { deep: false },
  );

  function select(sessionId: string, event: Event) {
    const originalEvent =
      "originalEvent" in event && event.originalEvent instanceof Event
        ? event.originalEvent
        : event;
    const selectionEvent =
      originalEvent instanceof MouseEvent ||
      originalEvent instanceof KeyboardEvent
        ? originalEvent
        : null;
    const sessionIds = sessions.value.map((session) => session.id);
    const selected = new Set(selectedIds.value);
    const anchorIndex = anchorId.value
      ? sessionIds.indexOf(anchorId.value)
      : -1;
    const sessionIndex = sessionIds.indexOf(sessionId);

    if (selectionEvent?.shiftKey && anchorIndex >= 0 && sessionIndex >= 0) {
      const range = sessionIds.slice(
        Math.min(anchorIndex, sessionIndex),
        Math.max(anchorIndex, sessionIndex) + 1,
      );

      if (selectionEvent?.metaKey || selectionEvent?.ctrlKey) {
        range.forEach((id) => selected.add(id));
      } else {
        selected.clear();
        range.forEach((id) => selected.add(id));
      }
    } else if (selected.has(sessionId)) {
      selected.delete(sessionId);
      anchorId.value = sessionId;
    } else {
      selected.add(sessionId);
      anchorId.value = sessionId;
    }

    selectedIds.value = selected;
  }

  function toggleAll() {
    selectedIds.value = allSelected.value
      ? new Set()
      : new Set(sessions.value.map((session) => session.id));
    anchorId.value = sessions.value.at(-1)?.id ?? null;
  }

  function clear() {
    selectedIds.value = new Set();
    anchorId.value = null;
  }

  return {
    selectedIds,
    selectedSessions,
    allSelected,
    select,
    toggleAll,
    clear,
  };
}
