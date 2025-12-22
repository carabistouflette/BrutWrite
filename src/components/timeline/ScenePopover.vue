<script setup lang="ts">
import { computed } from 'vue';
import { useTimeline } from '../../composables/useTimeline';

const props = defineProps<{
    sceneId: string;
    x: number;
    y: number;
}>();

const { allChapters, paradoxWarnings } = useTimeline();

const scene = computed(() => allChapters.value.find(c => c.id === props.sceneId));

const previousScene = computed(() => {
    if (!scene.value) return null;
    const idx = allChapters.value.findIndex(c => c.id === props.sceneId);
    return idx > 0 ? allChapters.value[idx - 1] : null;
});

const timeSincePrevious = computed(() => {
    if (!scene.value?.chronological_date || !previousScene.value?.chronological_date) {
        return null;
    }
    const current = new Date(scene.value.chronological_date).getTime();
    const prev = new Date(previousScene.value.chronological_date).getTime();
    const diffMs = current - prev;

    if (diffMs < 0) return 'Before previous scene';

    const hours = Math.floor(diffMs / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (days > 365) return `${Math.floor(days / 365)} year(s) later`;
    if (days > 30) return `${Math.floor(days / 30)} month(s) later`;
    if (days > 0) return `${days} day(s) later`;
    if (hours > 0) return `${hours} hour(s) later`;
    return 'Same time';
});

const warnings = computed(() =>
    paradoxWarnings.value.filter(w => w.sceneIds.includes(props.sceneId))
);

const popoverStyle = computed(() => ({
    left: `${props.x + 12}px`,
    top: `${props.y + 12}px`,
}));
import './ScenePopover.css';
</script>

<template>
    <div v-if="scene" class="scene-popover" :style="popoverStyle">
        <div class="popover-header">
            <h4 class="scene-title">{{ scene.title }}</h4>
            <span v-if="scene.plotline_tag" class="plotline-badge">
                {{ scene.plotline_tag }}
            </span>
        </div>

        <div class="popover-body">
            <div class="stat-row">
                <span class="stat-label">Word Count</span>
                <span class="stat-value">{{ scene.word_count?.toLocaleString() || '0' }}</span>
            </div>

            <div v-if="scene.pov_character_id" class="stat-row">
                <span class="stat-label">POV</span>
                <span class="stat-value">{{ scene.pov_character_id }}</span>
            </div>

            <div v-if="scene.duration" class="stat-row">
                <span class="stat-label">Duration</span>
                <span class="stat-value">{{ scene.duration }}</span>
            </div>

            <div v-if="timeSincePrevious" class="stat-row">
                <span class="stat-label">From Previous</span>
                <span class="stat-value time-delta">{{ timeSincePrevious }}</span>
            </div>

            <div v-if="scene.chronological_date" class="stat-row">
                <span class="stat-label">Date</span>
                <span class="stat-value date-display">
                    {{ new Date(scene.chronological_date).toLocaleDateString() }}
                </span>
            </div>

            <div v-if="scene.abstract_timeframe && !scene.chronological_date" class="stat-row">
                <span class="stat-label">Timeframe</span>
                <span class="stat-value">{{ scene.abstract_timeframe }}</span>
            </div>
        </div>

        <div v-if="warnings.length > 0" class="popover-warnings">
            <div v-for="warning in warnings" :key="warning.message" class="warning-item">
                <svg class="warning-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
                    <line x1="12" y1="9" x2="12" y2="13" />
                    <line x1="12" y1="17" x2="12.01" y2="17" />
                </svg>
                <span>{{ warning.message }}</span>
            </div>
        </div>
    </div>
</template>
<style scoped>
/* Styles moved to ScenePopover.css */
</style>
