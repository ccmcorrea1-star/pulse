import { createRouter, createWebHistory } from "vue-router";

import DeviceSectionView from "@/views/DeviceSectionView.vue";
import DeviceView from "@/views/DeviceView.vue";
import HistoryView from "@/views/HistoryView.vue";
import HomeView from "@/views/HomeView.vue";
import SettingsView from "@/views/SettingsView.vue";
import TransfersView from "@/views/TransfersView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    { path: "/transfers", name: "transfers", component: TransfersView },
    { path: "/history", name: "history", component: HistoryView },
    {
      path: "/device/:id",
      component: DeviceView,
      children: [
        { path: "", redirect: { name: "device-overview" } },
        {
          path: "overview",
          name: "device-overview",
          component: DeviceSectionView,
          meta: { title: "Visão geral", description: "Ponto de entrada para o estado do dispositivo." },
        },
        {
          path: "files",
          name: "device-files",
          component: DeviceSectionView,
          meta: { title: "Arquivos", description: "Espaço reservado para a futura experiência de arquivos." },
        },
        {
          path: "clipboard",
          name: "device-clipboard",
          component: DeviceSectionView,
          meta: { title: "Clipboard", description: "Espaço reservado para a futura experiência de Clipboard." },
        },
        {
          path: "media",
          name: "device-media",
          component: DeviceSectionView,
          meta: { title: "Mídia", description: "Espaço reservado para a futura experiência de mídia." },
        },
        {
          path: "control",
          name: "device-control",
          component: DeviceSectionView,
          meta: { title: "Controle", description: "Espaço reservado para a futura experiência de controle." },
        },
      ],
    },
    { path: "/settings", name: "settings", component: SettingsView },
  ],
});

export default router;
