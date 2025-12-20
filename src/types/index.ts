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
    title: string;
    filename: string;
    order: number;
    // Helper for frontend tree stricture (optional in Rust, but needed for recursive trees if we support them)
    // For now, flat chapters list in Rust, but frontend supports tree.
    // We will need to decide if Rust supports recursive structure.
    // The current Rust model is flat `chapters: Vec<Chapter>`.
    // Frontend `FileNode` has `children`.
    // Let's stick to the Rust model for persistence: Flat list.
    // We might need a `parent_id` in Rust if we want hierarchy.
    // For now, let's keep it simple: separate frontend "FileNode" from backend "Chapter" 
    // OR update Rust to support tree.
    // Given user desire for "FileTree", hierarchy is implied.
    // I will add `parent_id` to Chapter in types for future-proofing, 
    // but for now use FileNode for UI and map to Manifest for storage.
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
