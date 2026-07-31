<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import {
  Folder,
  Home,
  Inbox,
  ListTodo,
  Mic2,
  Plus,
  Search,
  Settings,
  Trash2,
} from "@lucide/vue";

import AppButton from "@/components/ui/AppButton.vue";
import AppDialog from "@/components/ui/AppDialog.vue";
import AppInputText from "@/components/ui/AppInputText.vue";
import { useApplicationStore } from "@/views/application/applicationStore";
import LibrarySearchDialog from "@/views/application/components/LibrarySearchDialog.vue";
import { useRecordingStore } from "@/views/application/recordingStore";

const application = useApplicationStore();
const recording = useRecordingStore();
const { t } = useI18n();
const router = useRouter();
const projectDialogOpen = ref(false);
const searchDialogOpen = ref(false);
const projectName = ref("");
const projectError = ref("");
const isCreatingProject = ref(false);
const isStartingRecording = ref(false);
const sidebarLinkClass =
  "sidebar__link flex min-h-[var(--control-height-small)] min-w-0 items-center gap-2 rounded-app-xs px-2 py-1.5 text-md font-medium text-[color-mix(in_srgb,var(--text)_82%,transparent)] hover:bg-surface/70 [&.router-link-active]:bg-surface/70 [&.router-link-active]:text-ink max-[900px]:justify-center";

async function createProject() {
  const name = projectName.value.trim();

  if (!name) {
    return;
  }

  isCreatingProject.value = true;
  projectError.value = "";

  try {
    await application.createProject(name);
    projectName.value = "";
    projectDialogOpen.value = false;
  } catch (reason) {
    projectError.value =
      reason instanceof Error ? reason.message : String(reason);
  } finally {
    isCreatingProject.value = false;
  }
}

function openProjectDialog() {
  projectError.value = "";
  projectDialogOpen.value = true;
}

async function startRecording() {
  isStartingRecording.value = true;

  try {
    const session = await recording.createSession(
      application.settings?.recording_mode ?? "record_only",
    );

    if (session) {
      await router.push(`/sessions/${session.id}`);
    }
  } finally {
    isStartingRecording.value = false;
  }
}

function handleShortcut(event: KeyboardEvent) {
  if (
    (event.metaKey || event.ctrlKey) &&
    !event.shiftKey &&
    event.key.toLocaleLowerCase() === "k"
  ) {
    event.preventDefault();
    searchDialogOpen.value = true;
  }
}

onMounted(() => window.addEventListener("keydown", handleShortcut));
onBeforeUnmount(() => window.removeEventListener("keydown", handleShortcut));
</script>

