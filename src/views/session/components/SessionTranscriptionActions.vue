<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { Cpu, Settings, Sparkles } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSplitButton from "@/components/ui/AppSplitButton.vue";
import { useOpenAiStore } from "@/views/application/openAiStore";
import { openAiModelOptions } from "@/views/application/openAiModels";
import { useLocalModelsStore } from "@/views/models/localModelsStore";
import { useSessionStore } from "@/views/session/sessionStore";

interface TranscriptionTarget {
  label: string;
  value: string;
  provider: "local" | "open_ai";
  modelId: string;
}

const props = defineProps<{
  canTranscribe: boolean;
  hasTranscript: boolean;
  sessionId: string;
}>();

const router = useRouter();
const openAi = useOpenAiStore();
const localModels = useLocalModelsStore();
const sessionStore = useSessionStore();
const { t } = useI18n();
const selectedTarget = ref("");
const activeJob = computed(() =>
  sessionStore.transcriptionJobForSession(props.sessionId),
);
const transcriptionTargets = computed<TranscriptionTarget[]>(() => [
  ...localModels.installedModels.map((model) => ({
    label: t("session.localOption", { model: model.label }),
    value: `local:${model.id}`,
    provider: "local" as const,
    modelId: model.id,
  })),
  ...(openAi.credential?.configured
    ? openAiModelOptions(t).map((model) => ({
        label: model.label,
        value: `open_ai:${model.value}`,
        provider: "open_ai" as const,
        modelId: model.modelId,
      }))
    : []),
]);
const selectedTargetOption = computed(
  () =>
    transcriptionTargets.value.find(
      (target) => target.value === selectedTarget.value,
    ) ?? null,
);

watch(
  transcriptionTargets,
  (targets) => {
    if (targets.some((target) => target.value === selectedTarget.value)) {
      return;
    }

    const preferredLocalModel = localModels.installedModels.find(
      (model) => model.preset === "balanced",
    );
    selectedTarget.value = preferredLocalModel
      ? `local:${preferredLocalModel.id}`
      : (targets[0]?.value ?? "");
  },
  { immediate: true },
);

async function transcribe(target = selectedTargetOption.value) {
  if (!target) {
    return;
  }

  if (target.provider === "local") {
    await sessionStore.transcribeLocally(props.sessionId, target.modelId);
    return;
  }

  await sessionStore.transcribeWithOpenAi(props.sessionId, target.modelId);
}

function selectTarget(value: string) {
  selectedTarget.value = value;
  void transcribe(
    transcriptionTargets.value.find((target) => target.value === value) ?? null,
  );
}

onMounted(() => {
  void localModels.load();
});
</script>

<template>
  <div
    v-if="canTranscribe"
    class="transcription-actions flex max-w-full min-w-0 flex-wrap items-center gap-2"
  >
    <template v-if="transcriptionTargets.length">
      <AppSplitButton
        class="max-w-full shrink-0 max-[720px]:w-full"
        :options="transcriptionTargets"
        :accessible-label="
          props.hasTranscript
            ? t('session.retranscribe')
            : t('session.transcribe')
        "
        :menu-accessible-label="t('session.transcriptionTarget')"
        size="medium"
        :disabled="Boolean(activeJob)"
        @click="transcribe()"
        @select="selectTarget"
      >
        <Cpu v-if="selectedTargetOption?.provider === 'local'" :size="15" />
        <Sparkles v-else :size="15" />
        {{
          props.hasTranscript
            ? t("session.retranscribe")
            : t("session.transcribe")
        }}
      </AppSplitButton>
    </template>
    <AppButton
      v-else
      class="max-[720px]:w-full"
      variant="secondary"
      @click="router.push('/settings#models')"
    >
      <Settings :size="15" />
      {{ t("session.setUpTranscription") }}
    </AppButton>
  </div>
</template>
