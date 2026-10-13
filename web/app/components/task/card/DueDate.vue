<script setup lang="ts">
const props = defineProps<{ dueAt: string; completed: boolean }>();
const { formattedDateTime, now } = useFormattedDateTime(() => props.dueAt);

const overdue = computed(() => {
  if (props.completed) return false;
  return new Date(props.dueAt).getTime() < now.value.getTime();
});
</script>

<template>
  <time
    :datetime="dueAt"
    class="inline-flex items-center gap-1 font-sans"
    :class="{
      'text-light-love dark:text-dark-love': overdue,
      'text-light-pine dark:text-dark-foam': !overdue && !completed,
    }"
  >
    <Icon name="carbon:calendar" class="size-3" aria-hidden="true" />
    {{ formattedDateTime }}
  </time>
</template>
