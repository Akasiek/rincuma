import type { RecurrenceConfig } from "#shared/types/task";
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
  if (!positiveInteger(rule.interval)) {
    return "Enter a whole repeat interval greater than zero.";
  }
  if (!rule.time_zone.trim() || /^[+-]/.test(rule.time_zone)) {
    return "Enter an IANA time zone, such as Europe/Warsaw.";
  }

  try {
    new Intl.DateTimeFormat("en-GB", { timeZone: rule.time_zone });
  } catch {
    return "Enter a valid IANA time zone, such as Europe/Warsaw.";
  }

  if (rule.end.mode === "after") {
    if (!positiveInteger(rule.end.count) || rule.end.count < occurrenceCount) {
      return `Enter an occurrence limit of at least ${occurrenceCount}.`;
    }
  }

  if (rule.end.mode === "through") {
    try {
      parseDate(rule.end.date);
    } catch {
      return "Enter a valid end date.";
    }
  }

  return true;
}
