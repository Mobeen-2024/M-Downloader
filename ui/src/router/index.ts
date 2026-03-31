import { createRouter, createWebHashHistory } from 'vue-router';
import ActiveView from '../views/ActiveView.vue';

const routes = [
  { path: '/', redirect: '/active' },
  { 
    path: '/active', 
    name: 'Active',
    component: ActiveView 
  },
  { 
    path: '/completed', 
    name: 'Completed',
    component: () => import('../views/CompletedView.vue') 
  },
  { 
    path: '/queued', 
    name: 'Queued',
    component: () => import('../views/QueuedView.vue') 
  },
  { 
    path: '/settings', 
    name: 'Settings',
    component: () => import('../views/SettingsView.vue') 
  },
  { 
    path: '/stats', 
    name: 'Statistics',
    component: () => import('../views/StatsView.vue') 
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
