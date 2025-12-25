import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { researchApi, type ResearchArtifact } from '../api/research';
import { APP_CONSTANTS } from '../config/constants';
import { useAppStatus } from '../composables/ui/useAppStatus';

export const useResearchStore = defineStore('research', () => {
  const artifacts = ref<ResearchArtifact[]>([]);
  const activeArtifact = ref<ResearchArtifact | null>(null);
  const { notifyError } = useAppStatus();
  const isLoading = ref(false);

  async function fetchArtifacts() {
    isLoading.value = true;
    try {
      artifacts.value = await researchApi.fetchArtifacts();
    } catch (error) {
      notifyError('Failed to fetch research artifacts', error);
    } finally {
      isLoading.value = false;
    }
  }

  function setActiveArtifact(artifact: ResearchArtifact | null) {
    activeArtifact.value = artifact;
  }

  async function addFiles(paths: string[]) {
    try {
      await researchApi.addFiles(paths);
      // Watcher should trigger update, but we can force fetch
      await fetchArtifacts();
    } catch (error) {
      notifyError('Failed to add files', error);
    }
  }

  async function updateArtifact(artifact: ResearchArtifact) {
    try {
      await researchApi.updateArtifact(artifact);
      // Optimistically update local state
      const index = artifacts.value.findIndex((a) => a.id === artifact.id);
      if (index !== -1) {
        artifacts.value[index] = artifact;
      }
      if (activeArtifact.value?.id === artifact.id) {
        activeArtifact.value = artifact;
      }
    } catch (error) {
      notifyError('Failed to update artifact', error);
    }
  }

  async function createNote(name: string) {
    try {
      const artifact = await researchApi.createNote(name);
      artifacts.value.push(artifact);
      setActiveArtifact(artifact);
      return artifact;
    } catch (error) {
      notifyError('Failed to create note', error);
      throw error;
    }
  }

  async function saveNoteContent(id: string, content: string) {
    try {
      await researchApi.saveNoteContent(id, content);
    } catch (error) {
      notifyError('Failed to save note content', error);
    }
  }

  async function renameArtifact(id: string, newName: string) {
    try {
      await researchApi.renameArtifact(id, newName);
      await fetchArtifacts();
    } catch (error) {
      notifyError('Failed to rename artifact', error);
      throw error;
    }
  }

  async function deleteArtifact(id: string) {
    try {
      await researchApi.deleteArtifact(id);
      artifacts.value = artifacts.value.filter((a) => a.id !== id);
      if (activeArtifact.value?.id === id) {
        activeArtifact.value = null;
      }
    } catch (error) {
      console.error('Failed to delete artifact:', error);
      throw error;
    }
  }

  // Listen for backend updates
  listen(APP_CONSTANTS.EVENTS.RESEARCH_UPDATE, () => {
    fetchArtifacts();
  });

  return {
    artifacts,
    activeArtifact,
    isLoading,
    fetchArtifacts,
    setActiveArtifact,
    addFiles,
    updateArtifact,
    createNote,
    saveNoteContent,
    renameArtifact,
    deleteArtifact,
  };
});
