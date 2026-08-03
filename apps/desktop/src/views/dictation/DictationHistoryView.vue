<script setup lang="ts">
import { onMounted, ref } from "vue";
import { MessageSquareText, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppCopyButton from "@/components/ui/AppCopyButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppEmptyState from "@/components/ui/AppEmptyState.vue";
import AppStatusState from "@/components/ui/AppStatusState.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import { native } from "@/core/native";
import type { DictationProvider } from "@/types/domain";
import { formatUsd } from "@/views/application/jobEstimates";

const { t } = useI18n();
const history = ref<Awaited<ReturnType<typeof native.dictationHistory>>>([]);
const isLoading = ref(false);
const isClearing = ref(false);
const clearDialogOpen = ref(false);
const errorMessage = ref<string | null>(null);

async function loadHistory() {
  isLoading.value = true;
  errorMessage.value = null;

  try {
    history.value = await native.dictationHistory();
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isLoading.value = false;
  }
}

function providerLabel(provider: DictationProvider) {
  return provider === "open_ai"
    ? t("dictationHistory.openAi")
    : t("dictationHistory.local");
}

async function clearHistory() {
  isClearing.value = true;
  errorMessage.value = null;

  try {
    await native.clearDictationHistory();
    history.value = [];
    clearDialogOpen.value = false;
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
    clearDialogOpen.value = false;
  } finally {
    isClearing.value = false;
  }
}

onMounted(() => void loadHistory());
</script>

<template>
  <div
    class="page library-page grid h-full min-h-0 w-full grid-rows-[auto_minmax(0,1fr)] overflow-hidden px-[var(--layout-page-gutter)] pt-[var(--space-13)] pb-[var(--space-14)]"
  >
    <header
      class="page-header page-header--compact mb-[var(--space-5-5)] flex items-start justify-between gap-6"
    >
      <div>
        <h1
          class="text-display m-0 max-w-[730px] leading-[var(--line-height-tight)] font-bold tracking-[-0.035em]"
        >
          {{ t("dictationHistory.title") }}
        </h1>
        <p class="text-ink mb-0">{{ t("dictationHistory.description") }}</p>
      </div>
      <AppButton
        v-if="history.length"
        class="shrink-0"
        size="small"
        variant="danger"
        :aria-label="t('dictationHistory.clearAction')"
        :title="t('dictationHistory.clearAction')"
        :disabled="isLoading || isClearing"
        @click="clearDialogOpen = true"
      >
        <Trash2 :size="16" aria-hidden="true" />
      </AppButton>
    </header>

    <AppSurface
      class="library-page__sessions grid min-h-0 grid-rows-[minmax(0,1fr)]"
      :padded="false"
    >
      <div class="library-page__scroll min-h-0 overflow-auto">
        <AppStatusState
          v-if="isLoading"
          :title="t('dictationHistory.loading')"
          loading
        />
        <AppStatusState
          v-else-if="errorMessage"
          :title="t('dictationHistory.loadFailed')"
          :message="errorMessage"
          :retry-label="t('dictationHistory.retry')"
          error
          @retry="loadHistory"
        />
        <div
          v-else-if="history.length"
          class="grid divide-y divide-[var(--divider)]"
        >
          <article
            v-for="entry in history"
            :key="entry.id"
            class="grid gap-2 px-[var(--space-5-5)] py-4"
          >
            <div class="flex items-start gap-3">
              <p class="text-ink m-0 min-w-0 flex-1 whitespace-pre-wrap">
                {{ entry.text }}
              </p>
              <AppCopyButton
                :text="entry.text"
                :label="t('dictationHistory.copy')"
                :copied-label="t('common.copied')"
              />
            </div>
            <div
              class="text-ink-muted flex flex-wrap items-center gap-x-2 gap-y-1 text-sm"
            >
              <span>{{ new Date(entry.created_at).toLocaleString() }}</span>
              <span aria-hidden="true">·</span>
              <span>{{ providerLabel(entry.provider) }}</span>
              <span aria-hidden="true">·</span>
              <span>{{ entry.model_id }}</span>
              <template v-if="entry.provider === 'open_ai'">
                <span aria-hidden="true">·</span>
                <span>
                  {{
                    entry.approximate_cost_usd === null
                      ? t("processing.costUnavailable")
                      : t("processing.totalEstimatedCost", {
                          cost: formatUsd(entry.approximate_cost_usd),
                        })
                  }}
                </span>
              </template>
            </div>
          </article>
        </div>
        <AppEmptyState
          v-else
          :title="t('dictationHistory.empty')"
          :message="t('dictationHistory.emptyDescription')"
        >
          <template #icon><MessageSquareText :size="21" /></template>
        </AppEmptyState>
      </div>
    </AppSurface>
    <AppDialog
      :open="clearDialogOpen"
      :title="t('dictationHistory.clearTitle')"
      @update:open="clearDialogOpen = $event"
    >
      <p class="m-0">{{ t("dictationHistory.clearConfirmation") }}</p>
      <template #footer>
        <AppButton
          variant="ghost"
          :disabled="isClearing"
          @click="clearDialogOpen = false"
        >
          {{ t("navigation.cancel") }}
        </AppButton>
        <AppButton variant="danger" :loading="isClearing" @click="clearHistory">
          {{ t("dictationHistory.clearConfirm") }}
        </AppButton>
      </template>
    </AppDialog>
  </div>
</template>
