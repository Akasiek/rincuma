import { fromDate, toCalendarDate } from "@internationalized/date";

export function formatDateTime(value: string, now: Date, timeZone: string): string {
  const date = new Date(value);
  const selectedDay = toCalendarDate(fromDate(date, timeZone));
  const currentDay = toCalendarDate(fromDate(now, timeZone));
  const dayDifference = selectedDay.compare(currentDay);

  let formattedDate: string;
  if (Math.abs(dayDifference) <= 1) {
    const relativeDate = new Intl.RelativeTimeFormat("en-GB", { numeric: "auto" }).format(
      dayDifference,
      "day",
    );
    formattedDate = relativeDate.charAt(0).toUpperCase() + relativeDate.slice(1);
  } else {
    formattedDate = new Intl.DateTimeFormat("en-GB", {
      day: "numeric",
      month: "short",
      year: selectedDay.year === currentDay.year ? undefined : "numeric",
      timeZone,
    }).format(date);
  }

  const formattedTime = new Intl.DateTimeFormat("en-GB", {
    timeStyle: "short",
    hourCycle: "h23",
    timeZone,
  }).format(date);

  return `${formattedDate}, ${formattedTime}`;
}
