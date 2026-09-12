<script setup lang="ts">
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const { t } = useI18n();
</script>

<template>
  <AppSurface id="settings-storage">
    <div class="border-b border-[var(--divider)] pb-3.5">
      <div>
        <h2 class="mb-[5px] text-2xl">
          {{ t("settings.navigation.storage") }}
        </h2>
        <p class="text-ink mt-[3px] leading-[var(--line-height-body)]">
          {{ t("settings.storage.description") }}
        </p>
      </div>
    </div>
    <div
      class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-t border-[var(--divider)] py-3 max-[700px]:grid-cols-1"
    >
      <span class="grid min-w-0 gap-1">
        <strong>{{ t("settings.storage.library") }}</strong>
        <small class="text-ink-muted wrap-anywhere">
          {{
            application.snapshot?.library?.path ??
            t("settings.storage.noLibrary")
          }}
        </small>
      </span>
      <AppButton variant="secondary" @click="application.chooseLibrary">
        {{
          application.snapshot?.library
            ? t("settings.storage.change")
            : t("settings.storage.choose")
        }}
      </AppButton>
    </div>
  </AppSurface>
</template>
