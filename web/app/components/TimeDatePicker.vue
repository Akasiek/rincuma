<script setup lang="ts">
import {
  CalendarDate,
  Time,
  fromDate,
  getLocalTimeZone,
  toCalendarDate,
  toCalendarDateTime,
  toZoned,
  type DateValue,
} from "@internationalized/date";
import {
  CalendarCell,
  CalendarCellTrigger,
  CalendarGrid,
  CalendarGridBody,
  CalendarGridHead,
  CalendarGridRow,
  CalendarHeadCell,
  CalendarHeader,
  CalendarHeading,
  CalendarNext,
  CalendarPrev,
  CalendarRoot,
  TimeFieldInput,
  TimeFieldRoot,
  type TimeValue,
} from "reka-ui";

const dateTime = defineModel<string | null>({ default: null });
const open = ref(false);
const selectedDate = shallowRef<DateValue>();
const selectedTime = shallowRef<TimeValue>();
const error = ref("");
const timeZone = ref("UTC");
const now = useNow({ scheduler: (update) => useIntervalFn(update, 60_000) });

onMounted(() => {
  timeZone.value = getLocalTimeZone();
});

const formattedDateTime = computed(() => {
  if (!dateTime.value) return "";
  const date = new Date(dateTime.value);
  const selectedDay = toCalendarDate(fromDate(date, timeZone.value));
  const currentDay = toCalendarDate(fromDate(now.value, timeZone.value));
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
      dateStyle: "medium",
      timeZone: timeZone.value,
    }).format(date);
  }

  const formattedTime = new Intl.DateTimeFormat("en-GB", {
    timeStyle: "short",
    hourCycle: "h23",
    timeZone: timeZone.value,
  }).format(date);

  return `${formattedDate}, ${formattedTime}`;
});

const canApply = computed(() => !!selectedDate.value && !!selectedTime.value);

watch(open, (isOpen) => {
  if (!isOpen) return;
  error.value = "";
  const date = new Date(dateTime.value ?? Date.now());
  selectedDate.value = new CalendarDate(date.getFullYear(), date.getMonth() + 1, date.getDate());
  selectedTime.value = new Time(date.getHours(), date.getMinutes());
});

function apply() {
  if (!selectedDate.value || !selectedTime.value) return;
  try {
    const calendarDateTime = toCalendarDateTime(
      selectedDate.value,
      new Time(selectedTime.value.hour, selectedTime.value.minute),
    );
    dateTime.value = toZoned(calendarDateTime, timeZone.value, "reject").toDate().toISOString();
    open.value = false;
  } catch {
    error.value =
      "This time is ambiguous or unavailable due to daylight saving. Choose another time.";
  }
}

function clear() {
  dateTime.value = null;
  open.value = false;
}
</script>

