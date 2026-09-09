<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderOpen } from "@lucide/vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import SessionRow from "@/components/session/SessionRow.vue";
import SessionListToolbar from "@/components/session/SessionListToolbar.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useMoveSession } from "@/views/application/useMoveSession";
import { useSessionSelection } from "@/views/application/useSessionSelection";

const application = useApplicationStore();
const route = useRoute();
const { t } = useI18n();
const projectId = computed(() => String(route.params.projectId));
const query = ref("");
const { isMoving, moveSessions, projectOptions } = useMoveSession();
const project = computed(() =>
  application.projects.find((item) => item.id === projectId.value),
);
const sessions = computed(() => {
  const normalizedQuery = query.value.trim().toLocaleLowerCase();

  return application.recentSessions.filter(
    (session) =>
      session.project_id === projectId.value &&
      (!normalizedQuery ||
        session.title.toLocaleLowerCase().includes(normalizedQuery)),
  );
});
const projectHasSessions = computed(() =>
  application.recentSessions.some(
    (session) => session.project_id === projectId.value,
  ),
);
const { selectedIds, selectedSessions, allSelected, select, toggleAll, clear } =
  useSessionSelection(sessions);
const isMovingSelected = computed(() =>
  selectedSessions.value.some((session) => isMoving(session.id)),
);

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
      <div>
        <p
          class="eyebrow text-lichen mb-1.5 text-xs font-black tracking-[0.1em] uppercase"
        >
          {{ t("projects.label") }}
        </p>
        <h1
          class="text-display m-0 max-w-[730px] leading-[var(--line-height-tight)] font-bold tracking-[-0.035em]"
        >
          {{ project?.name ?? t("projects.fallbackTitle") }}
        </h1>
        <div
          v-if="project?.glossary.length"
          class="tag-row mt-3 flex flex-wrap gap-1.5"
        >
          <StatusPill v-for="term in project?.glossary" :key="term">{{
            term
          }}</StatusPill>
        </div>
      </div>
    </header>

    <SessionListToolbar
      v-model="query"
      :search-placeholder="t('projects.search')"
      :selected-count="selectedIds.size"
      :result-count="sessions.length"
      :all-selected="allSelected"
      @toggle-all="toggleAll"
      :project-options="projectOptions"
      :moving="isMovingSelected"
      @move="moveSelectedSessions"
    />

    <section
      class="library-page__sessions grid min-h-0 grid-rows-[minmax(0,1fr)]"
    >
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
            projectHasSessions && query.trim()
              ? t('projects.noMatches')
              : t('projects.empty')
          "
          :message="
            projectHasSessions && query.trim()
              ? t('projects.noMatchesDescription')
              : t('projects.emptyDescription')
          "
        >
          <template #icon><FolderOpen :size="21" /></template>
          <AppButton
            v-if="query.trim()"
            variant="secondary"
            @click="query = ''"
            >{{ t("library.clearSearch") }}</AppButton
          >
        </AppEmptyState>
      </div>
    </section>
  </div>
</template>
