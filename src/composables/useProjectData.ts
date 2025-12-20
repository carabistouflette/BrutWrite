import { ref } from 'vue';
import { projectApi } from '../api/project';
import type { FileNode, Manifest, Chapter } from '../types';

// Singleton state
const projectData = ref<FileNode[]>([]);
const activeId = ref<string | undefined>(undefined);
const projectId = ref<string | undefined>(undefined); // Store active project UUID

export function useProjectData() {

    // --- Backend Sync Helpers ---

    const syncManifest = async () => {
        if (!projectId.value) return;

        const chapters: Chapter[] = [];
        let order = 0;

        const traverse = (nodes: FileNode[], parentId?: string) => {
            for (const node of nodes) {
                chapters.push({
                    id: node.id,
                    parent_id: parentId,
                    title: node.name,
                    filename: node.filename || `${node.id}.md`,
                    order: order++
                });
                if (node.children) traverse(node.children, node.id);
            }
        };

        traverse(projectData.value);
        const manifest: Manifest = { chapters };

        try {
            await projectApi.updateManifest(projectId.value, manifest);
            console.debug('Manifest synced');
        } catch (e) {
            console.error('Failed to sync manifest:', e);
        }
    };

    // --- Actions ---

    const loadProject = async (path: string) => {
        try {
            const metadata = await projectApi.load(path);
            projectId.value = metadata.id;

            // Map Manifest -> FileNode[] with hierarchy reconstruction
            const chapters = [...metadata.manifest.chapters].sort((a, b) => a.order - b.order);
            const nodeMap = new Map<string, FileNode>();
            const rootNodes: FileNode[] = [];

            // First pass: create all nodes
            for (const c of chapters) {
                const node: FileNode = {
                    id: c.id,
                    name: c.title,
                    filename: c.filename,
                    children: []
                };
                nodeMap.set(c.id, node);
            }

            // Second pass: link parents/children
            for (const c of chapters) {
                const node = nodeMap.get(c.id)!;
                if (c.parent_id && nodeMap.has(c.parent_id)) {
                    nodeMap.get(c.parent_id)!.children?.push(node);
                } else {
                    rootNodes.push(node);
                }
            }

            projectData.value = rootNodes;

            if (projectData.value.length > 0) {
                activeId.value = projectData.value[0].id;
            }
        } catch (e) {
            console.error('Failed to load project:', e);
        }
    };

    const createProject = async (path: string, name: string, author: string) => {
        try {
            const metadata = await projectApi.create(path, name, author);
            projectId.value = metadata.id;
            projectData.value = [];

            await addChapter();
        } catch (e) {
            console.error('Failed to create project:', e);
        }
    }

    const selectNode = (id: string) => {
        activeId.value = id;
    };

    const addChapter = async () => {
        const newId = `chapter-${Date.now()}`;
        const newNode: FileNode = {
            id: newId,
            name: 'New Chapter',
            filename: `${newId}.md`,
            children: []
        };

        projectData.value.push(newNode);
        activeId.value = newId;

        await syncManifest();
        return newId;
    };

    const addSection = async (parentId: string) => {
        const findAndAdd = (nodes: FileNode[]): boolean => {
            const index = nodes.findIndex(n => n.id === parentId);
            if (index !== -1) {
                if (!nodes[index].children) nodes[index].children = [];
                const newId = `sec-${Date.now()}`;
                nodes[index].children!.push({
                    id: newId,
                    name: 'New Section',
                    filename: `${newId}.md`,
                    children: []
                });
                return true;
            }
            for (const node of nodes) {
                if (node.children && findAndAdd(node.children)) return true;
            }
            return false;
        }

        if (findAndAdd(projectData.value)) {
            await syncManifest();
        }
    };

    const deleteNode = async (id: string) => {
        const deleteFromList = (nodes: FileNode[]): boolean => {
            const index = nodes.findIndex(n => n.id === id);
            if (index !== -1) {
                nodes.splice(index, 1);
                return true;
            }
            for (const node of nodes) {
                if (node.children && deleteFromList(node.children)) return true;
            }
            return false;
        };

        if (deleteFromList(projectData.value)) {
            if (activeId.value === id) activeId.value = undefined;
            await syncManifest();
        }
    };

    const renameNode = async (id: string, newName: string) => {
        const findAndRename = (nodes: FileNode[]) => {
            const node = nodes.find(n => n.id === id);
            if (node) {
                node.name = newName;
                return true;
            }
            for (const n of nodes) {
                if (n.children && findAndRename(n.children)) return true;
            }
            return false;
        };

        if (findAndRename(projectData.value)) {
            await syncManifest();
        }
    };

    return {
        projectData,
        activeId,
        projectId,
        loadProject,
        createProject,
        selectNode,
        addChapter,
        addSection,
        deleteNode,
        renameNode
    };
}
