<script setup lang="ts">
defineProps<{
    showConnectors: boolean;
}>();

const emit = defineEmits<{
    (e: 'toggle-connectors'): void;
    (e: 'fit'): void;
    (e: 'zoom-in'): void;
    (e: 'zoom-out'): void;
    (e: 'apply-chronological'): void;
    (e: 'export', format: 'png' | 'pdf'): void;
    (e: 'open-calendar'): void;
}>();
</script>

<template>
    <div class="timeline-controls">
        <div class="controls-group">
            <span class="group-label">View</span>

            <button
                class="control-btn"
                title="Calendar Settings"
                @click="emit('open-calendar')"
            >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
                    <line x1="16" y1="2" x2="16" y2="6" />
                    <line x1="8" y1="2" x2="8" y2="6" />
                    <line x1="3" y1="10" x2="21" y2="10" />
                </svg>
                <span class="btn-label">Calendar</span>
            </button>

            <button
                class="control-btn"
                :class="{ active: showConnectors }"
                title="Toggle narrative connectors"
                @click="emit('toggle-connectors')"
            >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6" />
                    <polyline points="15 3 21 3 21 9" />
                    <line x1="10" y1="14" x2="21" y2="3" />
                </svg>
                <span class="btn-label">Connectors</span>
            </button>

            <button class="control-btn" title="Fit to view" @click="emit('fit')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M8 3H5a2 2 0 00-2 2v3m18 0V5a2 2 0 00-2-2h-3m0 18h3a2 2 0 002-2v-3M3 16v3a2 2 0 002 2h3" />
                </svg>
            </button>

            <div class="zoom-controls">
                <button class="control-btn" title="Zoom out" @click="emit('zoom-out')">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="8" />
                        <line x1="21" y1="21" x2="16.65" y2="16.65" />
                        <line x1="8" y1="11" x2="14" y2="11" />
                    </svg>
                </button>
                <button class="control-btn" title="Zoom in" @click="emit('zoom-in')">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="8" />
                        <line x1="21" y1="21" x2="16.65" y2="16.65" />
                        <line x1="11" y1="8" x2="11" y2="14" />
                        <line x1="8" y1="11" x2="14" y2="11" />
                    </svg>
                </button>
            </div>
        </div>

        <div class="controls-group">
            <span class="group-label">Actions</span>

            <button
                class="control-btn action-btn"
                title="Reorder manuscript to chronological order"
                @click="emit('apply-chronological')"
            >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="17" y1="10" x2="3" y2="10" />
                    <line x1="21" y1="6" x2="3" y2="6" />
                    <line x1="21" y1="14" x2="3" y2="14" />
                    <line x1="17" y1="18" x2="3" y2="18" />
                </svg>
                <span class="btn-label">Apply Order</span>
            </button>
        </div>

        <div class="controls-group">
            <span class="group-label">Export</span>

            <button class="control-btn" title="Export as PNG" @click="emit('export', 'png')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                    <circle cx="8.5" cy="8.5" r="1.5" />
                    <polyline points="21 15 16 10 5 21" />
                </svg>
                <span class="btn-label">PNG</span>
            </button>

            <button class="control-btn" title="Export as PDF" @click="emit('export', 'pdf')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
                    <polyline points="14 2 14 8 20 8" />
                    <line x1="16" y1="13" x2="8" y2="13" />
                    <line x1="16" y1="17" x2="8" y2="17" />
                </svg>
                <span class="btn-label">PDF</span>
            </button>
        </div>
    </div>
</template>

<style scoped>
.timeline-controls {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 10px 16px;
    background: #000000; /* Pitch black */
    border-bottom: 1px solid #333;
}

.controls-group {
    display: flex;
    align-items: center;
    gap: 6px;
}

.group-label {
    font-size: 0.688rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #666;
    margin-right: 6px;
}

.control-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-secondary);
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
}

.control-btn:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
    background: rgba(59, 130, 246, 0.05);
}

.control-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
}

.control-btn svg {
    width: 16px;
    height: 16px;
}

.btn-label {
    display: none;
}

@media (min-width: 900px) {
    .btn-label {
        display: inline;
    }
}

.zoom-controls {
    display: flex;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    overflow: hidden;
}

.zoom-controls .control-btn {
    border: none;
    border-radius: 0;
    padding: 6px 8px;
}

.zoom-controls .control-btn:first-child {
    border-right: 1px solid var(--border-color);
}

.action-btn {
    background: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
    color: var(--color-primary);
}

.action-btn:hover {
    background: var(--color-primary);
    color: white;
}
</style>
