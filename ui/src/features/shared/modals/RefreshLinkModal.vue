<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue';
import { 
  PhArrowSquareOut, 
  PhLink, 
  PhWarning, 
  PhCheckCircle, 
  PhGlobe 
} from "@phosphor-icons/vue";
import { listen } from '@tauri-apps/api/event';
import { useDownload } from '@/composables/useDownload';
import BaseDialog from '@/features/shared/components/BaseDialog.vue';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseInput from '@/features/shared/components/BaseInput.vue';

const props = defineProps<{
  show: boolean;
  id: string;
  downloadName: string;
  sourceUrl?: string;
}>();

const emit = defineEmits(['close', 'refresh']);

const { startRefreshMode, cancelRefreshMode, resumeDownload } = useDownload();

const newUrl = ref('');
const error = ref('');
const isWaiting = ref(true);
const captureSuccess = ref(false);
let unlisten: (() => void) | null = null;

const setupListener = async () => {
  if (unlisten) unlisten();
  
  unlisten = await listen('url-refreshed', (event: any) => {
    if (event.payload.id === props.id) {
      isWaiting.value = false;
      captureSuccess.value = true;
      setTimeout(() => {
        emit('close');
        resumeDownload(props.id);
      }, 1500);
    }
  });
};

watch(() => props.show, async (newVal) => {
  if (newVal) {
    isWaiting.value = true;
    captureSuccess.value = false;
    await startRefreshMode(props.id);
    await setupListener();
  } else {
    await cancelRefreshMode();
    if (unlisten) unlisten();
  }
});

onUnmounted(async () => {
  await cancelRefreshMode();
  if (unlisten) unlisten();
});

const handleSubmit = () => {
  if (!newUrl.value) {
    error.value = 'PLEASE ENTER A VALID SOURCE ADDRESS';
    return;
  }
  if (!newUrl.value.startsWith('http')) {
    error.value = 'PROTOCOL IDENTIFIER MISSING';
    return;
  }
  emit('refresh', newUrl.value);
  newUrl.value = '';
};

const openUrl = (url: string) => {
  window.open(url, '_blank');
};
</script>

<template>
  <BaseDialog 
    :show="show" 
    title="Synchronize Transmission Link" 
    size="md" 
    @close="emit('close')"
  >
    <div class="flex flex-col gap-8">
      <!-- Status Banner (Warning) -->
      <Transition 
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
      >
        <div v-if="!captureSuccess" class="flex gap-4 p-5 bg-amber-500/5 border border-amber-500/20 rounded-2xl overflow-hidden relative">
          <div class="absolute inset-0 bg-gradient-to-r from-amber-500/5 to-transparent"></div>
          <PhWarning :size="20" weight="bold" class="text-amber-500 shrink-0 relative z-10" />
          <div class="space-y-1 relative z-10">
            <p class="text-[10px] font-black uppercase text-amber-500 tracking-widest">Link Identity Expired</p>
            <p class="text-[10px] font-bold text-white/50 tracking-widest leading-relaxed uppercase">
              Secure token for <code class="bg-black/40 px-1.5 py-0.5 rounded text-white">{{ downloadName }}</code> has been invalidated by host.
            </p>
          </div>
        </div>
      </Transition>

      <!-- Success Banner -->
      <Transition 
        enter-active-class="transition duration-500 cubic-bezier(0.16, 1, 0.3, 1)"
        enter-from-class="opacity-0 scale-95"
        enter-to-class="opacity-100 scale-100"
      >
        <div v-if="captureSuccess" class="flex gap-4 p-5 bg-emerald-500/10 border border-emerald-500/20 rounded-2xl overflow-hidden relative shadow-[0_0_20px_rgba(16,185,129,0.1)]">
          <PhCheckCircle :size="20" weight="bold" class="text-emerald-500 shrink-0" />
          <div class="space-y-1">
            <p class="text-[10px] font-black uppercase text-emerald-500 tracking-widest">Signal Captured</p>
            <p class="text-[10px] font-bold text-white/50 tracking-widest leading-relaxed uppercase">New transmission parameters intercepted. Resuming stream...</p>
          </div>
        </div>
      </Transition>

      <!-- Interception Area -->
      <section v-if="isWaiting" class="flex flex-col items-center gap-6 p-8 bg-white/[0.02] border border-white/5 rounded-[2rem] shadow-inner">
        <div class="relative w-16 h-16 flex items-center justify-center">
          <div class="absolute inset-0 bg-tactical-cyan/20 rounded-full animate-ping"></div>
          <div class="absolute inset-2 bg-tactical-cyan/40 rounded-full animate-pulse"></div>
          <div class="w-8 h-8 bg-tactical-cyan rounded-full flex items-center justify-center shadow-[0_0_20px_#00f2ff]">
            <PhLink :size="18" weight="bold" class="text-black" />
          </div>
        </div>
        
        <div class="text-center space-y-2">
          <h4 class="text-xs font-black uppercase tracking-[0.2em] text-white">Awaiting Handshake</h4>
          <p class="text-[10px] font-bold text-white/40 tracking-widest leading-relaxed uppercase max-w-[280px]">
            Re-trigger the download in your browser. Fresh address will be captured automatically.
          </p>
        </div>
      </section>

      <!-- Fallback Header -->
      <div v-if="!captureSuccess" class="flex items-center gap-4 py-4">
        <div class="flex-1 h-[1px] bg-white/5"></div>
        <span class="text-[9px] font-black uppercase text-white/20 tracking-[0.3em]">Manual Fallback</span>
        <div class="flex-1 h-[1px] bg-white/5"></div>
      </div>

      <!-- Source Context -->
      <div v-if="sourceUrl && !captureSuccess" class="space-y-3">
        <label class="text-[10px] font-black uppercase text-white/40 tracking-widest pl-1">Discovery Origin</label>
        <div class="flex items-center gap-3 p-4 bg-white/5 border border-white/5 rounded-2xl group hover:bg-white/[0.08] transition-colors">
          <PhGlobe :size="18" weight="duotone" class="text-tactical-cyan" />
          <span class="flex-1 text-[11px] font-data font-black text-white/60 truncate tracking-tight">{{ sourceUrl }}</span>
          <BaseButton variant="ghost" size="icon" @click="openUrl(sourceUrl)" class="!rounded-xl hover:!text-tactical-cyan transition-colors">
            <PhArrowSquareOut :size="14" weight="bold" />
          </BaseButton>
        </div>
      </div>

      <!-- Manual Input -->
      <div v-if="!captureSuccess && !isWaiting" class="space-y-4">
        <BaseInput 
          v-model="newUrl" 
          label="Manual Signal Submission"
          placeholder="PASTE FRESH LINK DESTINATION..." 
          :error="error"
          @keyup.enter="handleSubmit"
        />
      </div>
    </div>

    <template #footer>
      <BaseButton variant="ghost" @click="emit('close')" class="!px-8">ABORT</BaseButton>
      <BaseButton 
        variant="primary" 
        :disabled="captureSuccess || isWaiting"
        @click="handleSubmit"
        class="!px-10 !rounded-2xl"
      >
        SYNCHRONIZE_STREAM
      </BaseButton>
    </template>
  </BaseDialog>
</template>
