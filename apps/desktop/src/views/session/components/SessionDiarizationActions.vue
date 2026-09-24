<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { Users } from "@lucide/vue";

import AppButton from "@/components/ui/AppButton.vue";
import { useLocalModelsStore } from "@/views/application/localModelsStore";
import { useSessionStore } from "@/views/session/sessionStore";

const props = defineProps<{ sessionId: string; disabled: boolean }>();
const { t } = useI18n();
const router = useRouter();
const localModels = useLocalModelsStore();
const sessionStore = useSessionStore();
const model = computed(() => localModels.diarizationModel);

function diarize() {
  if (model.value?.installed) {
    void sessionStore.diarize(props.sessionId, model.value.id);
  }
}
</script>

<template>
  <div>
    <AppButton
      v-if="model?.installed"
      :title="`${t('session.diarizationHelp')} ${t('session.diarizationPrecision')}`"
      variant="secondary"
      :disabled="disabled"
      @click="diarize"
    >
      <Users :size="15" />
      {{ t("session.rediarize") }}
    </AppButton>
    <AppButton
      v-else
      variant="ghost"
      :disabled="disabled"
      @click="router.push('/settings#models')"
    >
      {{ t("session.setUpDiarization") }}
    </AppButton>
  </div>
</template>
