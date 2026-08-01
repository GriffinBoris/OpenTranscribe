<script setup lang="ts">
import { ArrowRight } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import SessionRow from "@/components/session/SessionRow.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useMoveSession } from "@/views/application/useMoveSession";

const application = useApplicationStore();
const { t } = useI18n();
const { isMoving, moveSession, projectOptions } = useMoveSession();
</script>

<template>
  <section class="home-section recent-card min-w-0">
    <div
      class="section-heading mb-0 flex items-center justify-between gap-4 border-b border-[var(--divider)] pb-[11px]"
    >
      <h2 class="m-0 text-xl tracking-[-0.01em]">
        {{ t("home.recentSessions") }}
      </h2>
      <RouterLink
        to="/inbox"
        class="text-link text-lichen inline-flex items-center gap-1.5 text-sm font-bold"
      >
        {{ t("home.viewAll") }} <ArrowRight :size="14" />
      </RouterLink>
    </div>
    <div class="session-list grid">
      <p
        v-if="application.recentSessions.length === 0"
        class="quiet-state text-md text-ink-muted m-0 py-[18px]"
      >
        {{ t("home.noSessions") }}
      </p>
      <SessionRow
        v-for="session in application.recentSessions.slice(0, 4)"
        :key="session.id"
        :session="session"
        :project-options="projectOptions"
        :moving="isMoving(session.id)"
        @move-to-project="moveSession(session, $event)"
      />
    </div>
  </section>
</template>
