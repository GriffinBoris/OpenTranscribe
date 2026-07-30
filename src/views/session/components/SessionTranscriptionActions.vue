<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Cpu, Settings, Sparkles } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import { useOpenAiStore } from "@/views/application/openAiStore";
import { useLocalModelsStore } from "@/views/models/localModelsStore";
import { useSessionStore } from "@/views/session/sessionStore";

const props = defineProps<{
  canTranscribe: boolean;
  sessionId: string;
}>();

const router = useRouter();
const openAi = useOpenAiStore();
const localModels = useLocalModelsStore();
const sessionStore = useSessionStore();
const { t } = useI18n();
const selectedLocalModelId = ref("");
const activeJob = computed(() =>
  sessionStore.transcriptionJobForSession(props.sessionId),
);

const localModelOptions = computed(() =>
  localModels.installedModels.map((model) => ({
    label: t("session.localOption", { model: model.label }),
    value: model.id,
  })),
);
async function transcribeLocally() {
  if (selectedLocalModelId.value) {
    await sessionStore.transcribeLocally(
      props.sessionId,
      selectedLocalModelId.value,
    );
  }
}

onMounted(async () => {
  await localModels.load();
  selectedLocalModelId.value =
    localModels.installedModels.find((model) => model.preset === "balanced")
      ?.id ??
    localModels.installedModels[0]?.id ??
    "";
});
</script>

<template>
  <div v-if="canTranscribe" class="transcription-actions">
    <template v-if="localModels.installedModels.length">
      <AppSelect
        v-model="selectedLocalModelId"
        :options="localModelOptions"
        :accessible-label="t('session.localModel')"
      />
      <AppButton
        variant="primary"
        :disabled="Boolean(activeJob)"
        :loading="activeJob?.kind === 'transcribe_local'"
        @click="transcribeLocally"
      >
        <Cpu :size="15" />
        {{ t("session.transcribe") }}
      </AppButton>
    </template>
    <AppButton
      v-if="openAi.credential?.configured"
      variant="secondary"
      :disabled="Boolean(activeJob)"
      :loading="activeJob?.kind === 'transcribe_open_ai'"
      @click="sessionStore.transcribeWithOpenAi(sessionId)"
    >
      <Sparkles :size="15" />
      {{ t("session.openAi") }}
    </AppButton>
    <AppButton
      v-if="
        !localModels.installedModels.length && !openAi.credential?.configured
      "
      variant="secondary"
      @click="router.push('/settings#models')"
    >
      <Settings :size="15" />
      {{ t("session.setUpTranscription") }}
    </AppButton>
  </div>
</template>
