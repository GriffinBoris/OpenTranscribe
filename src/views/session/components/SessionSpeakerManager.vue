<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import type { Speaker } from "@/types/domain";
import { useSessionStore } from "@/views/session/sessionStore";

const props = defineProps<{
  open: boolean;
  sessionId: string;
  speakers: Speaker[];
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
}>();

const { t } = useI18n();
const sessionStore = useSessionStore();
const names = ref<Record<string, string>>({});
const sourceSpeakerId = ref("");
const targetSpeakerId = ref("");
const savingSpeakerId = ref<string | null>(null);
const isMerging = ref(false);

const speakerOptions = computed(() =>
  props.speakers.map((speaker) => ({
    label: speaker.display_name,
    value: speaker.id,
  })),
);
const targetOptions = computed(() =>
  speakerOptions.value.filter(
    (speaker) => speaker.value !== sourceSpeakerId.value,
  ),
);

watch(
  () => [props.open, props.speakers] as const,
  () => {
    if (!props.open) {
      return;
    }

    names.value = Object.fromEntries(
      props.speakers.map((speaker) => [speaker.id, speaker.display_name]),
    );
    sourceSpeakerId.value = props.speakers[0]?.id ?? "";
    targetSpeakerId.value = props.speakers[1]?.id ?? "";
  },
  { immediate: true },
);

watch(sourceSpeakerId, () => {
  if (targetSpeakerId.value === sourceSpeakerId.value) {
    targetSpeakerId.value = targetOptions.value[0]?.value ?? "";
  }
});

async function saveName(speaker: Speaker) {
  const name = names.value[speaker.id]?.trim();

  if (!name || name === speaker.display_name) {
    return;
  }

  savingSpeakerId.value = speaker.id;

  try {
    await sessionStore.renameSpeaker(props.sessionId, speaker.id, name);
  } finally {
    savingSpeakerId.value = null;
  }
}

async function merge() {
  if (!sourceSpeakerId.value || !targetSpeakerId.value) {
    return;
  }

  isMerging.value = true;

  try {
    await sessionStore.mergeSpeakers(
      props.sessionId,
      sourceSpeakerId.value,
      targetSpeakerId.value,
    );
  } finally {
    isMerging.value = false;
  }
}
</script>

<template>
  <AppDialog
    :open="open"
    :title="t('session.speakers.manage')"
    @update:open="emit('update:open', $event)"
  >
    <div class="grid gap-[var(--space-5-5)]">
      <div class="grid gap-2.5">
        <div
          v-for="speaker in speakers"
          :key="speaker.id"
          class="flex items-center gap-2"
        >
          <AppInputText
            class="flex-1"
            v-model="names[speaker.id]"
            :aria-label="t('session.speakers.name')"
          />
          <AppButton
            variant="secondary"
            :disabled="
              !names[speaker.id]?.trim() ||
              names[speaker.id]?.trim() === speaker.display_name
            "
            :loading="savingSpeakerId === speaker.id"
            @click="saveName(speaker)"
          >
            {{ t("session.save") }}
          </AppButton>
        </div>
      </div>

      <div
        v-if="speakers.length > 1"
        class="grid gap-2 border-t border-[var(--divider)] pt-[18px]"
      >
        <strong>{{ t("session.speakers.merge") }}</strong>
        <p class="text-md text-ink-muted m-0">
          {{ t("session.speakers.mergeDescription") }}
        </p>
        <div class="flex items-center gap-2 max-[600px]:flex-wrap">
          <AppSelect
            class="flex-1"
            v-model="sourceSpeakerId"
            :options="speakerOptions"
            :accessible-label="t('session.speakers.mergeSource')"
          />
          <span aria-hidden="true">→</span>
          <AppSelect
            class="flex-1"
            v-model="targetSpeakerId"
            :options="targetOptions"
            :accessible-label="t('session.speakers.mergeTarget')"
          />
          <AppButton
            variant="secondary"
            :disabled="!sourceSpeakerId || !targetSpeakerId"
            :loading="isMerging"
            @click="merge"
          >
            {{ t("session.speakers.mergeAction") }}
          </AppButton>
        </div>
      </div>
    </div>
  </AppDialog>
</template>
