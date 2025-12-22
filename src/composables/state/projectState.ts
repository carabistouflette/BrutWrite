import { ref, shallowRef } from 'vue';
import type { FileNode, ProjectSettings } from '../../types';

// Singleton state
export const projectData = ref<FileNode[]>([]);
export const activeId = ref<string | undefined>(undefined);
export const projectId = ref<string | undefined>(undefined); // Store active project UUID
export const projectSettings = ref<ProjectSettings | null>(null);
export const projectPlotlines = ref<any[]>([]);

// Optimized lookups
export const nodeMap = shallowRef(new Map<string, FileNode>());
export const flatNodes = shallowRef<FileNode[]>([]);

// Shared debouncers and pending updates (Singleton)
export const pendingMetadataUpdates = new Map<string, any>();
