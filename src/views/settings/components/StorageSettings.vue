<script setup lang="ts">
import { useI18n } from "vue-i18n";

import AppButton from "@/components/ui/AppButton.vue";
import AppSurface from "@/components/ui/AppSurface.vue";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const { t } = useI18n();
</script>

<template>
  <AppSurface id="storage">
    <div class="section-heading">
      <div>
        <h2>{{ t("settings.navigation.storage") }}</h2>
        <p class="setting-description">
          {{ t("settings.storage.description") }}
        </p>
      </div>
    </div>
    <div class="setting-field storage-library-row">
      <span>
        <strong>{{ t("settings.storage.library") }}</strong>
        <small>
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
