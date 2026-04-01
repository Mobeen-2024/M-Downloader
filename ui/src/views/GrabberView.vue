<script setup lang="ts">
import { ref, computed } from 'vue';
import { useGrabberStore } from '@/stores/grabber.store';
import { 
  Search, 
  Loader2, 
  CheckCircle2, 
  Download, 
  Box, 
  Video, 
  Image as ImageIcon, 
  Music, 
  FileText,
  FileArchive,
  RefreshCw
} from 'lucide-vue-next';

const store = useGrabberStore();
const urlInput = ref('');
const filterCategory = ref('All');

const filteredAssets = computed(() => {
  if (filterCategory.value === 'All') return store.assets;
  return store.assets.filter(a => a.category === filterCategory.value);
});

const handleCrawl = async () => {
  if (!urlInput.value) return;
  await store.startCrawl(urlInput.value);
};

const handleBulkAdd = async () => {
  await store.bulkAddSelected(true); // Add to queue by default
};

const formatSize = (bytes: number) => {
  if (bytes === 0) return 'Unknown';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const categories = ['All', 'Video', 'Images', 'Music', 'Programs', 'Compressed', 'General'];

const getCategoryIcon = (cat: string) => {
  switch (cat) {
    case 'Video': return Video;
    case 'Images': return ImageIcon;
    case 'Music': return Music;
    case 'Programs': return Box;
    case 'Compressed': return FileArchive;
    case 'Documents': return FileText;
    default: return FileText;
  }
};
</script>

<template>
  <div class="grabber-view">
    <header class="page-header glass-panel">
      <div class="header-content">
        <div class="title-section">
          <div class="icon-box pulse">
            <RefreshCw :size="24" />
          </div>
          <div class="text-info">
            <h1>Universal Site Grabber</h1>
            <p>Crawl domains and bulk-add assets to the Industrial Queue</p>
          </div>
        </div>

        <div class="search-bar">
          <input 
            v-model="urlInput" 
            type="text" 
            placeholder="Enter Target URL (e.g. https://archive.org/...)" 
            @keyup.enter="handleCrawl"
          />
          <button @click="handleCrawl" :disabled="store.isCrawling" class="btn-primary">
            <template v-if="store.isCrawling">
              <Loader2 class="animate-spin" :size="18" />
              Crawling...
            </template>
            <template v-else>
              <Search :size="18" />
              Grab Assets
            </template>
          </button>
        </div>
      </div>
    </header>

    <div class="content-area">
      <aside class="sidebar glass-panel">
        <div class="sidebar-section">
          <h3>Filter Type</h3>
          <div class="category-list">
            <button 
              v-for="cat in categories" 
              :key="cat"
              class="cat-btn"
              :class="{ active: filterCategory === cat }"
              @click="filterCategory = cat"
            >
              <component :is="getCategoryIcon(cat)" :size="16" v-if="cat !== 'All'" />
              <CheckCircle2 :size="16" v-else />
              {{ cat }}
            </button>
          </div>
        </div>

        <div class="sidebar-section selection-summary">
          <h3>Selection</h3>
          <div class="stats-card">
            <div class="stat-item">
              <span>Selected:</span>
              <strong>{{ store.selectedAssets.length }}</strong>
            </div>
            <div class="stat-item">
              <span>Total Found:</span>
              <strong>{{ store.assets.length }}</strong>
            </div>
          </div>
          
          <div class="selection-actions">
            <button @click="store.selectAll(true)" class="btn-ghost">Select All</button>
            <button @click="store.selectAll(false)" class="btn-ghost">Deselect All</button>
          </div>

          <button 
            @click="handleBulkAdd" 
            :disabled="!store.selectedAssets.length" 
            class="btn-bulk-add"
          >
            <Download :size="18" />
            Add to Queue
          </button>
        </div>
      </aside>

      <main class="assets-main glass-panel">
        <div v-if="!store.assets.length && !store.isCrawling" class="empty-state">
          <RefreshCw :size="48" class="pulse opacity-10" />
          <p>No assets identified. Enter a URL to begin crawling.</p>
        </div>

        <div v-else-if="store.isCrawling" class="loading-state">
          <Loader2 class="animate-spin" :size="48" />
          <p>Crawling target domain and resolving file metadata...</p>
        </div>

        <div v-else class="assets-table-container">
          <table class="assets-table">
            <thead>
              <tr>
                <th width="40"></th>
                <th>File Name</th>
                <th>Category</th>
                <th>File Size</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="asset in filteredAssets" :key="asset.url" @click="store.toggleSelection(asset.url)">
                <td>
                  <input type="checkbox" :checked="asset.selected" @click.stop="store.toggleSelection(asset.url)" />
                </td>
                <td class="file-name">
                  <component :is="getCategoryIcon(asset.category)" :size="16" class="type-icon" />
                  {{ asset.filename }}
                </td>
                <td class="category">{{ asset.category }}</td>
                <td class="size">{{ formatSize(asset.size) }}</td>
                <td class="status">
                  <span class="badge">Identified</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
.grabber-view {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
  height: 100%;
}

.page-header {
  padding: 24px;
  border-radius: 20px;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 32px;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.icon-box {
  width: 52px;
  height: 52px;
  border-radius: 14px;
  background: var(--accent-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 0 20px rgba(59, 130, 246, 0.4);
}

.text-info h1 {
  font-size: 1.5rem;
  font-weight: 800;
  margin: 0;
  color: var(--text-primary);
}

.text-info p {
  font-size: 0.85rem;
  margin: 4px 0 0;
  color: var(--text-secondary);
}

.search-bar {
  flex: 1;
  max-width: 600px;
  display: flex;
  gap: 12px;
  background: rgba(255, 255, 255, 0.05);
  padding: 8px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.search-bar input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-primary);
  font-size: 0.95rem;
  padding: 0 12px;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--accent-primary);
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:disabled {
  opacity: 0.7;
}

.content-area {
  flex: 1;
  display: flex;
  gap: 24px;
  min-height: 0;
}

.sidebar {
  width: 280px;
  padding: 24px;
  border-radius: 20px;
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.sidebar-section h3 {
  font-size: 0.75rem;
  text-transform: uppercase;
  color: var(--text-secondary);
  letter-spacing: 0.1em;
  margin-bottom: 16px;
}

.category-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.cat-btn {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-weight: 600;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.cat-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-primary);
}

.cat-btn.active {
  background: var(--accent-primary);
  color: white;
}

.stats-card {
  background: rgba(255, 255, 255, 0.03);
  padding: 16px;
  border-radius: 12px;
  margin-bottom: 16px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
  margin-bottom: 8px;
}

.selection-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.btn-ghost {
  flex: 1;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: var(--text-secondary);
  font-size: 0.75rem;
  padding: 8px;
  border-radius: 8px;
  cursor: pointer;
}

.btn-bulk-add {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 14px;
  background: #10b981;
  color: white;
  border: none;
  border-radius: 12px;
  font-weight: 700;
  font-size: 0.95rem;
  cursor: pointer;
  box-shadow: 0 4px 15px rgba(16, 185, 129, 0.3);
}

.assets-main {
  flex: 1;
  border-radius: 20px;
  padding: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.empty-state, .loading-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.assets-table-container {
  flex: 1;
  overflow-y: auto;
}

.assets-table {
  width: 100%;
  border-collapse: collapse;
}

.assets-table th {
  text-align: left;
  padding: 16px;
  font-size: 0.75rem;
  text-transform: uppercase;
  color: var(--text-secondary);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.assets-table td {
  padding: 16px;
  font-size: 0.9rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.02);
  cursor: pointer;
}

.assets-table tr:hover {
  background: rgba(255, 255, 255, 0.02);
}

.file-name {
  display: flex;
  align-items: center;
  gap: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.type-icon {
  color: var(--accent-primary);
}

.badge {
  padding: 4px 8px;
  background: rgba(59, 130, 246, 0.1);
  color: var(--accent-primary);
  border-radius: 6px;
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.opacity-10 { opacity: 0.1; }

.pulse {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(1); }
  50% { transform: scale(0.95); opacity: 0.8; }
  100% { transform: scale(1); }
}
</style>
