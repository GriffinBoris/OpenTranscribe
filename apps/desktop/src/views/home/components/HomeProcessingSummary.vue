<script setup lang="ts">
import { ArrowRight } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import AppProgressBar from "@/components/ui/AppProgressBar.vue";
import type { Job } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

const application = useApplicationStore();
const { t } = useI18n();

function jobProgress(job: Job) {
  if (!job.progress?.total_units) {
    return undefined;
  }

  return (job.progress.completed_units / job.progress.total_units) * 100;
}

function jobProvider(job: Job) {
  return job.kind.includes("open_ai")
    ? t("processing.openAi")
    : t("home.localProvider");
}
</script>

<template>
  <section
    class="home-section processing-summary min-w-0 border-l border-[var(--divider)] pl-6 max-[1100px]:border-l-0 max-[1100px]:pl-0"
  >
    <div
      class="section-heading mb-0 flex items-center justify-between gap-4 border-b border-[var(--divider)] pb-[11px]"
    >
      <div>
        <h2 class="m-0 text-xl tracking-[-0.01em]">
          {{ t("home.processing") }}
        </h2>
        <p class="text-ink-muted mt-[3px] text-sm">
          {{ t("home.activeJobs", { count: application.runningJobs.length }) }}
        </p>
      </div>
    </div>
    <p
      v-if="application.runningJobs.length === 0"
      class="quiet-state text-md text-ink-muted m-0 py-[18px]"
    >
      {{ t("home.nothingRunning") }}
    </p>
    <div
      v-for="job in application.runningJobs"
      :key="job.id"
      class="compact-job grid gap-2.5 pt-3 pb-[18px]"
    >
      <div>
        <strong>{{ job.progress?.message }}</strong>
        <small class="text-ink-muted block">{{ jobProvider(job) }}</small>
      </div>
      <AppProgressBar
        :value="jobProgress(job)"
        :accessible-label="job.progress?.message ?? t('home.processingSession')"
      />
    </div>
    <RouterLink
      to="/processing"
      class="text-link text-lichen inline-flex items-center gap-1.5 text-sm font-bold"
    >
      {{ t("home.openProcessing") }} <ArrowRight :size="14" />
    </RouterLink>
  </section>
</template>
