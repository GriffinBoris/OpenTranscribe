import { createI18n } from "vue-i18n";

import { messages } from "@/locales/en";

export const i18n = createI18n({
  legacy: false,
  locale: "en",
  fallbackLocale: "en",
  messages: {
    en: messages,
  },
});
