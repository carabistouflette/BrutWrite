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
    totalProjectWords: number; // Snapshot of total words
    lastSessionDate: string;
}

const DEFAULT_STATE: GamificationState = {
    projectTarget: 50000, // Deprecated in local state, kept for migration
    history: [],
    totalProjectWords: 0,
    lastSessionDate: new Date().toISOString().split('T')[0]
};

// Singleton state
const state = ref<GamificationState>({ ...DEFAULT_STATE });
const sessionWords = ref(0); // Volatile session counter

export function useGamification() {
    const { settings } = useSettings();
    const { settings: projectSettings, updateSettings } = useProjectData();

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

    const saveState = () => {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(state.value));
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
            // Prevent negative history for a day (unless we want to allow it?)
            if (state.value.history[historyIndex].wordCount < 0) {
                state.value.history[historyIndex].wordCount = 0;
            }
        } else {
            state.value.history.push({ date: today, wordCount: Math.max(0, delta) });
        }

        // 3. Update total project words (Global counter)
        state.value.totalProjectWords += delta;
        if (state.value.totalProjectWords < 0) state.value.totalProjectWords = 0;

        saveState();
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
        saveState();
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
        return Math.min(100, Math.max(0, (state.value.totalProjectWords / target) * 100));
    });

    const streak = computed(() => calculateStreak(state.value.history));

    const averageDaily = computed(() => calculateAverage(state.value.history));

    const bestDay = computed(() => getBestDay(state.value.history));

    // Init
    loadState();

    return {
        dailyGoal: computed(() => settings.value.general.dailyGoal),
        projectTarget: computed(() => projectSettings.value?.word_target || state.value.projectTarget),
        totalProjectWords: computed(() => state.value.totalProjectWords),
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
