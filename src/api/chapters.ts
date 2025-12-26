import { invoke } from '@tauri-apps/api/core';
import type { ProjectMetadata, Chapter } from '../types';

export const chaptersApi = {
  loadContent: async (projectId: string, chapterId: string): Promise<string> => {
    return invoke<string>('load_chapter_content', { projectId, chapterId });
  },

  saveContent: async (
    projectId: string,
    chapterId: string,
    content: string
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>('save_chapter', {
      projectId,
      chapterId,
      content,
    });
  },

  createNode: async (
    projectId: string,
    parentId: string | undefined,
    name: string
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>('create_node', {
      projectId,
      parentId,
      name,
    });
  },

  deleteNode: async (projectId: string, id: string): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>('delete_node', { projectId, id });
  },

  updateNodeMetadata: async (
    projectId: string,
    nodeId: string,
    update: Partial<Chapter>
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>('update_node_metadata', {
      projectId,
      nodeId,
      update,
    });
  },
};
