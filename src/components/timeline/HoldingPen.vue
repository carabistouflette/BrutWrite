<script setup lang="ts">
import type { Chapter } from '../../types';

defineProps<{
    scenes: Chapter[];
}>();

const emit = defineEmits<{
    (e: 'drag-start', sceneId: string): void;
}>();

function handleDragStart(event: DragEvent, scene: Chapter) {
    if (event.dataTransfer) {
        event.dataTransfer.setData('text/plain', scene.id);
        event.dataTransfer.effectAllowed = 'move';
    }
    emit('drag-start', scene.id);
}
</script>

<template>
    <aside class="holding-pen">
        <header class="holding-pen-header">
            <h3 class="holding-pen-title">
                <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10" />
                    <path d="M12 6v6l4 2" />
                </svg>
                Unplaced Scenes
            </h3>
            <span class="holding-pen-count">{{ scenes.length }}</span>
        </header>

        <div v-if="scenes.length === 0" class="holding-pen-empty">
            <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5V3m0 2h6m-6 0v2m6-2V3m0 2v2" />
            </svg>
            <p>All scenes have been placed on the timeline</p>
        </div>

        <ul v-else class="holding-pen-list">
            <li
                v-for="scene in scenes"
                :key="scene.id"
                class="holding-pen-item"
                draggable="true"
                @dragstart="handleDragStart($event, scene)"
            >
                <div class="scene-card">
                    <span class="scene-title">{{ scene.title }}</span>
                    <span class="scene-words">{{ scene.word_count }} words</span>
                </div>
                <div class="drag-hint">
                    <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
                        <circle cx="9" cy="6" r="1.5" />
                        <circle cx="15" cy="6" r="1.5" />
                        <circle cx="9" cy="12" r="1.5" />
                        <circle cx="15" cy="12" r="1.5" />
                        <circle cx="9" cy="18" r="1.5" />
                        <circle cx="15" cy="18" r="1.5" />
                    </svg>
                </div>
            </li>
        </ul>
    </aside>
</template>

<style scoped>
.holding-pen {
    width: 220px;
    min-width: 200px;
    max-width: 280px;
    background: var(--bg-secondary); /* Keep as is, it matches sidebar */
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    z-index: 20; /* Ensure it stays above canvas elements if they overlap slightly */
}

.holding-pen-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-tertiary);
}

.holding-pen-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
}

.icon {
    width: 16px;
    height: 16px;
    stroke: var(--color-primary);
}

.holding-pen-count {
    background: var(--color-primary);
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 12px;
}

.holding-pen-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    text-align: center;
    color: var(--text-tertiary);
}

.empty-icon {
    width: 48px;
    height: 48px;
    stroke: var(--text-tertiary);
    opacity: 0.5;
}

.holding-pen-empty p {
    margin: 0;
    font-size: 0.813rem;
    line-height: 1.4;
}

.holding-pen-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    margin: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px; /* Reduced gap */
}

.holding-pen-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px; /* Reduced padding */
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    cursor: grab;
    transition: all 0.15s ease;
}

.holding-pen-item:hover {
    border-color: var(--color-primary);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    transform: translateX(2px);
}

.holding-pen-item:active {
    cursor: grabbing;
    transform: scale(0.98);
}

.scene-card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
}

.scene-title {
    font-size: 0.813rem;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.scene-words {
    font-size: 0.688rem;
    color: var(--text-tertiary);
}

.drag-hint {
    color: var(--text-tertiary);
    opacity: 0.4;
    transition: opacity 0.15s ease;
}

.holding-pen-item:hover .drag-hint {
    opacity: 1;
    color: var(--color-primary);
}
</style>
