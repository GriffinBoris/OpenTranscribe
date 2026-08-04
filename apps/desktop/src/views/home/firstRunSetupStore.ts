import { ref } from "vue";
import { defineStore } from "pinia";

const FIRST_SETUP_STEP = 1;

export const useFirstRunSetupStore = defineStore("first-run-setup", () => {
  const step = ref(FIRST_SETUP_STEP);

  function reset() {
    step.value = FIRST_SETUP_STEP;
  }

  return {
    step,
    reset,
  };
});
