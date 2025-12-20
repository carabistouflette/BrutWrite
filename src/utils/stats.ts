export interface DailyStats {
    date: string;
    wordCount: number;
}

export function getTodayStats(history: DailyStats[]): DailyStats {
    const today = new Date().toISOString().split('T')[0];
    return history.find(h => h.date === today) || { date: today, wordCount: 0 };
}

export function calculateStreak(history: DailyStats[]): number {
    let currentStreak = 0;
    const dayMap = new Map(history.map(h => [h.date, h.wordCount]));
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
}

export function calculateAverage(history: DailyStats[]): number {
    if (history.length === 0) return 0;
    const total = history.reduce((acc, curr) => acc + curr.wordCount, 0);
    return Math.round(total / history.length);
}

export function getBestDay(history: DailyStats[]): number {
    if (history.length === 0) return 0;
    return Math.max(...history.map(h => h.wordCount));
}
