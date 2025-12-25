import { invoke } from '@tauri-apps/api/core';

export interface ResearchArtifact {
  id: string;
  path: string;
  name: string;
  file_type: 'pdf' | 'image' | 'text' | 'other';
  tags: string[];
}

export const researchApi = {
  fetchArtifacts: async (): Promise<ResearchArtifact[]> => {
    return invoke<ResearchArtifact[]>('get_research_artifacts');
  },

  addFiles: async (paths: string[]): Promise<void> => {
    return invoke('add_research_files', { paths });
  },

  updateArtifact: async (artifact: ResearchArtifact): Promise<void> => {
    return invoke('update_research_artifact', { artifact });
  },

  createNote: async (name: string): Promise<ResearchArtifact> => {
    return invoke<ResearchArtifact>('create_research_note', { name });
  },

  saveNoteContent: async (id: string, content: string): Promise<void> => {
    return invoke('update_note_content', { id, content });
  },

  renameArtifact: async (id: string, newName: string): Promise<void> => {
    return invoke('rename_research_artifact', { id, newName });
  },

  deleteArtifact: async (id: string): Promise<void> => {
    return invoke('delete_research_artifact', { id });
  },
};
