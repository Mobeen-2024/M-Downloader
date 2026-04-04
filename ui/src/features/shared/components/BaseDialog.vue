<script setup lang="ts">
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
} from 'radix-vue';
import { PhX } from "@phosphor-icons/vue";
import BaseButton from './BaseButton.vue';

interface Props {
  show: boolean;
  title?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  closable?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  closable: true
});

const emit = defineEmits(['close']);

const updateShow = (val: boolean) => {
  if (!val) emit('close');
};

const sizeClasses = {
  sm: 'max-w-[400px]',
  md: 'max-w-[560px]',
  lg: 'max-w-[800px]',
  xl: 'max-w-[1100px]'
};
</script>

<template>
  <DialogRoot :open="show" @update:open="updateShow">
    <DialogPortal>
      <Transition 
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition duration-200 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <DialogOverlay class="fixed inset-0 bg-black/80 backdrop-blur-sm z-[100]" />
      </Transition>
      
      <Transition 
        enter-active-class="transition duration-500 cubic-bezier(0.16, 1, 0.3, 1)"
        enter-from-class="opacity-0 scale-95 translate-y-8"
        enter-to-class="opacity-100 scale-100 translate-y-0"
        leave-active-class="transition duration-300 ease-in"
        leave-from-class="opacity-100 scale-100 translate-y-0"
        leave-to-class="opacity-0 scale-95 translate-y-4"
      >
        <DialogContent 
          class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[calc(100%-2rem)] bg-[#050505] border border-white/10 rounded-[2.5rem] shadow-[0_0_80px_rgba(0,0,0,0.8)] z-[101] overflow-hidden flex flex-col focus:outline-none"
          :class="sizeClasses[size]"
        >
          <header class="px-10 py-8 border-b border-white/5 flex justify-between items-center bg-white/[0.01]">
            <div class="space-y-1">
              <h3 class="text-xl font-black uppercase tracking-tight text-white">{{ title }}</h3>
              <div class="h-[2px] w-8 bg-tactical-cyan rounded-full"></div>
            </div>
            
            <DialogClose as-child v-if="closable">
              <BaseButton 
                variant="ghost" 
                size="icon" 
                class="hover:!text-red-500 !bg-white/5 group transition-colors"
                aria-label="Close"
              >
                <PhX :size="18" weight="bold" class="group-hover:rotate-90 transition-transform duration-300" />
              </BaseButton>
            </DialogClose>
          </header>

          <div class="px-10 py-10 overflow-y-auto max-h-[70vh] custom-scrollbar">
            <slot></slot>
          </div>

          <footer v-if="$slots.footer" class="px-10 py-8 border-t border-white/5 bg-white/[0.01] flex justify-end gap-4">
            <slot name="footer"></slot>
          </footer>
        </DialogContent>
      </Transition>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 242, 255, 0.2);
}
</style>
