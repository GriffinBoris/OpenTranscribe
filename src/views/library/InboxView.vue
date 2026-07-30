<script setup lang="ts">
import { computed, ref } from "vue";
import { Search } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import SessionRow from "@/components/session/SessionRow.vue";
import AppButton from "@/components/ui/AppButton.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
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
  <div class="page">
    <header class="page-header page-header--compact">
      <h1>{{ t("library.inbox") }}</h1>
      <AppButton variant="secondary" @click="importMedia">
        {{ t("library.importMedia") }}
      </AppButton>
    </header>

    <div class="search-field">
      <Search :size="16" /><AppInputText
        v-model="query"
        type="search"
        :placeholder="t('library.search')"
      />
    </div>

    <AppSurface :padded="false">
      <div class="session-list">
        <SessionRow
          v-for="session in sessions"
          :key="session.id"
          :session="session"
        />
      </div>
    </AppSurface>
  </div>
</template>
