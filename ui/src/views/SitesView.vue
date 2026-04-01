<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAuthStore, type SiteCredential } from '@/stores/auth.store';
import { ShieldCheck, Globe, Key, Trash2, Plus, X } from 'lucide-vue-next';

const store = useAuthStore();
const showModal = ref(false);
const newSite = ref<SiteCredential>({ domain: '', username: '', password: '', cookies: '' });

onMounted(() => {
  store.fetchSites();
});

const handleAddSite = async () => {
  if (!newSite.value.domain) return;
  try {
    await store.addSite({ ...newSite.value });
    showSiteModal(false);
    newSite.value = { domain: '', username: '', password: '', cookies: '' };
  } catch (e) {
    alert("Failed to add site. Ensure domain is valid.");
  }
};

const showSiteModal = (val: boolean) => {
  showModal.value = val;
};
</script>

<template>
  <div class="view-container">
    <div class="header-section">
      <div class="title-group">
        <ShieldCheck class="header-icon" />
        <div>
          <h1>Site Manager</h1>
          <p>Manage credentials and cookies for restricted domains.</p>
        </div>
      </div>
      <button @click="showSiteModal(true)" class="add-btn">
        <Plus :size="18" />
        Add Site
      </button>
    </div>

    <div class="glass-panel sites-table-container">
      <table class="sites-table">
        <thead>
          <tr>
            <th>Domain</th>
            <th>Authentication</th>
            <th>Cookies</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="site in store.sites" :key="site.domain as string">
            <td class="domain-cell">
              <Globe :size="16" class="cell-icon" />
              {{ site.domain }}
            </td>
            <td>
              <div v-if="site.username" class="auth-badge">
                <Key :size="12" />
                {{ site.username }}
              </div>
              <span v-else class="text-muted">None</span>
            </td>
            <td>
              <span v-if="site.cookies" class="cookie-preview" :title="site.cookies as string">
                Active Session
              </span>
              <span v-else class="text-muted">None</span>
            </td>
            <td>
              <button @click="store.removeSite(site.domain as string)" class="delete-btn">
                <Trash2 :size="16" />
              </button>
            </td>
          </tr>
          <tr v-if="store.sites.length === 0">
            <td colspan="4" class="empty-row">No site credentials stored.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Add Site Modal -->
    <div v-if="showModal" class="modal-overlay" @click.self="showSiteModal(false)">
      <div class="modal-content glass-panel">
        <div class="modal-header">
          <h3>Add Restricted Domain</h3>
          <button @click="showSiteModal(false)" class="close-btn"><X :size="20" /></button>
        </div>
        
        <div class="modal-body">
          <div class="input-group">
            <label>Domain Name</label>
            <input v-model="newSite.domain" placeholder="example.com" />
          </div>
          
          <div class="row">
            <div class="input-group">
              <label>Username (Optional)</label>
              <input v-model="newSite.username" placeholder="Optional" />
            </div>
            <div class="input-group">
              <label>Password (Optional)</label>
              <input v-model="newSite.password" type="password" placeholder="Optional" />
            </div>
          </div>
          
          <div class="input-group">
            <label>Cookies (Optional)</label>
            <textarea v-model="newSite.cookies" placeholder="Raw cookie string..."></textarea>
          </div>
        </div>

        <div class="modal-footer">
          <button @click="showSiteModal(false)" class="btn-cancel">Cancel</button>
          <button @click="handleAddSite" class="btn-save">Save Credential</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.view-container {
  padding: 32px;
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}

.title-group {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon {
  color: var(--accent-primary);
  width: 32px;
  height: 32px;
}

.title-group h1 {
  font-size: 1.8rem;
  font-weight: 800;
  margin: 0;
}

.title-group p {
  color: var(--text-secondary);
  font-size: 0.9rem;
  margin: 4px 0 0 0;
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: var(--accent-primary);
  color: white;
  border: none;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: var(--transition-smooth);
}

.add-btn:hover {
  transform: translateY(-2px);
  filter: brightness(1.1);
}

.sites-table-container {
  border-radius: 20px;
  overflow: hidden;
}

.sites-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
}

.sites-table th {
  padding: 16px 24px;
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--text-secondary);
  text-transform: uppercase;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.sites-table td {
  padding: 18px 24px;
  font-size: 0.9rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.domain-cell {
  display: flex;
  align-items: center;
  gap: 12px;
  font-weight: 600;
}

.cell-icon {
  color: var(--accent-primary);
}

.auth-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
}

.cookie-preview {
  color: var(--accent-primary);
  font-weight: 600;
  cursor: help;
}

.delete-btn {
  background: transparent;
  border: none;
  color: #ef4444;
  cursor: pointer;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s;
}

.delete-btn:hover {
  background: rgba(239, 68, 68, 0.1);
}

.empty-row {
  text-align: center;
  color: var(--text-secondary);
  padding: 48px !important;
}

.text-muted {
  color: var(--text-secondary);
  opacity: 0.5;
}

/* Modal Styles */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.4);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  width: 100%;
  max-width: 500px;
  border-radius: 24px;
  padding: 32px;
  animation: modalScale 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes modalScale {
  from { transform: scale(0.9); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.row {
  display: flex;
  gap: 16px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.input-group label {
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--text-secondary);
}

.input-group input, .input-group textarea {
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  color: white;
  font-size: 0.9rem;
}

.input-group textarea {
  height: 100px;
  resize: none;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 16px;
  margin-top: 32px;
}

.btn-cancel {
  padding: 10px 20px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-weight: 700;
  cursor: pointer;
}

.btn-save {
  padding: 10px 24px;
  background: var(--accent-primary);
  color: white;
  border: none;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
}
</style>
