<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderOpen } from "@lucide/vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";

import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppSearchInput from "@/components/ui/AppSearchInput.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import MovableSessionRow from "@/views/application/components/MovableSessionRow.vue";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const route = useRoute();
const { t } = useI18n();
const projectId = computed(() => String(route.params.projectId));
const query = ref("");
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
  <div class="page library-page">
    <header class="page-header page-header--compact">
      <div>
        <p class="eyebrow">{{ t("projects.label") }}</p>
        <h1>{{ project?.name ?? t("projects.fallbackTitle") }}</h1>
        <div class="tag-row">
          <StatusPill v-for="term in project?.glossary" :key="term">{{
            term
          }}</StatusPill>
        </div>
      </div>
    </header>

    <AppSearchInput v-model="query" :placeholder="t('projects.search')" />

    <AppSurface class="library-page__sessions" :padded="false">
      <div class="library-page__scroll">
        <div v-if="sessions.length" class="session-list">
          <MovableSessionRow
            v-for="session in sessions"
            :key="session.id"
            :session="session"
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
