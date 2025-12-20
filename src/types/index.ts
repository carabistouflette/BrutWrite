export interface ProjectMetadata {
    id: string;
    title: string;
    author: string;
    created_at: string;
    updated_at: string;
    settings: ProjectSettings;
    manifest: Manifest;
}

export interface ProjectSettings {
    daily_target: number;
    theme: string;
}

export interface Manifest {
    chapters: Chapter[];
}

export interface Chapter {
    id: string;
    parent_id?: string;
    title: string;
    filename: string;
    order: number;
}

// Frontend specific for recursively rendering the tree
export interface FileNode {
    id: string;
    name: string;
    children?: FileNode[];
    // Link to backend data
    filename?: string;
}

export interface DailyStats {
    date: string;
    wordCount: number;
}

export interface GamificationState {
    dailyGoal: number;
    projectTarget: number;
    history: DailyStats[];
    totalProjectWords: number;
    lastSessionDate: string;
}

export interface ContextMenuPosition {
    x: number;
    y: number;
}
