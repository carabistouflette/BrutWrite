import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { researchApi, type ResearchArtifact } from '../api/research';
import { APP_CONSTANTS } from '../config/constants';
import { useAppStatus } from '../composables/ui/useAppStatus';

export const useResearchStore = defineStore('research', () => {
  const artifacts = ref<ResearchArtifact[]>([]);
  const activeArtifact = ref<ResearchArtifact | null>(null);
  const { notifyError } = useAppStatus();
  const isLoading = ref(false);

  // Store unlisten function to prevent memory leaks
  let unlistenResearchUpdate: UnlistenFn | null = null;
  let fetchGeneration = 0;

  async function fetchArtifacts() {
    const request = ++fetchGeneration;
    isLoading.value = true;
    try {
      const fetchedArtifacts = await researchApi.fetchArtifacts();
      if (request !== fetchGeneration) return;
      artifacts.value = fetchedArtifacts;
      const activeId = activeArtifact.value?.id;
      activeArtifact.value = activeId
        ? (fetchedArtifacts.find((artifact) => artifact.id === activeId) ?? null)
        : null;
    } catch (error) {
      if (request === fetchGeneration) {
        artifacts.value = [];
        activeArtifact.value = null;
        notifyError('Failed to fetch research artifacts', error);
      }
    } finally {
      if (request === fetchGeneration) {
        isLoading.value = false;
      }
    }
  }

  function setActiveArtifact(artifact: ResearchArtifact | null) {
    activeArtifact.value = artifact;
  }

  async function addFiles(paths: string[]) {
    try {
      await researchApi.addFiles(paths);
    } catch (error) {
      notifyError('Failed to add files', error);
    } finally {
      await fetchArtifacts();
    }
  }

  async function updateTags(id: string, tags: string[]) {
    try {
      await researchApi.updateTags(id, tags);
      const index = artifacts.value.findIndex((artifact) => artifact.id === id);
      if (index !== -1) {
        const updatedArtifact = { ...artifacts.value[index], tags };
        artifacts.value[index] = updatedArtifact;
        if (activeArtifact.value?.id === id) {
          activeArtifact.value = updatedArtifact;
        }
      }
    } catch (error) {
      notifyError('Failed to update research tags', error);
      throw error;
    }
  }

  async function createNote(name: string) {
    try {
      const artifact = await researchApi.createNote(name);
      const index = artifacts.value.findIndex((item) => item.id === artifact.id);
      if (index === -1) {
        artifacts.value.push(artifact);
      } else {
        artifacts.value[index] = artifact;
      }
      setActiveArtifact(artifact);
      return artifact;
    } catch (error) {
      notifyError('Failed to create note', error);
      throw error;
    }
  }

  async function saveNoteContent(id: string, expectedPath: string, content: string) {
    try {
      await researchApi.saveNoteContent(id, expectedPath, content);
    } catch (error) {
      notifyError('Failed to save note content', error);
      throw error;
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
      notifyError('Failed to delete artifact', error);
      throw error;
    }
  }

  // Setup event listener with proper cleanup
  async function setupListener() {
    // Clean up any existing listener first
    if (unlistenResearchUpdate) {
      unlistenResearchUpdate();
    }
    unlistenResearchUpdate = await listen(APP_CONSTANTS.EVENTS.RESEARCH_UPDATE, () => {
      fetchArtifacts();
    });
  }

  // Dispose function to clean up listeners (call when store is no longer needed)
  function dispose() {
    if (unlistenResearchUpdate) {
      unlistenResearchUpdate();
      unlistenResearchUpdate = null;
    }
  }

  // Initialize listener
  setupListener();

  return {
    artifacts,
    activeArtifact,
    isLoading,
    fetchArtifacts,
    setActiveArtifact,
    addFiles,
    updateTags,
    createNote,
    saveNoteContent,
    renameArtifact,
    deleteArtifact,
    dispose,
  };
});
