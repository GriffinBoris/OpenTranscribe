<script setup lang="ts">
import { ref } from "vue";
import { ArrowRight } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import SessionRow from "@/components/session/SessionRow.vue";
import SessionMoveDialog from "@/components/session/SessionMoveDialog.vue";
import type { Session } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useMoveSession } from "@/views/application/useMoveSession";

const application = useApplicationStore();
const { t } = useI18n();
const { isMoving, moveSession, projectOptions } = useMoveSession();
const sessionToMove = ref<Session | null>(null);

async function moveSelectedSession(projectId: string) {
  if (sessionToMove.value) {
    await moveSession(sessionToMove.value, projectId);
  }

  sessionToMove.value = null;
}

function updateMoveDialog(open: boolean) {
  if (!open) {
    sessionToMove.value = null;
  }
}
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
        @move="sessionToMove = session"
      />
    </div>
    <SessionMoveDialog
      :open="Boolean(sessionToMove)"
      :session-count="1"
      :project-options="projectOptions"
      :moving="sessionToMove ? isMoving(sessionToMove.id) : false"
      @update:open="updateMoveDialog"
      @move="moveSelectedSession"
    />
  </section>
</template>
