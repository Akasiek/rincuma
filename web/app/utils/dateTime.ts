import { fromDate, toCalendarDate } from "@internationalized/date";

type DateLabelOptions = Pick<Intl.DateTimeFormatOptions, "weekday" | "day" | "month">;

export function getCalendarDate(date: Date, timeZone: string) {
  return toCalendarDate(fromDate(date, timeZone));
}

export function getDayKey(date: Date, timeZone: string): string {
  return getCalendarDate(date, timeZone).toString();
}

export function formatDate(
  date: Date,
  now: Date,
  timeZone: string,
  options: DateLabelOptions = { day: "numeric", month: "short" },
): string {
  const selectedDay = getCalendarDate(date, timeZone);
  const currentDay = getCalendarDate(now, timeZone);

  return new Intl.DateTimeFormat("en-GB", {
    ...options,
    year: selectedDay.year === currentDay.year ? undefined : "numeric",
    timeZone,
  }).format(date);
}

function formatRelativeDate(date: Date, now: Date, timeZone: string): string {
  const selectedDay = getCalendarDate(date, timeZone);
  const currentDay = getCalendarDate(now, timeZone);
  const dayDifference = selectedDay.compare(currentDay);

  if (Math.abs(dayDifference) > 1) {
    return formatDate(date, now, timeZone);
  }

  const relativeDate = new Intl.RelativeTimeFormat("en-GB", { numeric: "auto" }).format(
    dayDifference,
    "day",
  );
  return relativeDate.charAt(0).toUpperCase() + relativeDate.slice(1);
}

export function formatDateTime(value: string, now: Date, timeZone: string): string {
  const date = new Date(value);
  const formattedDate = formatRelativeDate(date, now, timeZone);

  const formattedTime = new Intl.DateTimeFormat("en-GB", {
    timeStyle: "short",
    hourCycle: "h23",
    timeZone,
  }).format(date);

  return `${formattedDate}, ${formattedTime}`;
}
