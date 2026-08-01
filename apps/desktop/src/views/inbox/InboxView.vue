<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import AppButton from "@/components/ui/AppButton.vue";
import AppSearchInput from "@/components/ui/AppSearchInput.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import SessionRow from "@/components/session/SessionRow.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useMoveSession } from "@/views/application/useMoveSession";

const application = useApplicationStore();
const { t } = useI18n();
const router = useRouter();
const query = ref("");
const { isMoving, moveSession, projectOptions } = useMoveSession();
const sessions = computed(() => {
  const normalizedQuery = query.value.trim().toLocaleLowerCase();

  return application.recentSessions.filter(
    (session) =>
      session.project_id === null &&
      (!normalizedQuery ||
        session.title.toLocaleLowerCase().includes(normalizedQuery)),
  );
});

async function importMedia() {
  const session = await application.importMedia();

  if (session) {
    await router.push(`/sessions/${session.id}`);
  }
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
      <AppButton variant="secondary" @click="importMedia">
        {{ t("library.importMedia") }}
      </AppButton>
    </header>

    <AppSearchInput
      class="mb-4"
      v-model="query"
      :placeholder="t('library.search')"
    />

    <AppSurface
      class="library-page__sessions grid min-h-0 grid-rows-[minmax(0,1fr)]"
      :padded="false"
    >
      <div class="library-page__scroll min-h-0 overflow-auto">
        <div class="session-list grid">
          <SessionRow
            v-for="session in sessions"
            :key="session.id"
            :session="session"
            :project-options="projectOptions"
            :moving="isMoving(session.id)"
            @move-to-project="moveSession(session, $event)"
          />
        </div>
      </div>
    </AppSurface>
  </div>
</template>
