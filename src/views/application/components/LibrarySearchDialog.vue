<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { FileText, MessageSquareText, Search } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import { native } from "@/core/native";
import type { SearchResult } from "@/types/domain";

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const { t } = useI18n();
const router = useRouter();
const query = ref("");
const results = ref<SearchResult[]>([]);
const errorMessage = ref("");
const isSearching = ref(false);
const hasSearched = ref(false);

const canSearch = computed(
  () => query.value.trim().length > 0 && !isSearching.value,
);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }

    query.value = "";
    results.value = [];
    errorMessage.value = "";
    hasSearched.value = false;
  },
);

async function searchLibrary() {
  const searchQuery = query.value.trim();

  if (!searchQuery) {
    return;
  }

  isSearching.value = true;
  errorMessage.value = "";

  try {
    const page = await native.searchLibrary(searchQuery, {
      project_id: null,
      content_kinds: [],
    });
    results.value = page.results;
    hasSearched.value = true;
  } catch (reason) {
    errorMessage.value =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isSearching.value = false;
  }
}

async function openResult(result: SearchResult) {
  emit("update:open", false);
  await router.push({
    name: "session",
    params: { sessionId: result.session_id },
    query:
      result.timestamp_ms === null
        ? undefined
        : { at: String(result.timestamp_ms) },
  });
}

function resultIcon(kind: SearchResult["kind"]) {
  if (kind === "notes") {
    return MessageSquareText;
  }

  return FileText;
}
</script>

<template>
  <AppDialog
    class="library-search-dialog"
    :open="open"
    :title="t('librarySearch.title')"
    @update:open="emit('update:open', $event)"
  >
    <form class="library-search" @submit.prevent="searchLibrary">
      <div class="search-field">
        <Search :size="16" aria-hidden="true" />
        <AppInputText
          v-model="query"
          type="search"
          :placeholder="t('librarySearch.placeholder')"
          autofocus
        />
      </div>
      <AppButton type="submit" variant="primary" :disabled="!canSearch">
        {{ t("librarySearch.action") }}
      </AppButton>
    </form>

    <p v-if="errorMessage" class="shell-state shell-state--error" role="alert">
      {{ errorMessage }}
    </p>

    <div v-else-if="results.length" class="library-search__results">
      <AppButton
        v-for="result in results"
        :key="`${result.session_id}-${result.kind}-${result.excerpt}`"
        class="library-search__result"
        variant="ghost"
        @click="openResult(result)"
      >
        <component
          :is="resultIcon(result.kind)"
          :size="16"
          aria-hidden="true"
        />
        <span>
          <strong>{{ result.session_title }}</strong>
          <small>
            {{ t(`librarySearch.kinds.${result.kind}`) }} ·
            {{ result.excerpt }}
          </small>
        </span>
      </AppButton>
    </div>

    <div v-else-if="hasSearched" class="empty-setting">
      <span>
        <strong>{{ t("librarySearch.empty") }}</strong>
        <small>{{ t("librarySearch.emptyDescription") }}</small>
      </span>
    </div>
  </AppDialog>
</template>
