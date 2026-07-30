<script setup lang="ts">
import { onMounted } from "vue";
import { RotateCcw, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppStatusState from "@/components/ui/AppStatusState.vue";
import { useTrashStore } from "@/views/trash/trashStore";

const { t } = useI18n();
const trash = useTrashStore();

onMounted(() => {
  void trash.load();
});
</script>

<template>
  <div class="page">
    <header class="page-header page-header--compact">
      <div>
        <p class="eyebrow">{{ t("trash.library") }}</p>
        <h1>{{ t("trash.title") }}</h1>
        <p>{{ t("trash.description") }}</p>
      </div>
    </header>
    <AppSurface :padded="false">
      <AppStatusState
        v-if="trash.isLoading"
        :title="t('trash.loading')"
        loading
      />
      <AppStatusState
        v-else-if="trash.error"
        :title="t('trash.loadFailed')"
        :message="trash.error"
        :retry-label="t('trash.retry')"
        error
        @retry="trash.load"
      />
      <div v-else-if="trash.sessions.length" class="session-list">
        <div
          v-for="session in trash.sessions"
          :key="session.id"
          class="session-row"
        >
          <span class="session-row__icon"><Trash2 :size="18" /></span>
          <span class="session-row__body">
            <strong>{{ session.title }}</strong>
            <small>{{ new Date(session.created_at).toLocaleString() }}</small>
          </span>
          <AppButton
            size="small"
            variant="secondary"
            :loading="trash.restoringSessionId === session.id"
            @click="trash.restore(session.id)"
          >
            <RotateCcw :size="14" />
            {{ t("trash.restore") }}
          </AppButton>
        </div>
      </div>
      <AppEmptyState
        v-else
        :title="t('trash.empty')"
        :message="t('trash.emptyDescription')"
      >
        <template #icon><Trash2 :size="21" /></template>
      </AppEmptyState>
    </AppSurface>
  </div>
</template>
