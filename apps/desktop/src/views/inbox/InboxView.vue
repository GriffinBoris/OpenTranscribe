<script setup lang="ts">
import { computed, ref } from "vue";
import { Inbox } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import AppButton from "@/components/ui/AppButton.vue";
import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import SessionRow from "@/components/session/SessionRow.vue";
import SessionListToolbar from "@/components/session/SessionListToolbar.vue";
import AppCheckbox from "@/components/ui/AppCheckbox.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useMoveSession } from "@/views/application/useMoveSession";
import { useSessionSelection } from "@/views/application/useSessionSelection";

const application = useApplicationStore();
const { t } = useI18n();
const router = useRouter();
const query = ref("");
const { isMoving, moveSessions, projectOptions } = useMoveSession();
const sessions = computed(() => {
  const normalizedQuery = query.value.trim().toLocaleLowerCase();

  return application.recentSessions.filter(
    (session) =>
      session.project_id === null &&
      (!normalizedQuery ||
        session.title.toLocaleLowerCase().includes(normalizedQuery)),
  );
});
const inboxHasSessions = computed(() =>
  application.recentSessions.some((session) => session.project_id === null),
);
const { selectedIds, selectedSessions, allSelected, select, toggleAll, clear } =
  useSessionSelection(sessions);
const isMovingSelected = computed(() =>
  selectedSessions.value.some((session) => isMoving(session.id)),
);

async function importMedia() {
  const session = await application.importMedia();

  if (session) {
    await router.push(`/sessions/${session.id}`);
  }
}

async function moveSelectedSessions(projectId: string) {
  await moveSessions(selectedSessions.value, projectId);
  clear();
}
</script>

<template>
  <div
    class="page library-page grid h-full min-h-0 w-full grid-rows-[auto_auto_minmax(0,1fr)] overflow-hidden px-[var(--layout-page-gutter)] pt-[var(--space-13)] pb-[var(--space-14)]"
  >
    <header
      class="page-header page-header--compact mb-[var(--space-5-5)] flex items-start justify-between gap-6"
    >
      <h1
        class="text-display m-0 max-w-[730px] leading-[var(--line-height-tight)] font-bold tracking-[-0.035em]"
      >
        {{ t("library.inbox") }}
      </h1>
      <AppButton
        variant="secondary"
        :disabled="application.isImporting"
        :loading="application.isImporting"
        @click="importMedia"
      >
        {{ t("library.importMedia") }}
      </AppButton>
    </header>

    <SessionListToolbar
      v-model="query"
      :search-placeholder="t('library.search')"
      :selected-count="selectedIds.size"
      :project-options="projectOptions"
      :moving="isMovingSelected"
      @move="moveSelectedSessions"
    />

    <AppSurface
      class="library-page__sessions grid min-h-0 grid-rows-[auto_minmax(0,1fr)]"
      :padded="false"
    >
      <div
        v-if="sessions.length"
        class="flex h-[45px] items-center border-b border-[var(--divider)] px-[15px]"
      >
        <AppCheckbox
          :model-value="allSelected"
          :accessible-label="t('session.selectAllMeetings')"
          @change="toggleAll"
        />
      </div>
      <div class="library-page__scroll min-h-0 overflow-auto">
        <div v-if="sessions.length" class="session-list grid">
          <SessionRow
            v-for="session in sessions"
            :key="session.id"
            :session="session"
            selectable
            :selected="selectedIds.has(session.id)"
            @selection-change="select(session.id, $event)"
          />
        </div>
        <AppEmptyState
          v-else
          :title="
            inboxHasSessions && query.trim()
              ? t('library.noMatches')
              : t('library.empty')
          "
          :message="
            inboxHasSessions && query.trim()
              ? t('library.noMatchesDescription')
              : t('library.emptyDescription')
          "
        >
          <template #icon><Inbox :size="21" /></template>
        </AppEmptyState>
      </div>
    </AppSurface>
  </div>
</template>
