<script setup lang="ts">
import { toRef } from 'vue';
import { useNarrativeConnectors } from '../../composables/useNarrativeConnectors';
import { useTimeline } from '../../composables/useTimeline';

const props = defineProps<{
    container: HTMLElement | null;
    isMounted: boolean;
}>();

const { narrativeConnectors } = useTimeline();

const { 
    connectorPaths, 
    showNarrativeConnectors, 
    toggleConnectors,
    updateConnectorPositions
} = useNarrativeConnectors(toRef(props, 'container'), narrativeConnectors, toRef(props, 'isMounted'));

defineExpose({
    showNarrativeConnectors,
    toggleConnectors,
    updateConnectorPositions
});
</script>

<template>
    <svg
        v-if="showNarrativeConnectors"
        class="narrative-overlay"
        xmlns="http://www.w3.org/2000/svg"
    >
        <defs>
            <marker id="arrow" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto">
                <path d="M0,0 L0,6 L6,3 z" fill="var(--color-primary)" />
            </marker>
            <marker id="arrow-flashback" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto">
                <path d="M0,0 L0,6 L6,3 z" fill="var(--color-warning)" />
            </marker>
        </defs>
        <path
            v-for="(path, idx) in connectorPaths"
            :key="idx"
            :d="path.d"
            :stroke="path.color"
            stroke-width="2"
            fill="none"
            :marker-end="path.isFlashback ? 'url(#arrow-flashback)' : 'url(#arrow)'"
            :class="{ 'flashback-connector': path.isFlashback }"
        />
    </svg>
</template>

<style scoped>
.narrative-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 10;
}

.flashback-connector {
    stroke-dasharray: 8 4;
    animation: flashback-pulse 2s ease-in-out infinite;
}

@keyframes flashback-pulse {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
}
</style>
