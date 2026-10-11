// Calendar input/default only. Rust resolves weeks and validates stored dates.
export function localToday(date = new Date()): string {
  return `${date.getFullYear().toString().padStart(4, '0')}-${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')}`;
}
export function dateLabel(iso: string): string {
  // Noon avoids midnight DST transitions; the ISO date never passes through UTC.
  const [year, month, day] = iso.split('-').map(Number);
  const date = new Date(0);
  date.setFullYear(year ?? 2000, (month ?? 1) - 1, day ?? 1);
  date.setHours(12, 0, 0, 0);
  return date.toLocaleDateString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}
