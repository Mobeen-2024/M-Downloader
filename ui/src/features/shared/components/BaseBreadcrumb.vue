<script setup lang="ts">
import { ChevronRight, Home } from 'lucide-vue-next';

interface BreadcrumbItem {
  label: string;
  id: string | number;
}

interface Props {
  items: BreadcrumbItem[];
}

const props = defineProps<Props>();
const emit = defineEmits(['navigate']);

const handleClick = (item: BreadcrumbItem, index: number) => {
  // Don't emit for the last item (current directory)
  if (index === props.items.length - 1) return;
  emit('navigate', item);
};
</script>

<template>
  <nav class="base-breadcrumb glass-panel">
    <ol class="breadcrumb-list">
      <li class="breadcrumb-item home">
        <button @click="emit('navigate', { label: 'Root', id: 'root' })" class="breadcrumb-btn home-btn">
          <Home :size="14" />
        </button>
      </li>
      
      <li v-for="(item, index) in items" :key="item.id" class="breadcrumb-item">
        <ChevronRight :size="14" class="separator" />
        <button 
          @click="handleClick(item, index)" 
          class="breadcrumb-btn"
          :class="{ 'is-current': index === items.length - 1 }"
          :disabled="index === items.length - 1"
        >
          {{ item.label }}
        </button>
      </li>
    </ol>
  </nav>
</template>

<style scoped>
.base-breadcrumb {
  display: inline-flex;
  padding: 6px 16px;
  border-radius: var(--radius-md);
  margin-bottom: 20px;
}

.breadcrumb-list {
  display: flex;
  align-items: center;
  list-style: none;
  padding: 0;
  margin: 0;
  gap: 4px;
}

.breadcrumb-item {
  display: flex;
  align-items: center;
}

.breadcrumb-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 0.85rem;
  font-weight: 500;
  padding: 4px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.breadcrumb-btn:not(:disabled):hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--accent-primary);
}

.breadcrumb-btn.is-current {
  color: var(--text-primary);
  font-weight: 700;
  cursor: default;
}

.home-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px;
}

.separator {
  color: var(--text-secondary);
  opacity: 0.4;
  margin: 0 2px;
}

.home {
  display: flex;
  align-items: center;
}
</style>
