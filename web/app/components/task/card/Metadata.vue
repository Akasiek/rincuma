<script setup lang="ts">
import type { Task } from "#shared/types/task";
import TagBadge from "./TagBadge.vue";
import ProjectBadge from "./ProjectBadge.vue";
import PriorityIcon from "~/components/task/card/PriorityIcon.vue";

const props = defineProps<{ task: Task }>();
const dictionaries = useDictionariesStore();
const project = computed(() =>
  dictionaries.projects.find((project) => project.id === props.task.project_id),
);
const tags = computed(() => dictionaries.tags.filter((tag) => props.task.tag_ids.includes(tag.id)));
</script>

<template>
  <div
    v-if="tags.length || project || task.priority !== 'none'"
    class="flex max-w-full flex-wrap items-center gap-x-2.5 gap-y-1.5 pt-0.5 text-[11px] leading-5 sm:max-w-[45%] sm:justify-end"
  >
    <TagBadge v-for="tag in tags" :key="tag.id" :tag="tag" />
    <ProjectBadge v-if="project" :project="project" />
    <PriorityIcon v-if="task.priority !== 'none'" :priority="task.priority" />
  </div>
</template>
