import { ref } from 'vue';
import type { FileNode } from '../types';

// Mock initial data
const INITIAL_DATA: FileNode[] = [
    {
        id: '1',
        name: 'Chapter 1: The Beginning',
        children: [
            { id: '1-1', name: 'Section 1.1: Intro' },
            { id: '1-2', name: 'Section 1.2: The Incident' }
        ]
    },
    {
        id: '2',
        name: 'Chapter 2: The Middle',
        children: []
    }
];

// Singleton state (simple store pattern)
const projectData = ref<FileNode[]>(INITIAL_DATA);
const activeId = ref<string | undefined>(undefined);

export function useProjectData() {

    // Helper to find a node by ID and its parent list
    const findNodeAndParent = (list: FileNode[], id: string): { node: FileNode | null, parentList: FileNode[] | null, index: number } => {
        const index = list.findIndex(item => item.id === id);
        if (index !== -1) {
            return { node: list[index], parentList: list, index };
        }
        for (const item of list) {
            if (item.children) {
                const result = findNodeAndParent(item.children, id);
                if (result.node) return result;
            }
        }
        return { node: null, parentList: null, index: -1 };
    };

    const selectNode = (id: string) => {
        activeId.value = id;
    };

    const addChapter = () => {
        const newId = `chapter-${Date.now()}`;
        projectData.value.push({
            id: newId,
            name: 'New Chapter',
            children: []
        });
        activeId.value = newId;
        return newId;
    };

    const addSection = (parentId: string) => {
        const { node } = findNodeAndParent(projectData.value, parentId);
        if (node) {
            if (!node.children) node.children = [];
            const newId = `${parentId}-${Date.now()}`;
            node.children.push({
                id: newId,
                name: 'New Section',
                children: []
            });
            // Optionally select the new section?
            // activeId.value = newId; 
        }
    };

    const deleteNode = (id: string) => {
        const { parentList, index } = findNodeAndParent(projectData.value, id);
        if (parentList && index !== -1) {
            parentList.splice(index, 1);
        }

        if (activeId.value === id) {
            activeId.value = undefined;
        }
    };

    const renameNode = (id: string, newName: string) => {
        const { node } = findNodeAndParent(projectData.value, id);
        if (node) {
            node.name = newName;
        }
    };

    return {
        projectData,
        activeId,
        selectNode,
        addChapter,
        addSection,
        deleteNode,
        renameNode
    };
}
