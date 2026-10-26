import { parseAbsolute, parseDateTime, toCalendarDate } from "@internationalized/date";
import type { RecurrenceConfig } from "#shared/types/task";

export const recurrenceWeekdays = [
  { value: 1, label: "Monday" },
  { value: 2, label: "Tuesday" },
  { value: 3, label: "Wednesday" },
  { value: 4, label: "Thursday" },
  { value: 5, label: "Friday" },
  { value: 6, label: "Saturday" },
  { value: 7, label: "Sunday" },
];

export const recurrenceOrdinals = [
  { value: 1, label: "First" },
  { value: 2, label: "Second" },
  { value: 3, label: "Third" },
  { value: 4, label: "Fourth" },
  { value: 5, label: "Fifth" },
  { value: -1, label: "Last" },
];

export function cloneRecurrenceConfig(rule: RecurrenceConfig): RecurrenceConfig {
  const pattern = rule.pattern;
  return {
    frequency: rule.frequency,
    interval: rule.interval,
    time_zone: rule.time_zone,
    end: { ...rule.end },
    ...(pattern
      ? {
          pattern:
            pattern.kind === "weekdays" ? { ...pattern, days: [...pattern.days] } : { ...pattern },
        }
      : {}),
  };
}

export function recurrenceSummary(rule: RecurrenceConfig | null | undefined): string {
  if (!rule) {
    return "Does not repeat";
  }

  const unit = { daily: "day", weekly: "week", monthly: "month" }[rule.frequency];
  const interval = rule.interval === 1 ? `Every ${unit}` : `Every ${rule.interval} ${unit}s`;
  const pattern = rule.pattern;
  if (!pattern) {
    return interval;
  }

  if (pattern.kind === "weekdays") {
    const days = recurrenceWeekdays
      .filter((day) => pattern.days.includes(day.value))
      .map((day) => day.label);
    return `${interval} on ${days.join(", ")}`;
  }

  const ordinal = recurrenceOrdinals
    .find((option) => option.value === pattern.ordinal)
    ?.label.toLowerCase();
  const weekday = recurrenceWeekdays.find((option) => option.value === pattern.weekday)?.label;

  return `${interval} on the ${ordinal} ${weekday}`;
}

export function recurrenceReference(
  dueAt: string | null | undefined,
  timeZone: string,
  anchor?: string,
) {
  try {
    const value = anchor
      ? parseDateTime(anchor)
      : dueAt
        ? parseAbsolute(dueAt, timeZone)
        : undefined;

    if (value) {
      const date = toCalendarDate(value);
      const weekday = date.toDate("UTC").getUTCDay();

      return { weekday: weekday === 0 ? 7 : weekday, ordinal: Math.ceil(date.day / 7) };
    }
  } catch {
    return { weekday: 1, ordinal: 1 };
  }

  return { weekday: 1, ordinal: 1 };
}