<template>
  <aside
    class="sidebar relative col-start-1 row-start-2 flex min-w-0 flex-col bg-transparent px-2.5 pt-2 pb-2.5"
  >
    <div
      class="sidebar__brand flex items-center gap-2 px-2 pt-0.5 pb-4 text-base font-bold max-[900px]:justify-center max-[900px]:px-0"
    >
      <img
        class="sidebar__logo rounded-app-xs size-[26px] shadow-[var(--shadow-brand)]"
        src="@/assets/opentranscribe-icon.svg"
        alt=""
        data-testid="app-brand-icon"
      />
      <span class="max-[900px]:hidden">{{ t("appName") }}</span>
    </div>

    <div class="sidebar__primary-actions mb-2.5 grid gap-1">
      <AppButton
        class="w-full justify-start max-[900px]:size-[38px] max-[900px]:justify-self-center max-[900px]:p-0 max-[900px]:text-[0]"
        variant="primary"
        :loading="isStartingRecording"
        :disabled="Boolean(recording.activeRecording)"
        @click="startRecording"
      >
        <Mic2 :size="16" />
        <span class="max-[900px]:hidden">
          {{ t("navigation.newRecording") }}
        </span>
      </AppButton>
      <AppButton
        class="w-full justify-start max-[900px]:size-[38px] max-[900px]:justify-self-center max-[900px]:p-0 max-[900px]:text-[0] max-[900px]:[&>kbd]:hidden"
        variant="ghost"
        @click="searchDialogOpen = true"
      >
        <Search :size="16" />
        <span class="max-[900px]:hidden">{{ t("navigation.search") }}</span>
        <kbd class="text-ink-muted ml-auto text-xs max-[900px]:hidden">⌘K</kbd>
      </AppButton>
    </div>

    <nav class="sidebar__nav grid gap-1" :aria-label="t('navigation.main')">
      <RouterLink to="/" :class="sidebarLinkClass"
        ><Home :size="17" /><span class="truncate max-[900px]:hidden">{{
          t("navigation.home")
        }}</span></RouterLink
      >
      <RouterLink to="/inbox" :class="sidebarLinkClass"
        ><Inbox :size="17" /><span class="truncate max-[900px]:hidden">{{
          t("navigation.inbox")
        }}</span></RouterLink
      >
      <RouterLink to="/processing" :class="sidebarLinkClass">
        <ListTodo :size="17" />
        <span class="truncate max-[900px]:hidden">{{
          t("navigation.processing")
        }}</span>
        <span
          v-if="application.activeJobs.length"
          class="sidebar__count bg-accent text-accent-contrast ml-auto min-w-5 rounded-full px-1.5 py-0.5 text-center text-xs max-[900px]:hidden"
        >
          {{ application.activeJobs.length }}
        </span>
      </RouterLink>
    </nav>

    <div class="sidebar__section mt-4 grid min-h-0 gap-1 overflow-auto">
      <div
        class="sidebar__section-label text-2xs text-ink-muted px-2 pb-1.5 font-bold tracking-[0.07em] uppercase max-[900px]:hidden"
      >
        {{ t("navigation.projects") }}
      </div>
      <RouterLink
        v-for="project in application.projects"
        :key="project.id"
        :to="`/projects/${project.id}`"
        :aria-label="project.name"
        :class="sidebarLinkClass"
      >
        <Folder :size="16" />
        <span class="truncate max-[900px]:hidden">{{ project.name }}</span>
        <span
          class="sidebar__project-count text-ink-muted ml-auto text-xs tabular-nums max-[900px]:hidden"
          aria-hidden="true"
        >
          {{ application.projectSessionCounts.get(project.id) ?? 0 }}
        </span>
      </RouterLink>
      <AppButton
        class="sidebar__new-project text-ink-muted w-full justify-start px-2 py-[7px] text-left max-[900px]:hidden"
        variant="ghost"
        size="small"
        @click="openProjectDialog"
      >
        <Plus :size="15" />{{ t("navigation.newProject") }}
      </AppButton>
    </div>

    <div class="sidebar__footer mt-auto grid gap-1 pt-2.5">
      <RouterLink to="/trash" :class="sidebarLinkClass"
        ><Trash2 :size="17" /><span class="truncate max-[900px]:hidden">{{
          t("navigation.trash")
        }}</span></RouterLink
      >
      <RouterLink to="/settings" :class="sidebarLinkClass">
        <Settings :size="17" /><span class="truncate max-[900px]:hidden">{{
          t("navigation.settings")
        }}</span>
      </RouterLink>
    </div>

    <AppDialog
      :open="projectDialogOpen"
      :title="t('navigation.newProject')"
      @update:open="projectDialogOpen = $event"
    >
      <form id="create-project-form" @submit.prevent="createProject">
        <label
          class="dialog-field text-ink-muted grid gap-2 text-sm font-semibold"
        >
          <span>{{ t("navigation.projectName") }}</span>
          <AppInputText
            v-model="projectName"
            autocomplete="off"
            :placeholder="t('navigation.projectNamePlaceholder')"
            autofocus
            required
          />
        </label>
        <p
          v-if="projectError"
          class="text-accent mt-[var(--space-2-5)] text-sm"
          role="alert"
        >
          {{ projectError }}
        </p>
      </form>
      <template #footer>
        <AppButton
          variant="ghost"
          :disabled="isCreatingProject"
          @click="projectDialogOpen = false"
        >
          {{ t("navigation.cancel") }}
        </AppButton>
        <AppButton
          type="submit"
          form="create-project-form"
          variant="primary"
          :disabled="!projectName.trim()"
          :loading="isCreatingProject"
        >
          {{ t("navigation.createProject") }}
        </AppButton>
      </template>
    </AppDialog>

    <LibrarySearchDialog
      :open="searchDialogOpen"
      @update:open="searchDialogOpen = $event"
    />
  </aside>
</template>
