import { createRouter, createWebHashHistory } from "vue-router";
import ActiveView from "../views/ActiveView.vue";

const routes = [
  { path: "/", redirect: "/active" },
  {
    path: "/active",
    name: "Active",
    component: ActiveView,
  },
  {
    path: "/completed",
    name: "Completed",
    component: () => import("../views/CompletedView.vue"),
  },
  {
    path: "/queued",
    name: "Queued",
    component: () => import("../views/QueuedView.vue"),
  },
  {
    path: "/settings",
    name: "Settings",
    component: () => import("../views/SettingsView.vue"),
  },
  {
    path: "/stats",
    name: "Statistics",
    component: () => import("../views/StatsView.vue"),
  },
  {
    path: "/sites",
    name: "Sites",
    component: () => import("../views/SitesView.vue"),
  },
  {
    path: "/grabber",
    component: () => import("../views/GrabberView.vue"),
  },
  {
    path: "/cloud-drive",
    name: "Cloud Drive",
    component: () => import("../views/CloudDriveView.vue"),
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
