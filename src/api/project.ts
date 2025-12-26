import { invoke } from '@tauri-apps/api/core';
import type { ProjectMetadata, Manifest, ProjectSettings, Plotline } from '../types';

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

  updateSettings: async (
    projectId: string,
    settings: ProjectSettings
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>('update_project_settings', {
      projectId,
      settings,
    });
  },

  updatePlotlines: async (projectId: string, plotlines: Plotline[]): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>('update_plotlines', {
      projectId,
      plotlines,
    });
  },
};
