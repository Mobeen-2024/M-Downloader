<script setup lang="ts">
import { 
  Download, 
  CheckCircle, 
  Clock, 
  Settings, 
  LayoutDashboard 
} from 'lucide-vue-next';
import { useDownloadStore } from '../../stores/download.store';

const store = useDownloadStore();

const navItems = [
  { name: 'Active', icon: Download, path: '/active', badge: () => store.activeDownloads.length },
  { name: 'Completed', icon: CheckCircle, path: '/completed' },
  { name: 'Queued', icon: Clock, path: '/queued' },
  { name: 'Settings', icon: Settings, path: '/settings' },
];
</script>

<template>
  <aside class="sidebar glass-panel">
    <div class="logo">
      <LayoutDashboard class="logo-icon" />
      <h2 class="logo-text">Mdownloader</h2>
    </div>

    <nav class="nav-list">
      <router-link 
        v-for="item in navItems" 
        :key="item.path"
        :to="item.path"
        class="nav-item"
        active-class="active"
      >
        <component :is="item.icon" class="nav-icon" />
        <span class="nav-label">{{ item.name }}</span>
        <span v-if="item.badge && item.badge() > 0" class="badge">
          {{ item.badge() }}
        </span>
      </router-link>
    </nav>

    <div class="sidebar-footer">
      <div class="version-info">
        <span class="text-secondary">v1.0.0 Master</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 260px;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px 16px;
  border-radius: 0;
  border-left: none;
  border-top: none;
  border-bottom: none;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  margin-bottom: 40px;
}

.logo-icon {
  color: var(--accent-primary);
  width: 28px;
  height: 28px;
}

.logo-text {
  font-size: 1.25rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  background: linear-gradient(135deg, var(--text-primary), var(--accent-primary));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.nav-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: var(--transition-smooth);
  font-weight: 500;
  border: 1px solid transparent;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-primary);
}

.nav-item.active {
  background: rgba(59, 130, 246, 0.1);
  color: var(--accent-primary);
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.nav-icon {
  width: 20px;
  height: 20px;
}

.badge {
  margin-left: auto;
  background: var(--accent-primary);
  color: white;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 0.7rem;
  font-weight: 700;
  box-shadow: 0 0 10px rgba(59, 130, 246, 0.4);
}

.sidebar-footer {
  margin-top: auto;
  padding: 16px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 16px;
}

.flex { display: flex; }
.justify-between { justify-content: space-between; }
.text-xs { font-size: 0.75rem; }
.mb-1 { margin-bottom: 4px; }

.progress-bar-mini {
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-primary);
}
</style>
