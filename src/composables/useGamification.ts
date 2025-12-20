
import { ref, computed } from 'vue';
import { useSettings } from './useSettings';

const STORAGE_KEY = 'brutwrite_gamification';

interface DailyStats {
    date: string;
    wordCount: number;
}

interface GamificationState {
    projectTarget: number;
    history: DailyStats[];
    totalProjectWords: number; // Snapshot of total words
    lastSessionDate: string;
}

const DEFAULT_STATE: GamificationState = {
    projectTarget: 50000,
    history: [],
    totalProjectWords: 0,
    lastSessionDate: new Date().toISOString().split('T')[0]
};

// Singleton state
const state = ref<GamificationState>({ ...DEFAULT_STATE });
const sessionWords = ref(0); // Volatile session counter

export function useGamification() {
    const { settings } = useSettings();

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

    const setProjectTarget = (target: number) => {
        state.value.projectTarget = target;
        saveState();
    };

    // Computed
    const todayStats = computed(() => {
        const today = new Date().toISOString().split('T')[0];
        return state.value.history.find(h => h.date === today) || { date: today, wordCount: 0 };
    });

    const progressDaily = computed(() => {
        const goal = settings.value.general.dailyGoal || 500;
        return Math.min(100, Math.max(0, (todayStats.value.wordCount / goal) * 100));
    });

    const progressProject = computed(() => {
        return Math.min(100, Math.max(0, (state.value.totalProjectWords / state.value.projectTarget) * 100));
    });

    // Streak calculation
    const streak = computed(() => {
        // Current streak: consecutive days with > 0 words ending at today (or yesterday if today is 0)
        let currentStreak = 0;
        const dayMap = new Map(state.value.history.map(h => [h.date, h.wordCount]));
        const today = new Date().toISOString().split('T')[0];

        let d = new Date();
        const todayCount = dayMap.get(today) || 0;
        if (todayCount === 0) {
            d.setDate(d.getDate() - 1);
        }

        while (true) {
            const dateStr = d.toISOString().split('T')[0];
            const count = dayMap.get(dateStr) || 0;

            if (count > 0) {
                currentStreak++;
                d.setDate(d.getDate() - 1);
            } else {
                break;
            }
        }
        return currentStreak;
    });

    const averageDaily = computed(() => {
        if (state.value.history.length === 0) return 0;
        const total = state.value.history.reduce((acc, curr) => acc + curr.wordCount, 0);
        return Math.round(total / state.value.history.length);
    });

    const bestDay = computed(() => {
        if (state.value.history.length === 0) return 0;
        return Math.max(...state.value.history.map(h => h.wordCount));
    });

    // Init
    loadState();

    return {
        dailyGoal: computed(() => settings.value.general.dailyGoal),
        projectTarget: computed(() => state.value.projectTarget),
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
