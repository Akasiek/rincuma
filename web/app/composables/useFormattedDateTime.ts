import { getLocalTimeZone } from "@internationalized/date";
import type { MaybeRefOrGetter } from "vue";

export function useFormattedDateTime(value: MaybeRefOrGetter<string | null | undefined>) {
  const timeZone = ref("UTC");
  const now = useNow({ scheduler: (update) => useIntervalFn(update, 60_000) });

  onMounted(() => {
    timeZone.value = getLocalTimeZone();
  });

  const formattedDateTime = computed(() => {
    const dateTime = toValue(value);
    if (!dateTime) return "";
    return formatDateTime(dateTime, now.value, timeZone.value);
  });

  return { formattedDateTime, timeZone };
}
