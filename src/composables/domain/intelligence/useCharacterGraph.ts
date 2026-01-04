import { defineStore, storeToRefs } from 'pinia';
import { ref, computed } from 'vue';
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

// Internal store definition
export const useCharacterGraph = defineStore('character-graph', () => {
  const state = ref<CharacterGraphState>({
    payload: null,
    isLoading: false,
    lastAnalyzedAt: null,
    error: null,
  });

  const lastAnalysisTime = ref(0);

  const projectStore = useProjectStore();
  const settingsStore = useSettingsStore();
  const { settings } = storeToRefs(settingsStore);

  // --- Computed ---

  const ghosts = computed<GraphNode[]>(() => {
    if (!state.value.payload) return [];
    return state.value.payload.nodes.filter((n) => !n.isMapped);
  });

  const mappedNodes = computed<GraphNode[]>(() => {
    if (!state.value.payload) return [];
    return state.value.payload.nodes.filter((n) => n.isMapped);
  });

  const alerts = computed<GraphAlert[]>(() => {
    if (!state.value.payload) return [];

    const { metrics, nodes } = state.value.payload;
    const result: GraphAlert[] = [];

    if (metrics.isolationRatio > 0.5) {
      result.push({
        code: 'SOLIPSISM',
        primaryText: 'Low Connectivity',
        tooltip: 'Over 50% of your cast has no meaningful interactions.',
      });
    }

    if (metrics.connectedComponents > 1) {
      result.push({
        code: 'SATELLITE',
        primaryText: 'Isolated Subplots',
        tooltip: `There are ${metrics.connectedComponents} groups of characters that never interact with the main cast.`,
      });
    }

    if (ghosts.value.length > 0) {
      result.push({
        code: 'GHOST',
        primaryText: 'Unmapped Characters',
        tooltip: `${ghosts.value.length} character(s) declared but never mentioned.`,
      });
    }

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

  const isStale = computed<boolean>(() => {
    if (!state.value.lastAnalyzedAt) return true;
    return false;
  });

  // --- Actions ---

  async function analyze(chapterIds?: string[]): Promise<void> {
    const projectId = projectStore.projectId;
    if (!projectId) {
      state.value.error = 'No project loaded';
      return;
    }

    // Rate limiting: prevent spamming
    const cooldown = 500;
    const now = Date.now();
    if (now - lastAnalysisTime.value < cooldown) {
      return;
    }

    state.value.isLoading = true;
    state.value.error = null;

    try {
      const { proximityWindow, pruneThreshold } = settings.value.intelligence;

      const payload = await intelligenceApi.getCharacterGraph(projectId, {
        proximityWindow,
        pruneThreshold,
        chapterIds,
      });
      state.value.payload = payload;
      state.value.lastAnalyzedAt = new Date().toISOString();
      lastAnalysisTime.value = now;
    } catch (err) {
      state.value.error = err instanceof Error ? err.message : String(err);
    } finally {
      state.value.isLoading = false;
    }
  }

  function clear(): void {
    state.value = {
      payload: null,
      isLoading: false,
      lastAnalyzedAt: null,
      error: null,
    };
    lastAnalysisTime.value = 0;
  }

  const payload = computed(() => state.value.payload);
  const isLoading = computed(() => state.value.isLoading);
  const lastAnalyzedAt = computed(() => state.value.lastAnalyzedAt);
  const error = computed(() => state.value.error);

  return {
    state,
    payload,
    isLoading,
    lastAnalyzedAt,
    error,
    ghosts,
    mappedNodes,
    alerts,
    isStale,
    analyze,
    clear,
  };
});
