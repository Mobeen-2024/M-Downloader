<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAuthStore, type SiteCredential } from '@/stores/auth.store';
import { 
  PhShieldCheck, 
  PhGlobe, 
  PhKey, 
  PhTrash, 
  PhPlus,
  PhCookie
} from "@phosphor-icons/vue";
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseDialog from '@/features/shared/components/BaseDialog.vue';
import BaseInput from '@/features/shared/components/BaseInput.vue';

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
    showModal.value = false;
    newSite.value = { domain: '', username: '', password: '', cookies: '' };
  } catch (e) {
    console.error("Failed to add site:", e);
  }
};
</script>

<template>
  <div class="h-full flex flex-col p-8 md:p-12 gap-10 overflow-y-auto select-none">
    <!-- Header -->
    <header class="flex justify-between items-end shrink-0">
      <div class="space-y-1">
        <h2 class="text-2xl md:text-3xl font-black uppercase tracking-tighter text-white">Security Vault</h2>
        <p class="text-[10px] font-bold text-white/40 tracking-widest uppercase flex items-center gap-2">
          <PhShieldCheck :size="12" class="text-tactical-cyan" />
          Manage credentials and session cookies for restricted domains
        </p>
      </div>
      <BaseButton @click="showModal = true" class="!bg-tactical-cyan !text-black !font-black !px-6 !rounded-xl !transition-transform active:scale-95">
        <template #icon-left><PhPlus :size="16" weight="bold" /></template>
        AUTHORIZE SITE
      </BaseButton>
    </header>

    <!-- Site Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 pb-20">
      <article v-for="site in store.sites" :key="site.domain as string" 
               class="group relative bg-white/5 border border-white/5 rounded-2xl p-6 flex flex-col gap-6 hover:bg-white/[0.08] transition-all duration-300 backdrop-blur-md overflow-hidden">
        
        <!-- Background Accent -->
        <div class="absolute -top-10 -right-10 w-24 h-24 bg-tactical-cyan/5 rounded-full blur-2xl group-hover:bg-tactical-cyan/10 transition-all"></div>

        <div class="flex justify-between items-start z-10">
          <div class="p-3 bg-black/40 border border-white/5 rounded-xl text-tactical-cyan/60 group-hover:text-tactical-cyan transition-colors">
            <PhGlobe :size="24" weight="duotone" />
          </div>
          <button @click="store.removeSite(site.domain as string)" 
                  class="p-2 text-red-500/20 hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-all">
            <PhTrash :size="18" />
          </button>
        </div>

        <div class="space-y-1 z-10">
          <h3 class="text-sm font-black text-white uppercase tracking-tight truncate">{{ site.domain }}</h3>
          <p class="text-[9px] font-bold text-white/20 uppercase tracking-[0.2em]">Authorized_Endpoint</p>
        </div>

        <div class="flex flex-col gap-3 z-10 pt-2">
          <div class="flex items-center justify-between p-3 bg-black/20 border border-white/5 rounded-xl">
            <div class="flex items-center gap-2">
              <PhKey :size="14" weight="duotone" class="text-tactical-cyan/40" />
              <span class="text-[10px] font-black text-white/40 uppercase tracking-widest">Identity</span>
            </div>
            <span v-if="site.username" class="text-[10px] font-data font-black text-tactical-cyan truncate ml-4">{{ site.username }}</span>
            <span v-else class="text-[9px] font-data font-black text-white/10 italic">UNSPECIFIED</span>
          </div>

          <div class="flex items-center justify-between p-3 bg-black/20 border border-white/5 rounded-xl">
            <div class="flex items-center gap-2">
              <PhCookie :size="14" weight="duotone" class="text-tactical-cyan/40" />
              <span class="text-[10px] font-black text-white/40 uppercase tracking-widest">Session</span>
            </div>
            <span v-if="site.cookies" class="text-[9px] font-data font-black text-terminal-green uppercase tracking-widest">Engaged</span>
            <span v-else class="text-[9px] font-data font-black text-white/10 italic">INACTIVE</span>
          </div>
        </div>
      </article>

      <!-- Empty State -->
      <div v-if="store.sites.length === 0" 
           class="col-span-full h-[300px] flex flex-col items-center justify-center border-2 border-dashed border-white/5 rounded-2xl opacity-10 gap-6">
        <PhShieldCheck :size="80" weight="thin" />
        <p class="text-sm font-black uppercase tracking-[0.4em]">Vault_Empty</p>
      </div>
    </div>

    <!-- Add Site Modal -->
    <BaseDialog
      :show="showModal"
      title="Engage Restricted Access"
      @close="showModal = false"
    >
      <div class="space-y-6 py-4">
        <div class="space-y-2">
          <label class="text-[10px] font-black uppercase text-white/30 tracking-widest">Target Domain</label>
          <BaseInput v-model="newSite.domain" placeholder="EXAMPLE.NET" />
        </div>
        
        <div class="grid grid-cols-2 gap-6">
          <div class="space-y-2">
            <label class="text-[10px] font-black uppercase text-white/30 tracking-widest">Identity (Optional)</label>
            <BaseInput v-model="newSite.username" placeholder="USERNAME/ID" />
          </div>
          <div class="space-y-2">
            <label class="text-[10px] font-black uppercase text-white/30 tracking-widest">Security Key (Optional)</label>
            <BaseInput v-model="newSite.password" type="password" placeholder="PASSWORD" />
          </div>
        </div>
        
        <div class="space-y-2">
          <label class="text-[10px] font-black uppercase text-white/30 tracking-widest">Session Payload (Raw Cookies)</label>
          <textarea 
            v-model="newSite.cookies" 
            placeholder="PASTE RAW COOKIE STRING HERE..."
            class="w-full h-32 p-4 bg-black/40 border border-white/10 rounded-xl text-white font-data text-xs outline-none focus:border-tactical-cyan/40 transition-colors resize-none scrollbar-tactical"
          ></textarea>
        </div>
      </div>

      <template #footer>
        <div class="flex gap-4 w-full">
          <BaseButton @click="showModal = false" class="flex-1 !bg-white/5 !text-white/60 !rounded-xl">ABORT</BaseButton>
          <BaseButton @click="handleAddSite" class="flex-[2] !bg-tactical-cyan !text-black !font-black !rounded-xl">ENGAGE AUTHORIZATION</BaseButton>
        </div>
      </template>
    </BaseDialog>
  </div>
</template>

<style scoped>
/* Scoped styles removed in favor of CSS Modules */
</style>
