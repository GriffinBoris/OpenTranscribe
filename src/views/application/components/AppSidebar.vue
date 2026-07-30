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
  <aside class="sidebar">
    <div class="sidebar__brand">
      <img
        class="sidebar__logo"
        src="@/assets/opentranscribe-icon.svg"
        alt=""
        data-testid="app-brand-icon"
      />
      <span>{{ t("appName") }}</span>
    </div>

    <div class="sidebar__primary-actions">
      <AppButton
        variant="primary"
        :loading="isStartingRecording"
        :disabled="Boolean(recording.activeRecording)"
        @click="startRecording"
      >
        <Mic2 :size="16" />{{ t("navigation.newRecording") }}
      </AppButton>
      <AppButton variant="ghost" @click="searchDialogOpen = true">
        <Search :size="16" />{{ t("navigation.search") }}
        <kbd>⌘K</kbd>
      </AppButton>
    </div>

    <nav class="sidebar__nav" :aria-label="t('navigation.main')">
      <RouterLink to="/" class="sidebar__link"
        ><Home :size="17" />{{ t("navigation.home") }}</RouterLink
      >
      <RouterLink to="/inbox" class="sidebar__link"
        ><Inbox :size="17" />{{ t("navigation.inbox") }}</RouterLink
      >
      <RouterLink to="/processing" class="sidebar__link">
        <ListTodo :size="17" />
        {{ t("navigation.processing") }}
        <span v-if="application.activeJobs.length" class="sidebar__count">
          {{ application.activeJobs.length }}
        </span>
      </RouterLink>
    </nav>

    <div class="sidebar__section">
      <div class="sidebar__section-label">{{ t("navigation.projects") }}</div>
      <RouterLink
        v-for="project in application.projects"
        :key="project.id"
        :to="`/projects/${project.id}`"
        class="sidebar__link sidebar__link--project"
      >
        <Folder :size="16" />
        <span>{{ project.name }}</span>
      </RouterLink>
      <AppButton
        class="sidebar__new-project"
        variant="ghost"
        size="small"
        @click="openProjectDialog"
      >
        <Plus :size="15" />{{ t("navigation.newProject") }}
      </AppButton>
    </div>

    <div class="sidebar__footer">
      <RouterLink to="/trash" class="sidebar__link"
        ><Trash2 :size="17" />{{ t("navigation.trash") }}</RouterLink
      >
      <RouterLink to="/settings" class="sidebar__link">
        <Settings :size="17" />{{ t("navigation.settings") }}
      </RouterLink>
    </div>

    <AppDialog
      :open="projectDialogOpen"
      :title="t('navigation.newProject')"
      @update:open="projectDialogOpen = $event"
    >
      <form id="create-project-form" @submit.prevent="createProject">
        <label class="dialog-field">
          <span>{{ t("navigation.projectName") }}</span>
          <AppInputText
            v-model="projectName"
            autocomplete="off"
            :placeholder="t('navigation.projectNamePlaceholder')"
            autofocus
            required
          />
        </label>
        <p v-if="projectError" class="dialog-error" role="alert">
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
