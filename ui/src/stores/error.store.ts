import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useUIStore } from './ui.store';

export interface AppError {
  id: string;
  message: string;
  context?: string;
  source: 'ipc' | 'ui' | 'network' | 'unknown';
  timestamp: number;
}

export const useErrorStore = defineStore('errors', () => {
  const errors = ref<AppError[]>([]);
  const hasCriticalError = ref(false);
  const uiStore = useUIStore();

  const logError = (message: string, source: AppError['source'] = 'unknown', context?: string) => {
    const error: AppError = {
      id: crypto.randomUUID(),
      message,
      source,
      context,
      timestamp: Date.now()
    };
    
    errors.value.unshift(error);
    
    // Trigger visible UI toast
    uiStore.error(`[${source.toUpperCase()}] ${message}`);
    
    // Keep only the last 50 errors in memory
    if (errors.value.length > 50) {
      errors.value.pop();
    }
  };

  const setCriticalError = (state: boolean) => {
    hasCriticalError.value = state;
  };

  const clearErrors = () => {
    errors.value = [];
    hasCriticalError.value = false;
  };

  return {
    errors,
    hasCriticalError,
    logError,
    setCriticalError,
    clearErrors
  };
});

