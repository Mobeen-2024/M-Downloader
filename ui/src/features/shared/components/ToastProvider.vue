<script setup lang="ts">
import { useUIStore } from '@/stores/ui.store';
import { 
  PhCheckCircle, 
  PhWarningOctagon, 
  PhInfo, 
  PhWarning, 
  PhX 
} from "@phosphor-icons/vue";

const ui = useUIStore();

const variantStyles = {
  success: 'text-emerald-400 bg-emerald-400/5 border-emerald-400/20 shadow-[0_0_20px_rgba(16,185,129,0.1)]',
  error: 'text-red-500 bg-red-500/5 border-red-500/20 shadow-[0_0_20px_rgba(239,68,68,0.1)]',
  warning: 'text-amber-500 bg-amber-500/5 border-amber-500/20 shadow-[0_0_20px_rgba(245,158,11,0.1)]',
  info: 'text-tactical-cyan bg-tactical-cyan/5 border-tactical-cyan/20 shadow-[0_0_20px_rgba(0,242,255,0.1)]',
};

const getIcon = (type: string) => {
  switch (type) {
    case 'success': return PhCheckCircle;
    case 'error': return PhWarningOctagon;
    case 'warning': return PhWarning;
    default: return PhInfo;
  }
};
</script>

<template>
  <div class="fixed bottom-12 left-1/2 -translate-x-1/2 z-[200] flex flex-col gap-3 min-w-[340px] max-w-[90vw] pointer-events-none">
    <TransitionGroup 
      enter-active-class="transition duration-500 cubic-bezier(0.17, 0.67, 0.83, 0.67)"
      enter-from-class="opacity-0 translate-y-8 scale-90"
      enter-to-class="opacity-100 translate-y-0 scale-100"
      leave-active-class="transition duration-300 ease-in absolute w-full"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
      move-class="transition duration-500 ease-out"
    >
      <div 
        v-for="toast in ui.toasts" 
        :key="toast.id" 
        class="pointer-events-auto px-6 py-4 bg-[#0a0a0a]/95 backdrop-blur-3xl border rounded-2xl flex items-center gap-4 relative overflow-hidden group shadow-[0_12px_40px_rgba(0,0,0,0.6)]"
        :class="variantStyles[toast.type as keyof typeof variantStyles] || variantStyles.info"
      >
        <div class="w-8 h-8 rounded-xl bg-current opacity-10 absolute -left-1 -top-1 blur-xl"></div>
        
        <component 
          :is="getIcon(toast.type)" 
          :size="20" 
          weight="duotone" 
          class="shrink-0 animate-pulse" 
        />
        
        <span class="flex-1 text-[10px] font-black uppercase tracking-[0.15em] leading-tight text-white/90">
          {{ toast.message }}
        </span>

        <button 
          @click="ui.removeToast(toast.id)" 
          class="text-white/20 hover:text-white hover:bg-white/5 p-1 rounded-lg transition-all active:scale-95"
        >
          <PhX :size="14" weight="bold" />
        </button>

        <!-- Optional progress bar for auto-dismissible toasts -->
        <div class="absolute bottom-0 left-0 h-[2px] bg-current opacity-20 w-full origin-left animate-[shrink-x_5s_linear_forwards]"></div>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
@keyframes shrink-x {
  from { transform: scaleX(1); }
  to { transform: scaleX(0); }
}
</style>
