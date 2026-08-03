import { createRouter, createWebHistory } from "vue-router";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: () => import("@/views/home/HomeView.vue"),
    },
    {
      path: "/inbox",
      name: "inbox",
      component: () => import("@/views/inbox/InboxView.vue"),
    },
    {
      path: "/projects/:projectId",
      name: "project",
      component: () => import("@/views/projects/ProjectView.vue"),
    },
    {
      path: "/sessions/:sessionId",
      name: "session",
      component: () => import("@/views/session/SessionView.vue"),
    },
    {
      path: "/processing",
      name: "processing",
      component: () => import("@/views/processing/ProcessingView.vue"),
    },
    {
      path: "/dictation",
      name: "dictation-history",
      component: () => import("@/views/dictation/DictationHistoryView.vue"),
    },
    {
      path: "/trash",
      name: "trash",
      component: () => import("@/views/trash/TrashView.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/views/settings/SettingsView.vue"),
    },
  ],
});
