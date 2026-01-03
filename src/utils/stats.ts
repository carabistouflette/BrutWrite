export interface DailyStats {
  date: string;
  wordCount: number;
}

export function getTodayStats(history: DailyStats[]): DailyStats {
  const today = new Date().toISOString().split('T')[0];
  return history.find((h) => h.date === today) || { date: today, wordCount: 0 };
}

export function calculateStreak(history: DailyStats[]): number {
  let currentStreak = 0;
  const dayMap = new Map(history.map((h) => [h.date, h.wordCount]));
  const today = new Date().toISOString().split('T')[0];

  const d = new Date();
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
  return Math.max(...history.map((h) => h.wordCount));
}

/**
 * Counts words in an HTML string using a robust regex approach.
 * Replaces DOM-based counting to avoid main thread blocking.
 */
export function countWords(html: string): number {
  if (!html) return 0;
  // 1. Replace block tags with newline to ensure word separation
  const withNewlines = html.replace(/<\/(p|div|h[1-6]|li|tr)>/gi, '\n');
  // 2. Strip all tags
  const text = withNewlines.replace(/<[^>]*>/g, ' ');
  // 3. Decode common entities
  const decoded = text
    .replace(/&nbsp;/g, ' ')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&');

  return decoded
    .trim()
    .split(/\s+/)
    .filter((w) => w.length > 0).length;
}
