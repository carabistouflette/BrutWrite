import { invoke } from "@tauri-apps/api/core";
import type {
  ProjectMetadata,
  Manifest,
  Character,
  ProjectSettings,
  Plotline,
  Chapter,
} from "../types";

export const projectApi = {
  // Project Management
  create: async (
    path: string,
    name: string,
    author: string
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("create_project", { path, name, author });
  },

  load: async (path: string): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("load_project", { path });
  },

  updateManifest: async (
    projectId: string,
    manifest: Manifest
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("update_manifest", { projectId, manifest });
  },

  updateSettings: async (
    projectId: string,
    settings: ProjectSettings
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("update_project_settings", {
      projectId,
      settings,
    });
  },

  updatePlotlines: async (
    projectId: string,
    plotlines: Plotline[]
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("update_plotlines", {
      projectId,
      plotlines,
    });
  },

  // Content Management
  loadChapter: async (
    projectId: string,
    chapterId: string
  ): Promise<string> => {
    return invoke<string>("load_chapter_content", { projectId, chapterId });
  },

  saveChapter: async (
    projectId: string,
    chapterId: string,
    content: string
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("save_chapter", {
      projectId,
      chapterId,
      content,
    });
  },

  deleteNode: async (
    projectId: string,
    id: string
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("delete_node", { projectId, id });
  },

  // Character Management
  saveCharacter: async (
    projectId: string,
    character: Character
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("save_character", { projectId, character });
  },

  deleteCharacter: async (
    projectId: string,
    characterId: string
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("delete_character", {
      projectId,
      characterId,
    });
  },

  createNode: async (
    projectId: string,
    parentId: string | undefined,
    name: string
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("create_node", {
      projectId,
      parentId,
      name,
    });
  },

  updateNodeMetadata: async (
    projectId: string,
    nodeId: string,
    update: Partial<Chapter>
  ): Promise<ProjectMetadata> => {
    return invoke<ProjectMetadata>("update_node_metadata", {
      projectId,
      nodeId,
      update,
    });
  },
};
