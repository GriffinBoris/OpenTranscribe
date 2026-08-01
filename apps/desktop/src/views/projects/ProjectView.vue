<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderOpen } from "@lucide/vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";

import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppSearchInput from "@/components/ui/AppSearchInput.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import SessionRow from "@/components/session/SessionRow.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useMoveSession } from "@/views/application/useMoveSession";

const application = useApplicationStore();
const route = useRoute();
const { t } = useI18n();
const projectId = computed(() => String(route.params.projectId));
const query = ref("");
const { isMoving, moveSession, projectOptions } = useMoveSession();
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
        <div class="tag-row flex flex-wrap gap-1.5">
          <StatusPill v-for="term in project?.glossary" :key="term">{{
            term
          }}</StatusPill>
        </div>
      </div>
    </header>

    <AppSearchInput
      class="mb-4"
      v-model="query"
      :placeholder="t('projects.search')"
    />

    <AppSurface
      class="library-page__sessions grid min-h-0 grid-rows-[minmax(0,1fr)]"
      :padded="false"
    >
      <div class="library-page__scroll min-h-0 overflow-auto">
        <div v-if="sessions.length" class="session-list grid">
          <SessionRow
            v-for="session in sessions"
            :key="session.id"
            :session="session"
            :project-options="projectOptions"
            :moving="isMoving(session.id)"
            @move-to-project="moveSession(session, $event)"
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
        </AppEmptyState>
      </div>
    </AppSurface>
  </div>
</template>
