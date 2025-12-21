export interface ProjectMetadata {
    id: string;
    title: string;
    author: string;
    created_at: string;
    updated_at: string;
    settings: ProjectSettings;
    manifest: Manifest;
    characters: Character[];
}

export interface ProjectSettings {
    daily_target: number;
    word_target: number;
}

export interface Manifest {
    chapters: Chapter[];
}

export interface Chapter {
    id: string;
    parent_id?: string;
    title: string;
    filename: string;
    word_count: number;
    order: number;
    /** ISO 8601 date/time for chronological placement */
    chronological_date?: string;
    /** Abstract timeframe (e.g., "Day 1", "Year 5") for fantasy/sci-fi */
    abstract_timeframe?: string;
    /** Estimated in-world duration (e.g., "2 hours", "3 days") */
    duration?: string;
    /** Plotline/subplot tag for swimlane grouping */
    plotline_tag?: string;
    /** Scene that must occur before this one (for causality checking) */
    depends_on?: string;
    /** POV character ID (for simultaneous-scene paradox detection) */
    pov_character_id?: string;
}

export interface Character {
    id: string;
    name: string;
    role: CharacterRole;
    archetype?: string;
    description: string;
    engine?: CharacterEngine;
    physical_features?: string;
    traits: string[];
    arc: string;
    notes: string;
}

export interface CharacterEngine {
    desire: string;
    fear: string;
    wound: string;
    secret: string;
}

export enum CharacterRole {
    Protagonist = "protagonist",
    Antagonist = "antagonist",
    Secondary = "secondary",
    Extra = "extra",
}

// Frontend specific for recursively rendering the tree
export interface FileNode {
    id: string;
    name: string;
    children?: FileNode[];
    // Link to backend data
    filename?: string;
    word_count?: number;
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

// Timeline Visualization Types

export interface Plotline {
    id: string;
    name: string;
    color: string;
}

export interface TimelineScene {
    chapter: Chapter;
    x: number;           // Computed pixel position
    width: number;       // Based on duration
    plotlineLaneY: number;
}

export type ParadoxType = 'simultaneous_presence' | 'causality_violation' | 'orphan_gap';

export interface ParadoxWarning {
    type: ParadoxType;
    sceneIds: string[];
    message: string;
}
