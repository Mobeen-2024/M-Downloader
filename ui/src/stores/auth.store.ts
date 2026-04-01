import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface SiteCredential {
  domain: String;
  username?: string;
  password?: string;
  cookies?: string;
}

export const useAuthStore = defineStore('auth', () => {
  const sites = ref<SiteCredential[]>([]);
  const loading = ref(false);

  const fetchSites = async () => {
    loading.value = true;
    try {
      sites.value = await invoke('get_all_site_credentials');
    } catch (e) {
      console.error('Failed to fetch site credentials:', e);
    } finally {
      loading.value = false;
    }
  };

  const addSite = async (cred: SiteCredential) => {
    try {
      await invoke('add_site_credential', { credential: cred });
      await fetchSites();
    } catch (e) {
      console.error('Failed to add site credential:', e);
      throw e;
    }
  };

  const removeSite = async (domain: string) => {
    try {
      await invoke('remove_site_credential', { domain });
      await fetchSites();
    } catch (e) {
      console.error('Failed to remove site credential:', e);
    }
  };

  return {
    sites,
    loading,
    fetchSites,
    addSite,
    removeSite,
  };
});
