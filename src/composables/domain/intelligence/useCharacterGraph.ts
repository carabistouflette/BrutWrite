import { defineStore, storeToRefs } from 'pinia';
import { ref, computed } from 'vue';
import { intelligenceApi } from '../../../api/intelligence';
import type { CharacterGraphPayload, GraphNode, GraphAlert } from '../../../types/intelligence';
import { useProjectStore } from '../../../stores/project';
import { useSettingsStore } from '../../../stores/settings';
import { getGraphAlerts } from '../../../utils/intelligence/graphAnalysis';

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

    // Filter protagonists using the store data
    const protagonists = nodes.filter((n) => {
      const char = projectStore.characterById(n.id);
      return char?.role === 'protagonist';
    });

    return getGraphAlerts(metrics, ghosts.value, protagonists);
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
