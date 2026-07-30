<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderOpen, Search } from "@lucide/vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";

import SessionRow from "@/components/session/SessionRow.vue";
import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
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
  <div class="page">
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

    <div class="search-field">
      <Search :size="16" />
      <AppInputText
        v-model="query"
        type="search"
        :placeholder="t('projects.search')"
      />
    </div>

    <AppSurface :padded="false">
      <div v-if="sessions.length" class="session-list">
        <SessionRow
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
    </AppSurface>
  </div>
</template>
