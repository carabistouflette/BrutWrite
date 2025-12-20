import { invoke } from '@tauri-apps/api/core';
import type { ProjectMetadata, Manifest } from '../types';

export const projectApi = {
    // Project Management
    create: async (path: string, name: string, author: string): Promise<ProjectMetadata> => {
        return invoke<ProjectMetadata>('create_project', { path, name, author });
    },

    load: async (path: string): Promise<ProjectMetadata> => {
        return invoke<ProjectMetadata>('load_project', { path });
    },

    updateManifest: async (projectId: string, manifest: Manifest): Promise<ProjectMetadata> => {
        return invoke<ProjectMetadata>('update_manifest', { projectId, manifest });
    },

    // Content Management
    loadChapter: async (projectId: string, chapterId: string): Promise<string> => {
        return invoke<string>('load_chapter_content', { projectId, chapterId });
    },

    saveChapter: async (projectId: string, chapterId: string, content: string): Promise<void> => {
        return invoke('save_chapter', { projectId, chapterId, content });
    }
};
