import { ref, computed } from 'vue';
import { useSettings } from './useSettings';
import { useProjectData } from './useProjectData';
import {
    calculateStreak,
    calculateAverage,
    getBestDay,
    getTodayStats
} from '../utils/stats';
import type { DailyStats } from '../utils/stats';

const STORAGE_KEY = 'brutwrite_gamification';

interface GamificationState {
    projectTarget: number;
    history: DailyStats[];
    lastSessionDate: string;
}

const DEFAULT_STATE: GamificationState = {
    projectTarget: 50000, // Deprecated in local state, kept for migration
    history: [],
    lastSessionDate: new Date().toISOString().split('T')[0]
};

// Singleton state
const state = ref<GamificationState>({ ...DEFAULT_STATE });
const sessionWords = ref(0); // Volatile session counter

export function useGamification() {
    const { settings } = useSettings();
    const { settings: projectSettings, updateSettings, totalWords } = useProjectData();

    // Load from local storage
    const loadState = () => {
        const stored = localStorage.getItem(STORAGE_KEY);
        if (stored) {
            try {
                const parsed = JSON.parse(stored);
                // migration for old state removing dailyGoal if exists in stored
                const { dailyGoal, ...rest } = parsed;
                state.value = { ...DEFAULT_STATE, ...rest };
            } catch (e) {
                console.error('Failed to parse gamification data', e);
            }
        }
        checkNewDay();
    };

    let saveTimeout: ReturnType<typeof setTimeout>;
    const saveStateDebounced = () => {
        clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(state.value));
        }, 2000); // 2 second debounce
    };

    // Check if it's a new day to reset daily counters if needed (though we track history)
    const checkNewDay = () => {
        const today = new Date().toISOString().split('T')[0];
        if (state.value.lastSessionDate !== today) {
            // If we missed days, we might want to fill gaps with 0 or just ignore
            state.value.lastSessionDate = today;
        }
    };

    // Add words (delta)
    const addWords = (delta: number) => {
        // 1. Update session
        sessionWords.value += delta;

        // 2. Update today's history
        const today = new Date().toISOString().split('T')[0];
        const historyIndex = state.value.history.findIndex(h => h.date === today);

        if (historyIndex >= 0) {
            state.value.history[historyIndex].wordCount += delta;
            if (state.value.history[historyIndex].wordCount < 0) {
                state.value.history[historyIndex].wordCount = 0;
            }
        } else {
            state.value.history.push({ date: today, wordCount: Math.max(0, delta) });
        }

        // Note: Total project words are now calculated via useProjectData from recursive nodes
        saveStateDebounced();
    };

    const setProjectTarget = async (target: number) => {
        // Update via project settings
        if (projectSettings.value) {
            await updateSettings({
                ...projectSettings.value,
                word_target: target
            });
        }
        // Fallback for local state (UI compatibility)
        state.value.projectTarget = target;
        saveStateDebounced();
    };

    // Computed
    const todayStats = computed(() => getTodayStats(state.value.history));

    const progressDaily = computed(() => {
        const goal = settings.value.general.dailyGoal || 500;
        return Math.min(100, Math.max(0, (todayStats.value.wordCount / goal) * 100));
    });

    const progressProject = computed(() => {
        // Use project setting if available, else fallback to local state
        const target = projectSettings.value?.word_target || state.value.projectTarget || 50000;
        const current = totalWords.value || 0;
        return Math.min(100, Math.max(0, (current / target) * 100));
    });

    const streak = computed(() => calculateStreak(state.value.history));

    const averageDaily = computed(() => calculateAverage(state.value.history));

    const bestDay = computed(() => getBestDay(state.value.history));

    // Init
    loadState();

    return {
        dailyGoal: computed(() => settings.value.general.dailyGoal),
        projectTarget: computed(() => projectSettings.value?.word_target || state.value.projectTarget),
        totalProjectWords: computed(() => totalWords.value || 0),
        todayWords: computed(() => todayStats.value.wordCount),
        history: computed(() => state.value.history),
        progressDaily,
        progressProject,
        streak,
        averageDaily,
        bestDay,
        addWords,
        setProjectTarget
    };
}
