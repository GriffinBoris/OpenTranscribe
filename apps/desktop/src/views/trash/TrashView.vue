<script setup lang="ts">
import { onMounted, ref } from "vue";
import { RotateCcw, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import AppStatusState from "@/components/ui/AppStatusState.vue";
import { useTrashStore } from "@/views/trash/trashStore";

const { t } = useI18n();
const trash = useTrashStore();
const emptyDialogOpen = ref(false);

onMounted(() => {
  void trash.load();
});

async function emptyTrash() {
  if (await trash.empty()) {
    emptyDialogOpen.value = false;
  }
}
</script>

<template>
  <div
    class="page library-page trash-page grid h-full min-h-0 w-full grid-rows-[auto_minmax(0,1fr)] overflow-hidden px-[var(--layout-page-gutter)] pt-[var(--space-13)] pb-[var(--space-14)]"
  >
    <header
      class="page-header page-header--compact mb-[var(--space-5-5)] flex items-start justify-between gap-6"
    >
      <div>
        <p
          class="eyebrow text-ink mb-1.5 text-xs font-black tracking-[0.1em] uppercase"
        >
          {{ t("trash.library") }}
        </p>
        <h1
          class="text-display m-0 max-w-[730px] leading-[var(--line-height-tight)] font-bold tracking-[-0.035em]"
        >
          {{ t("trash.title") }}
        </h1>
        <p class="text-ink">{{ t("trash.description") }}</p>
      </div>
      <AppButton
        v-if="trash.sessions.length"
        variant="danger"
        :disabled="trash.isLoading"
        @click="emptyDialogOpen = true"
      >
        <Trash2 :size="16" />{{ t("trash.emptyAction") }}
      </AppButton>
    </header>
    <AppSurface
      class="library-page__sessions grid min-h-0 grid-rows-[minmax(0,1fr)]"
      :padded="false"
    >
      <div class="library-page__scroll min-h-0 overflow-auto">
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
        <div v-else-if="trash.sessions.length" class="session-list grid">
          <div
            v-for="session in trash.sessions"
            :key="session.id"
            class="session-row hover:bg-canvas-subtle flex min-w-0 items-center gap-3 border-b border-[var(--divider)] px-[15px] py-[13px] last:border-b-0"
          >
            <span
              class="session-row__icon rounded-app-md bg-lichen-soft text-lichen grid size-[35px] shrink-0 place-items-center"
              ><Trash2 :size="18"
            /></span>
            <span class="session-row__body grid min-w-0 flex-1 gap-1">
              <strong class="truncate">{{ session.title }}</strong>
              <small class="text-ink-muted block">{{
                new Date(session.created_at).toLocaleString()
              }}</small>
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
      </div>
    </AppSurface>
    <AppDialog
      :open="emptyDialogOpen"
      :title="t('trash.emptyTitle')"
      @update:open="emptyDialogOpen = $event"
    >
      <p class="m-0">{{ t("trash.emptyConfirmation") }}</p>
      <template #footer>
        <AppButton
          variant="ghost"
          :disabled="trash.isEmptying"
          @click="emptyDialogOpen = false"
        >
          {{ t("navigation.cancel") }}
        </AppButton>
        <AppButton
          variant="danger"
          :loading="trash.isEmptying"
          @click="emptyTrash"
        >
          {{ t("trash.emptyConfirm") }}
        </AppButton>
      </template>
    </AppDialog>
  </div>
</template>
