import type { MaybeRefOrGetter } from "vue";

export function useFormattedDateTime(value: MaybeRefOrGetter<string | null | undefined>) {
  const { timeZone, isTimeZoneReady } = useTimeZone();
  const now = useNow({ scheduler: (update) => useIntervalFn(update, 60_000) });

  const formattedDateTime = computed(() => {
    const dateTime = toValue(value);

    return !dateTime || !isTimeZoneReady.value
      ? ""
      : formatDateTime(dateTime, now.value, timeZone.value);
  });

  return { formattedDateTime, timeZone, now, isTimeZoneReady };
}
