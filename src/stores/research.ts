import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface ResearchArtifact {
  id: string;
  path: string;
  name: string;
  file_type: 'pdf' | 'image' | 'text' | 'other';
  tags: string[];
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

  async function updateArtifact(artifact: ResearchArtifact) {
    try {
      await invoke('update_research_artifact', { artifact });
      // Optimistically update local state
      const index = artifacts.value.findIndex((a) => a.id === artifact.id);
      if (index !== -1) {
        artifacts.value[index] = artifact;
      }
      if (activeArtifact.value?.id === artifact.id) {
        activeArtifact.value = artifact;
      }
    } catch (error) {
      console.error('Failed to update artifact:', error);
    }
  }

  async function createNote(name: string) {
    try {
      const artifact = await invoke<ResearchArtifact>('create_research_note', { name });
      artifacts.value.push(artifact);
      setActiveArtifact(artifact);
      return artifact;
    } catch (error) {
      console.error('Failed to create note:', error);
      throw error;
    }
  }

  async function saveNoteContent(id: string, content: string) {
    try {
      await invoke('update_note_content', { id, content });
    } catch (error) {
      console.error('Failed to save note content:', error);
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
    updateArtifact,
    createNote,
    saveNoteContent,
  };
});
