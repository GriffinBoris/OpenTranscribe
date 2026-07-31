import { ref } from "vue";
import { defineStore } from "pinia";

import { native } from "@/core/native";
import type { CredentialStatus } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";

export const useOpenAiStore = defineStore("openAi", () => {
  const application = useApplicationStore();
  const credential = ref<CredentialStatus | null>(null);
  const connectionMessage = ref<string | null>(null);

  async function loadCredential() {
    application.operationError = null;

    try {
      credential.value = await native.openAiCredentialStatus();
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function saveApiKey(apiKey: string) {
    application.operationError = null;

    try {
      credential.value = await native.saveOpenAiApiKey(apiKey);
      connectionMessage.value = null;
      return true;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
      return false;
    }
  }

  async function removeApiKey() {
    application.operationError = null;

    try {
      credential.value = await native.removeOpenAiApiKey();
      connectionMessage.value = null;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  async function testConnection() {
    application.operationError = null;

    try {
      const result = await native.testOpenAiConnection();
      connectionMessage.value = result.message;
    } catch (reason) {
      application.operationError =
        reason instanceof Error ? reason.message : String(reason);
    }
  }

  return {
    credential,
    connectionMessage,
    loadCredential,
    saveApiKey,
    removeApiKey,
    testConnection,
  };
});
