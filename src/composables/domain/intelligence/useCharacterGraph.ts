/**
 * Character Graph Composable
 *
 * Provides reactive state management for character interaction analysis,
 * including caching, loading states, and derived computed properties.
 */

import { ref, computed } from 'vue';
import { storeToRefs } from 'pinia';
import { intelligenceApi } from '../../../api/intelligence';
import type { CharacterGraphPayload, GraphNode, GraphAlert } from '../../../types/intelligence';
import { useProjectStore } from '../../../stores/project';
import { useSettingsStore } from '../../../stores/settings';

interface CharacterGraphState {
  payload: CharacterGraphPayload | null;
  isLoading: boolean;
  lastAnalyzedAt: string | null;
  error: string | null;
}

// Module-level state for singleton behavior
const state = ref<CharacterGraphState>({
  payload: null,
  isLoading: false,
  lastAnalyzedAt: null,
  error: null,
});

// Cooldown tracking
let lastAnalysisTime = 0;
const ANALYSIS_COOLDOWN_MS = 2000;

/**
 * Composable for character graph analysis.
 *
 * Provides:
 * - Reactive graph payload with caching
 * - Loading/error states
 * - Ghost (unmapped) character filtering
 * - Alert generation based on metrics
 * - Debounced analysis with cooldown
 */
export function useCharacterGraph() {
  const projectStore = useProjectStore();
  const settingsStore = useSettingsStore();
  const { settings } = storeToRefs(settingsStore);

  // --- Computed ---

  /**
   * Characters with 0 mentions (declared but never referenced).
   */
  const ghosts = computed<GraphNode[]>(() => {
    if (!state.value.payload) return [];
    return state.value.payload.nodes.filter((n) => !n.isMapped);
  });

  /**
   * Characters that are actively referenced in the manuscript.
   */
  const mappedNodes = computed<GraphNode[]>(() => {
    if (!state.value.payload) return [];
    return state.value.payload.nodes.filter((n) => n.isMapped);
  });

  /**
   * Generate alerts based on current graph metrics.
   */
  const alerts = computed<GraphAlert[]>(() => {
    if (!state.value.payload) return [];

    const { metrics, nodes } = state.value.payload;
    const result: GraphAlert[] = [];

    // SOLIPSISM: Over 50% isolation
    if (metrics.isolationRatio > 0.5) {
      result.push({
        code: 'SOLIPSISM',
        primaryText: 'Low Connectivity',
        tooltip: 'Over 50% of your cast has no meaningful interactions.',
      });
    }

    // SATELLITE: Multiple disconnected components
    if (metrics.connectedComponents > 1) {
      result.push({
        code: 'SATELLITE',
        primaryText: 'Isolated Subplots',
        tooltip: `There are ${metrics.connectedComponents} groups of characters that never interact with the main cast.`,
      });
    }

    // GHOST: Unmapped characters exist
    if (ghosts.value.length > 0) {
      result.push({
        code: 'GHOST',
        primaryText: 'Unmapped Characters',
        tooltip: `${ghosts.value.length} character(s) declared but never mentioned.`,
      });
    }

    // PROTAGONIST_ABSENT: Check if protagonist has low mentions
    const protagonists = nodes.filter((n) => {
      const char = projectStore.characterById(n.id);
      return char?.role === 'protagonist';
    });
    if (protagonists.length > 0 && protagonists.every((p) => p.mentionCount < 3)) {
      result.push({
        code: 'PROTAGONIST_ABSENT',
        primaryText: 'Protagonist Fading',
        tooltip: 'Your protagonist has very few mentions.',
      });
    }

    return result;
  });

  /**
   * Whether the cached data is stale (project updated since last analysis).
   */
  const isStale = computed<boolean>(() => {
    if (!state.value.lastAnalyzedAt) return true;
    return false;
  });

  // --- Actions ---

  /**
   * Analyze the character graph for the current project.
   *
   * Respects a 2-second cooldown to prevent spam.
   * Uses settings from the settings store.
   *
   * @param chapterIds - Optional array of chapter IDs to filter analysis
   */
  async function analyze(chapterIds?: string[]): Promise<void> {
    const projectId = projectStore.projectId;
    if (!projectId) {
      state.value.error = 'No project loaded';
      return;
    }

    // Enforce cooldown
    const now = Date.now();
    if (now - lastAnalysisTime < ANALYSIS_COOLDOWN_MS) {
      return;
    }

    state.value.isLoading = true;
    state.value.error = null;

    try {
      // Get settings and pass to API
      const { proximityWindow, pruneThreshold } = settings.value.intelligence;

      const payload = await intelligenceApi.getCharacterGraph(projectId, {
        proximityWindow,
        pruneThreshold,
        chapterIds,
      });
      state.value.payload = payload;
      state.value.lastAnalyzedAt = new Date().toISOString();
      lastAnalysisTime = now;
    } catch (err) {
      state.value.error = err instanceof Error ? err.message : String(err);
    } finally {
      state.value.isLoading = false;
    }
  }

  /**
   * Clear the cached graph data.
   */
  function clear(): void {
    state.value = {
      payload: null,
      isLoading: false,
      lastAnalyzedAt: null,
      error: null,
    };
    lastAnalysisTime = 0;
  }

  return {
    // State (readonly)
    payload: computed(() => state.value.payload),
    isLoading: computed(() => state.value.isLoading),
    lastAnalyzedAt: computed(() => state.value.lastAnalyzedAt),
    error: computed(() => state.value.error),

    // Derived
    ghosts,
    mappedNodes,
    alerts,
    isStale,

    // Actions
    analyze,
    clear,
  };
}
