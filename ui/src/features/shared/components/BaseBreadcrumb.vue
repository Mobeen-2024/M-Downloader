<script setup lang="ts">
import { PhCaretRight, PhHouseSimple } from "@phosphor-icons/vue";

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
  if (index === props.items.length - 1) return;
  emit('navigate', item);
};
</script>

<template>
  <nav class="flex items-center select-none">
    <ol class="flex items-center flex-wrap gap-y-2">
      <!-- Root / Home link -->
      <li class="flex items-center">
        <button 
          @click="emit('navigate', { label: 'Root', id: 'root' })" 
          class="flex items-center justify-center p-1.5 rounded-lg bg-white/5 border border-white/5 text-tactical-cyan/40 hover:text-tactical-cyan hover:bg-white/10 transition-all active:scale-95"
        >
          <PhHouseSimple :size="14" weight="duotone" />
        </button>
      </li>
      
      <!-- Path items -->
      <li v-for="(item, index) in items" :key="item.id" class="flex items-center">
        <PhCaretRight :size="10" weight="bold" class="text-white/10 mx-2" />
        
        <button 
          @click="handleClick(item, index)" 
          class="text-[10px] font-black uppercase tracking-[0.2em] transition-all"
          :class="[
            index === items.length - 1 
              ? 'text-white cursor-default' 
              : 'text-white/30 hover:text-white cursor-pointer active:scale-95'
          ]"
          :disabled="index === items.length - 1"
        >
          {{ item.label }}
        </button>
      </li>
    </ol>
  </nav>
</template>
