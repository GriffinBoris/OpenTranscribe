<script setup lang="ts">
import { onMounted, ref } from "vue";
import { KeyRound } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import StatusPill from "@/components/ui/StatusPill.vue";
import { useOpenAiStore } from "@/views/application/openAiStore";

const openAi = useOpenAiStore();
const { t } = useI18n();
const apiKey = ref("");

async function saveApiKey() {
  if (await openAi.saveApiKey(apiKey.value)) {
    apiKey.value = "";
  }
}

onMounted(() => {
  void openAi.loadCredential();
});
</script>

<template>
  <AppSurface id="openai">
    <div
      class="section-heading flex items-start justify-between gap-4 border-b border-[var(--divider)] pb-3.5"
    >
      <div>
        <h2 class="mb-[5px] text-2xl">
          {{ t("settings.navigation.openAi") }}
        </h2>
        <p class="text-ink-muted mt-[3px] leading-[var(--line-height-body)]">
          {{ t("settings.openAi.description") }}
        </p>
      </div>
      <StatusPill v-if="openAi.credential?.configured" tone="success">
        {{ t("settings.openAi.configured") }}
      </StatusPill>
    </div>
    <div
      v-if="openAi.credential?.configured"
      class="grid grid-cols-[auto_minmax(220px,1fr)_auto_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <KeyRound :size="22" />
      <span class="grid gap-1"
        ><strong>{{ t("settings.openAi.defaultProfile") }}</strong
        ><small>{{ openAi.credential.masked_key }}</small></span
      >
      <AppButton size="small" @click="openAi.testConnection">
        {{ t("settings.openAi.testConnection") }}
      </AppButton>
      <AppButton size="small" variant="danger" @click="openAi.removeApiKey">
        {{ t("settings.openAi.remove") }}
      </AppButton>
    </div>
    <form
      v-else
      class="openai-key-form grid grid-cols-[minmax(120px,0.5fr)_minmax(240px,1.3fr)_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
      @submit.prevent="saveApiKey"
    >
      <span>{{ t("settings.openAi.apiKey") }}</span>
      <AppInputText
        v-model="apiKey"
        type="password"
        autocomplete="off"
        placeholder="sk-…"
        required
      />
      <AppButton type="submit">
        {{ t("settings.openAi.save") }}
      </AppButton>
    </form>
    <p v-if="openAi.connectionMessage" role="status">
      {{ openAi.connectionMessage }}
    </p>
  </AppSurface>
</template>
