import { createPinia } from "pinia";
import { createApp } from "vue";

import App from "./App.vue";
import router from "./router";
import "./styles/index.css";
import { useAppStore } from "@/stores/app";

const pinia = createPinia();
const app = createApp(App);

app.use(pinia).use(router);
void useAppStore(pinia).initialize();
app.mount("#app");
