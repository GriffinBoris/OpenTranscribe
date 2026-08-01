import { createApp } from "vue";
import { createPinia } from "pinia";
import PrimeVue from "primevue/config";

import App from "@/App.vue";
import { i18n } from "@/i18n";
import { router } from "@/router";
import "@/styles/main.css";

const app = createApp(App);

app.use(createPinia());
app.use(router);
app.use(PrimeVue, { unstyled: true });
app.use(i18n);

app.mount("#app");
