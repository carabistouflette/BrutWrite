import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface ResearchArtifact {
  id: string;
  path: string;
  name: string;
  file_type: 'pdf' | 'image' | 'text' | 'other';
}

export const useResearchStore = defineStore('research', () => {
  const artifacts = ref<ResearchArtifact[]>([]);
  const activeArtifact = ref<ResearchArtifact | null>(null);
  const isLoading = ref(false);

  async function fetchArtifacts() {
    isLoading.value = true;
    try {
      artifacts.value = await invoke<ResearchArtifact[]>('get_research_artifacts');
    } catch (error) {
      console.error('Failed to fetch research artifacts:', error);
    } finally {
      isLoading.value = false;
    }
  }

  function setActiveArtifact(artifact: ResearchArtifact | null) {
    activeArtifact.value = artifact;
  }

  async function addFiles(paths: string[]) {
    try {
      await invoke('add_research_files', { paths });
      // Watcher should trigger update, but we can force fetch
      await fetchArtifacts();
    } catch (error) {
      console.error('Failed to add files:', error);
    }
  }

  // Listen for backend updates
  listen('research-update', () => {
    fetchArtifacts();
  });

  return {
    artifacts,
    activeArtifact,
    isLoading,
    fetchArtifacts,
    setActiveArtifact,
    addFiles,
  };
});
