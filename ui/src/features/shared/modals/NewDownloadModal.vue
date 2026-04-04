<script setup lang="ts">
import { ref, computed } from 'vue';
import { PhDownloadSimple, PhInfo } from "@phosphor-icons/vue";
import { useDownload } from '@/composables/useDownload';
import BaseDialog from '@/features/shared/components/BaseDialog.vue';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseInput from '@/features/shared/components/BaseInput.vue';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits(['close']);

const url = ref('https://releases.ubuntu.com/22.04/ubuntu-22.04.3-desktop-amd64.iso');
const { startDownload } = useDownload();
const isSubmitting = ref(false);
const error = ref('');

const isValidUrl = computed(() => {
  try {
    new URL(url.value);
    return true;
  } catch {
    return false;
  }
});

const fileName = computed(() => {
  if (!isValidUrl.value) return '';
  return url.value.split('/').pop() || 'UNKNOWN_BINARY_BLOB';
});

const handleStart = async () => {
  if (!isValidUrl.value || isSubmitting.value) return;
  
  isSubmitting.value = true;
  error.value = '';
  
  try {
    await startDownload(url.value);
    emit('close');
  } catch (e: any) {
    error.value = (typeof e === 'string' ? e : e?.message) || 'TRANSMISSION_INITIALIZATION_FAILED';
  } finally {
    isSubmitting.value = false;
  }
};
</script>

<template>
  <BaseDialog 
    :show="show" 
    title="Initialize Tactical Download" 
    size="md" 
    @close="$emit('close')"
  >
    <div class="flex flex-col gap-8">
      <!-- Input Section -->
      <section class="space-y-4">
        <BaseInput 
          v-model="url" 
          label="Target Resource URL"
          placeholder="ENTER HIGH-SPEED SOURCE URL..." 
          :error="error"
          autofocus
          @keyup.enter="handleStart"
        >
          <template #icon-left><PhDownloadSimple :size="18" weight="duotone" /></template>
        </BaseInput>
      </section>

      <!-- Metadata Preview -->
      <Transition 
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
      >
        <div v-if="isValidUrl" class="bg-white/5 border border-white/5 rounded-2xl p-6 space-y-4 shadow-inner">
          <div class="flex justify-between items-center">
            <span class="text-[10px] font-black uppercase text-white/30 tracking-[0.2em]">Calculated Filename</span>
            <span class="text-xs font-data font-black text-tactical-cyan truncate max-w-[240px]">{{ fileName }}</span>
          </div>
          <div class="h-[1px] bg-white/5 w-full"></div>
          <div class="flex justify-between items-center">
            <span class="text-[10px] font-black uppercase text-white/30 tracking-[0.2em]">Engine Strategy</span>
            <span class="text-[10px] font-black text-white/60 tracking-widest uppercase italic">Multi_Segment_Adaptive_Split</span>
          </div>
        </div>
      </Transition>

      <!-- Tactical Notice -->
      <div class="flex gap-4 p-5 bg-tactical-cyan/5 border border-tactical-cyan/10 rounded-2xl overflow-hidden relative group">
        <div class="absolute inset-0 bg-tactical-cyan/5 opacity-0 group-hover:opacity-100 transition-opacity"></div>
        <PhInfo :size="20" weight="duotone" class="text-tactical-cyan shrink-0" />
        <p class="text-[10px] font-bold text-white/50 tracking-widest leading-relaxed uppercase">
          Resource processing uses <strong class="text-white">Segmented Byte-Range Retrieval</strong>, dynamically splitting payloads for maximum throughput efficiency.
        </p>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="ghost" @click="$emit('close')" class="!px-8">ABORT</BaseButton>
      <BaseButton 
        variant="primary" 
        :disabled="!isValidUrl || isSubmitting"
        :loading="isSubmitting"
        @click="handleStart"
        class="!px-10 !rounded-2xl"
      >
        <template #icon-left v-if="!isSubmitting"><PhDownloadSimple :size="20" weight="bold" /></template>
        ENGAGE TRANSMISSION
      </BaseButton>
    </template>
  </BaseDialog>
</template>