<template>
  <AppModal v-model:open="open" title="Date and time" size="sm">
    <template #trigger>
      <button
        type="button"
        :aria-label="
          formattedDateTime ? `Change date and time: ${formattedDateTime}` : 'Set date and time'
        "
        class="flex cursor-pointer items-center gap-2 rounded-md border-2 border-transparent px-2.5 py-1.5 hover:bg-light-highlight-med focus:border-light-text focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-light-text dark:hover:bg-dark-highlight-med dark:focus:border-dark-text dark:focus-visible:outline-dark-text"
      >
        <Icon name="carbon:calendar" class="size-5" aria-hidden="true" />
        <span v-if="formattedDateTime" class="font-sans text-sm">{{ formattedDateTime }}</span>
      </button>
    </template>

    <div class="p-4 sm:p-6">
      <div class="flex flex-col gap-5">
        <CalendarRoot
          v-model="selectedDate"
          v-slot="{ grid, weekDays }"
          :week-starts-on="1"
          locale="en-GB"
          calendar-label="Date"
          fixed-weeks
          prevent-deselect
          class="min-w-0"
        >
          <CalendarHeader class="mb-3 flex items-center justify-between gap-2">
            <CalendarPrev
              aria-label="Previous month"
              class="cursor-pointer rounded-md p-1.5 hover:bg-light-highlight-med focus-visible:outline-2 focus-visible:outline-light-love dark:hover:bg-dark-highlight-med"
            >
              <Icon name="carbon:chevron-left" class="size-4" aria-hidden="true" />
            </CalendarPrev>
            <CalendarHeading class="text-sm font-bold" />
            <CalendarNext
              aria-label="Next month"
              class="cursor-pointer rounded-md p-1.5 hover:bg-light-highlight-med focus-visible:outline-2 focus-visible:outline-light-love dark:hover:bg-dark-highlight-med"
            >
              <Icon name="carbon:chevron-right" class="size-4" aria-hidden="true" />
            </CalendarNext>
          </CalendarHeader>

          <CalendarGrid
            v-for="month in grid"
            :key="month.value.toString()"
            class="w-full table-fixed border-collapse"
          >
            <CalendarGridHead>
              <CalendarGridRow>
                <CalendarHeadCell
                  v-for="day in weekDays"
                  :key="day"
                  class="pb-2 text-xs font-normal text-light-subtle dark:text-dark-subtle"
                >
                  {{ day }}
                </CalendarHeadCell>
              </CalendarGridRow>
            </CalendarGridHead>
            <CalendarGridBody>
              <CalendarGridRow v-for="(week, index) in month.rows" :key="index">
                <CalendarCell
                  v-for="day in week"
                  :key="day.toString()"
                  :date="day"
                  class="p-0.5 text-center font-sans"
                >
                  <CalendarCellTrigger
                    :day="day"
                    :month="month.value"
                    class="flex h-10 w-full cursor-pointer items-center justify-center rounded-md border border-transparent text-sm hover:bg-light-highlight-med focus-visible:outline-2 focus-visible:outline-light-love data-[outside-view]:opacity-40 data-[selected]:bg-light-love data-[selected]:text-light-surface data-[today]:border-light-highlight-med data-[today]:font-black dark:hover:bg-dark-highlight-med dark:data-[selected]:bg-dark-love dark:data-[selected]:text-dark-base dark:data-[today]:border-dark-highlight-med"
                  />
                </CalendarCell>
              </CalendarGridRow>
            </CalendarGridBody>
          </CalendarGrid>
        </CalendarRoot>

        <div
          class="flex items-center justify-between gap-4 border-t border-light-highlight-med pt-4 dark:border-dark-highlight-med"
        >
          <div class="min-w-0">
            <p class="font-bold">Time</p>
            <p class="mt-1 text-xs break-words text-light-subtle dark:text-dark-subtle">
              {{ timeZone }}
            </p>
          </div>
          <TimeFieldRoot
            v-model="selectedTime"
            v-slot="{ segments }"
            :hour-cycle="24"
            granularity="minute"
            locale="en-GB"
            aria-label="Time"
            class="flex shrink-0 items-center justify-center rounded-md border-2 border-light-highlight-med px-3 py-2 text-xl dark:border-dark-highlight-med"
          >
            <TimeFieldInput
              v-for="segment in segments"
              :key="segment.part"
              :part="segment.part"
              class="rounded-sm px-0.5 font-sans tabular-nums focus:bg-light-highlight-med focus:outline-none dark:focus:bg-dark-highlight-med"
            >
              {{ segment.value }}
            </TimeFieldInput>
          </TimeFieldRoot>
        </div>
      </div>

      <p v-if="error" role="alert" class="mt-4 text-sm text-light-love dark:text-dark-love">
        {{ error }}
      </p>

      <div
        class="mt-5 flex flex-wrap items-center gap-2 border-t border-light-highlight-med pt-4 dark:border-dark-highlight-med"
      >
        <Button v-if="dateTime" @click="clear">Clear</Button>
        <div class="ml-auto flex gap-2">
          <Button @click="open = false">Cancel</Button>
          <Button
            :disabled="!canApply"
            class="disabled:cursor-not-allowed disabled:opacity-50"
            @click="apply"
            >Apply</Button
          >
        </div>
      </div>
    </div>
  </AppModal>
</template>
