<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import AppButton from "@/components/ui/AppButton.vue";
import AppSearchInput from "@/components/ui/AppSearchInput.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import MovableSessionRow from "@/views/application/components/MovableSessionRow.vue";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const { t } = useI18n();
const router = useRouter();
const query = ref("");
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
  <div class="page library-page">
    <header class="page-header page-header--compact">
      <h1>{{ t("library.inbox") }}</h1>
      <AppButton variant="secondary" @click="importMedia">
        {{ t("library.importMedia") }}
      </AppButton>
    </header>

    <AppSearchInput v-model="query" :placeholder="t('library.search')" />

    <AppSurface class="library-page__sessions" :padded="false">
      <div class="library-page__scroll">
        <div class="session-list">
          <MovableSessionRow
            v-for="session in sessions"
            :key="session.id"
            :session="session"
          />
        </div>
      </div>
    </AppSurface>
  </div>
</template>
