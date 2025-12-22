// Helper: parse duration string to MILLISECONDS
export function parseDurationToMillis(duration?: string): number {
    if (!duration) return 0;
    const lower = duration.toLowerCase().trim();
    const num = parseFloat(lower) || 0;
    
    // Multipliers in milliseconds
    const MINUTE = 60 * 1000;
    const HOUR = 60 * MINUTE;
    const DAY = 24 * HOUR;
    const WEEK = 7 * DAY;
    const MONTH = 30 * DAY; // approx
    const YEAR = 365 * DAY; // approx

    if (lower.includes('minute') || lower.includes('min')) return num * MINUTE;
    if (lower.includes('hour')) return num * HOUR;
    if (lower.includes('week')) return num * WEEK;
    if (lower.includes('month')) return num * MONTH;
    if (lower.includes('year')) return num * YEAR;
    // Default to days if "day" or just number? safest to assume days if explicitly 'day'
    if (lower.includes('day')) return num * DAY;
    
    return 0;
}

// Helper: format milliseconds back to string
export function formatDurationFromMillis(millis: number): string {
    if (!millis || millis <= 0) return '';
    
    const MINUTE = 60 * 1000;
    const HOUR = 60 * MINUTE;
    const DAY = 24 * HOUR;
    const WEEK = 7 * DAY;
    const MONTH = 30 * DAY; // approx
    const YEAR = 365 * DAY; // approx

    // Determine the best unit
    if (millis >= YEAR && millis % YEAR === 0) return `${millis / YEAR} years`;
    if (millis >= MONTH && millis % MONTH === 0) return `${millis / MONTH} months`;
    if (millis >= WEEK && millis % WEEK === 0) return `${millis / WEEK} weeks`;
    if (millis >= DAY && millis % DAY === 0) return `${millis / DAY} days`;
    if (millis >= HOUR && millis % HOUR === 0) return `${millis / HOUR} hours`;
    if (millis >= MINUTE && millis % MINUTE === 0) return `${millis / MINUTE} minutes`;
    
    // Fallback to days with decimals if needed, or just days
    const days = millis / DAY;
    if (days >= 1) return `${parseFloat(days.toFixed(2))} days`;
    
    return `${Math.floor(millis / MINUTE)} minutes`;
}

// Helper: compute gap in days between two ISO dates
export function computeTimeGap(date1?: string, date2?: string): number | null {
    if (!date1 || !date2) return null;
    try {
        const d1 = new Date(date1).getTime();
        const d2 = new Date(date2).getTime();
        if (isNaN(d1) || isNaN(d2)) return null;
        return Math.abs(d2 - d1) / (1000 * 60 * 60 * 24);
    } catch {
        return null;
    }
}
