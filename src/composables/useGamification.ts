
import { ref, computed } from 'vue';

const STORAGE_KEY = 'brutwrite_gamification';

interface DailyStats {
    date: string;
    wordCount: number;
}

interface GamificationState {
    dailyGoal: number;
    projectTarget: number;
    history: DailyStats[];
    totalProjectWords: number; // Snapshot of total words
    lastSessionDate: string;
}

const DEFAULT_STATE: GamificationState = {
    dailyGoal: 500,
    projectTarget: 50000,
    history: [],
    totalProjectWords: 0,
    lastSessionDate: new Date().toISOString().split('T')[0]
};

// Singleton state
const state = ref<GamificationState>({ ...DEFAULT_STATE });
const sessionWords = ref(0); // Volatile session counter

export function useGamification() {

    // Load from local storage
    const loadState = () => {
        const stored = localStorage.getItem(STORAGE_KEY);
        if (stored) {
            try {
                const parsed = JSON.parse(stored);
                state.value = { ...DEFAULT_STATE, ...parsed };
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

    // Update total project words absolute value (useful when loading a chapter)
    // This is tricky because we don't know the delta unless we track prev length.
    // We'll rely on `addWords` with delta from the editor.

    const setDailyGoal = (goal: number) => {
        state.value.dailyGoal = goal;
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
        return Math.min(100, Math.max(0, (todayStats.value.wordCount / state.value.dailyGoal) * 100));
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
        // Check back up to X days or until break

        // First, check if streak is alive (today or yesterday has activity)
        // If I haven't written today yet, streak from yesterday is kept.
        // So if today == 0, we treat it as "current", but don't count it for length unless > 0?
        // Actually, widespread convention: Streak is "consecutive days YOU WROTE".
        // If you miss today (so far), your streak is "N days" (from yesterday).
        // If you assume today is 0, start check from yesterday.

        const todayCount = dayMap.get(today) || 0;
        if (todayCount === 0) {
            // Start check from yesterday
            d.setDate(d.getDate() - 1);
        }

        while (true) {
            const dateStr = d.toISOString().split('T')[0];
            const count = dayMap.get(dateStr) || 0;

            if (count > 0) {
                currentStreak++;
                d.setDate(d.getDate() - 1);
            } else {
                // Gap found
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
        dailyGoal: computed(() => state.value.dailyGoal),
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
        setDailyGoal,
        setProjectTarget
    };
}
