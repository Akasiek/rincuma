import type {
  RecurrenceConfig,
  RecurrenceEnd,
  RecurrenceFrequency,
  RecurrencePattern,
} from "#shared/types/task";
import { parseDate } from "@internationalized/date";

function positiveInteger(value: number): boolean {
  return Number.isInteger(value) && value > 0 && value <= 4_294_967_295;
}

export function validateRecurrence(
  rule: RecurrenceConfig | null | undefined,
  dueAt: string | null | undefined,
  occurrenceCount = 1,
): true | string {
  return rule && !dueAt
    ? "Set a date and time for a repeating task."
    : validateRecurrenceConfig(rule, occurrenceCount);
}

export function validateRecurrenceConfig(
  rule: RecurrenceConfig | null | undefined,
  occurrenceCount = 1,
): true | string {
  if (!rule) {
    return true;
  }

  return (
    validatePattern(rule.frequency, rule.pattern) ??
    validateInterval(rule.interval) ??
    validateTimeZone(rule.time_zone) ??
    validateEnd(rule.end, occurrenceCount) ??
    true
  );
}

function validatePattern(
  frequency: RecurrenceFrequency,
  pattern: RecurrencePattern | null | undefined,
): string | undefined {
  switch (pattern?.kind) {
    case "weekdays":
      return validateWeekdays(frequency, pattern.days);
    case "monthly_weekday":
      return validateMonthlyWeekday(frequency, pattern.ordinal, pattern.weekday);
    default:
      return undefined;
  }
}

function validateWeekdays(frequency: RecurrenceFrequency, days: number[]): string | undefined {
  const valid =
    frequency === "weekly" &&
    days.length > 0 &&
    new Set(days).size === days.length &&
    days.every(isWeekday);

  return valid ? undefined : "Select at least one valid weekday for a weekly repeat.";
}

function validateMonthlyWeekday(
  frequency: RecurrenceFrequency,
  ordinal: number,
  weekday: number,
): string | undefined {
  const validOrdinal =
    ordinal === -1 || (Number.isInteger(ordinal) && ordinal >= 1 && ordinal <= 5);
  const valid = frequency === "monthly" && validOrdinal && isWeekday(weekday);

  return valid ? undefined : "Select a valid weekday and occurrence for a monthly repeat.";
}

function isWeekday(day: number): boolean {
  return Number.isInteger(day) && day >= 1 && day <= 7;
}

function validateInterval(interval: number): string | undefined {
  return positiveInteger(interval) ? undefined : "Enter a whole repeat interval greater than zero.";
}

function validateTimeZone(timeZone: string): string | undefined {
  if (!timeZone.trim() || /^[+-]/.test(timeZone)) {
    return "Enter an IANA time zone, such as Europe/Warsaw.";
  }

  try {
    new Intl.DateTimeFormat("en-GB", { timeZone });
  } catch {
    return "Enter a valid IANA time zone, such as Europe/Warsaw.";
  }

  return undefined;
}

function validateEnd(end: RecurrenceEnd, occurrenceCount: number): string | undefined {
  switch (end.mode) {
    case "after":
      return validateOccurrenceLimit(end.count, occurrenceCount);
    case "through":
      return validateEndDate(end.date);
    default:
      return undefined;
  }
}

function validateOccurrenceLimit(count: number, occurrenceCount: number): string | undefined {
  const invalid = !positiveInteger(count) || count < occurrenceCount;

  return invalid ? `Enter an occurrence limit of at least ${occurrenceCount}.` : undefined;
}

function validateEndDate(date: string): string | undefined {
  try {
    parseDate(date);
  } catch {
    return "Enter a valid end date.";
  }

  return undefined;
}
