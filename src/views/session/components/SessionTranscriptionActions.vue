<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Cpu, Settings, Sparkles } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import type { OpenAiTranscriptionModel } from "@/types/domain";
import { useOpenAiStore } from "@/views/application/openAiStore";
import {
  openAiModelId,
  openAiModelOptions,
} from "@/views/application/openAiModels";
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
const selectedOpenAiModel = ref<OpenAiTranscriptionModel>("gpt_transcribe");
const activeJob = computed(() =>
  sessionStore.transcriptionJobForSession(props.sessionId),
);

const localModelOptions = computed(() =>
  localModels.installedModels.map((model) => ({
    label: t("session.localOption", { model: model.label }),
    value: model.id,
  })),
);
const cloudModelOptions = computed(() => openAiModelOptions(t));
async function transcribeLocally() {
  if (selectedLocalModelId.value) {
    await sessionStore.transcribeLocally(
      props.sessionId,
      selectedLocalModelId.value,
    );
  }
}

async function transcribeWithOpenAi() {
  await sessionStore.transcribeWithOpenAi(
    props.sessionId,
    openAiModelId(selectedOpenAiModel.value),
  );
}

onMounted(async () => {
  await localModels.load();
  selectedLocalModelId.value =
    localModels.installedModels.find((model) => model.preset === "balanced")
      ?.id ??
    localModels.installedModels[0]?.id ??
    "";
  selectedOpenAiModel.value = "gpt_transcribe";
});
</script>

<template>
  <div
    v-if="canTranscribe"
    class="transcription-actions flex w-full min-w-0 items-center gap-2 max-[720px]:flex-wrap"
  >
    <template v-if="localModels.installedModels.length">
      <AppSelect
        class="w-[150px] flex-1 text-sm max-[720px]:w-full max-[720px]:flex-none"
        v-model="selectedLocalModelId"
        :options="localModelOptions"
        :accessible-label="t('session.localModel')"
      />
      <AppButton
        class="max-[720px]:w-full"
        variant="primary"
        :disabled="Boolean(activeJob)"
        :loading="activeJob?.kind === 'transcribe_local'"
        @click="transcribeLocally"
      >
        <Cpu :size="15" />
        {{ t("session.transcribe") }}
      </AppButton>
    </template>
    <template v-if="openAi.credential?.configured">
      <AppSelect
        v-model="selectedOpenAiModel"
        class="w-[190px] flex-1 text-sm max-[720px]:w-full max-[720px]:flex-none"
        :options="cloudModelOptions"
        :accessible-label="t('session.openAiModel')"
      />
      <AppButton
        class="max-[720px]:w-full"
        variant="secondary"
        :disabled="Boolean(activeJob)"
        :loading="activeJob?.kind === 'transcribe_open_ai'"
        @click="transcribeWithOpenAi"
      >
        <Sparkles :size="15" />
        {{ t("session.openAi") }}
      </AppButton>
    </template>
    <AppButton
      v-if="
        !localModels.installedModels.length && !openAi.credential?.configured
      "
      class="max-[720px]:w-full"
      variant="secondary"
      @click="router.push('/settings#models')"
    >
      <Settings :size="15" />
      {{ t("session.setUpTranscription") }}
    </AppButton>
  </div>
</template>
