import { getLocalTimeZone } from "@internationalized/date";

function isValidTimeZone(value: unknown): value is string {
  if (typeof value !== "string" || !value) {
    return false;
  }

  try {
    new Intl.DateTimeFormat("en-GB", { timeZone: value });
    return true;
  } catch {
    return false;
  }
}

export function useTimeZone() {
  const savedTimeZone = useCookie<string | null>("time-zone", {
    default: () => null,
    sameSite: "lax",
    maxAge: 31_536_000,
  });

  const detectedTimeZone = useState<string | null>("time-zone", () =>
    isValidTimeZone(savedTimeZone.value) ? savedTimeZone.value : null,
  );

  const timeZone = computed(() => detectedTimeZone.value ?? "UTC");
  const isTimeZoneReady = computed(() => detectedTimeZone.value !== null);

  onMounted(() => {
    const localTimeZone = getLocalTimeZone();
    detectedTimeZone.value = localTimeZone;
    savedTimeZone.value = localTimeZone;
  });

  return { timeZone, isTimeZoneReady };
}
