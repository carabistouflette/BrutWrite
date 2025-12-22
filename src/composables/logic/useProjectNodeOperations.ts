import type { FileNode } from '../../types';
import { projectApi } from '../../api/project';
import { useAppStatus } from '../useAppStatus';
import { reconstructHierarchy } from '../../utils/tree';
import { projectData, projectId, activeId, pendingMetadataUpdates } from '../state/projectState';
import { useProjectSync } from './useProjectSync';

export function useProjectNodeOperations() {
    const { notifyError } = useAppStatus();
    const { syncNodeMetadataDebounced, syncManifestDebounced } = useProjectSync();

    const addChapter = async () => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.createNode(projectId.value, undefined, 'New Chapter');
            projectData.value = reconstructHierarchy(metadata.manifest.chapters);
            
             const roots = projectData.value;
             if (roots.length > 0) {
                 const newChapter = roots[roots.length - 1];
                 activeId.value = newChapter.id;
                 return newChapter.id;
             }
        } catch (e) {
            notifyError('Failed to create chapter', e);
        }
    };

    const addSection = async (parentId: string) => {
        if (!projectId.value) return;
        try {
            const metadata = await projectApi.createNode(projectId.value, parentId, 'New Section');
            projectData.value = reconstructHierarchy(metadata.manifest.chapters);
        } catch (e) {
            notifyError('Failed to create section', e);
        }
    };

    const deleteNode = async (id: string, nodeMap: Map<string, FileNode>) => {
        if (!projectId.value) return;

        const node = nodeMap.get(id);
        if (!node) return;

        const collectFilenames = (n: FileNode, acc: string[]) => {
            if (n.filename) acc.push(n.filename);
            if (n.children) {
                n.children.forEach(child => collectFilenames(child, acc));
            }
        };

        const filesToDelete: string[] = [];
        collectFilenames(node, filesToDelete);

        try {
            const metadata = await projectApi.deleteNode(projectId.value, id, filesToDelete);

            // Atomic sync from backend
            projectData.value = reconstructHierarchy(metadata.manifest.chapters);
            if (activeId.value === id) activeId.value = undefined;

        } catch (e) {
            notifyError(`Failed to delete node ${id}`, e);
        }
    };

    const renameNode = async (id: string, newName: string, nodeMap: Map<string, FileNode>) => {
        const node = nodeMap.get(id);
        if (node && node.name !== newName) {
            node.name = newName;
            syncNodeMetadataDebounced(id, { title: newName });
        }
    };

    const updateStructure = async (newStructure: FileNode[]) => {
        projectData.value = newStructure;
        syncManifestDebounced();
    };

    const updateNodeStats = (id: string, wordCount: number, nodeMap: Map<string, FileNode>) => {
        const node = nodeMap.get(id);
        if (node && node.word_count !== wordCount) {
            node.word_count = wordCount;
        }
    };

    const updateNodeTemporal = async (id: string, updates: Partial<FileNode>, nodeMap: Map<string, FileNode>) => {
        const node = nodeMap.get(id);
        if (node) {
            // Only allow temporal updates here
            const allowed = ['chronological_date', 'abstract_timeframe', 'duration', 'plotline_tag', 'depends_on', 'pov_character_id'];
            let changed = false;
            const updateForBackend: any = {};
            
            allowed.forEach(key => {
                if (key in updates && (node as any)[key] !== (updates as any)[key]) {
                    (node as any)[key] = (updates as any)[key];
                    (updateForBackend as any)[key] = (updates as any)[key];
                    changed = true;
                }
            });

            if (changed) {
                syncNodeMetadataDebounced(id, updateForBackend);
            }
        }
    };

    return {
        addChapter,
        addSection,
        deleteNode,
        renameNode,
        updateStructure,
        updateNodeStats,
        updateNodeTemporal
    };
}
