<script setup lang="ts">
import {
  CollapsibleContent,
  CollapsibleRoot,
  CollapsibleTrigger,
  ToggleGroupItem,
  ToggleGroupRoot,
} from "reka-ui";

const model = defineModel<string | null>({ default: null });
const props = defineProps<{
  id: string;
  invalid?: boolean;
  describedby?: string;
  disabled?: boolean;
}>();
const pickerOpen = ref(false);
const colors = [
  { name: "Rose", value: "#b4637a" },
  { name: "Gold", value: "#ea9d34" },
  { name: "Coral", value: "#d7827e" },
  { name: "Blue", value: "#286983" },
  { name: "Teal", value: "#56949f" },
  { name: "Purple", value: "#907aa9" },
  { name: "Green", value: "#6c8a56" },
];

const customColor = computed(() => {
  const value = model.value?.toLowerCase();
  return value && !colors.some((color) => color.value === value) ? value : null;
});

function updateColor(value: string | string[]) {
  if (typeof value === "string") {
    model.value = value === "none" || !value ? null : value;
  }
}
</script>

<template>
  <CollapsibleRoot
    :id="props.id"
    v-model:open="pickerOpen"
    :disabled="disabled"
    :aria-invalid="invalid"
    :aria-describedby="describedby"
  >
    <div class="flex flex-wrap items-center gap-1.5">
      <ToggleGroupRoot
        :model-value="model?.toLowerCase() ?? 'none'"
        type="single"
        :disabled="disabled"
        aria-label="Color"
        class="flex flex-wrap gap-1.5"
        @update:model-value="updateColor"
      >
        <ToggleGroupItem
          value="none"
          aria-label="No color"
          title="No color"
          class="flex size-7 cursor-pointer items-center justify-center rounded-full border-2 border-light-highlight-med text-light-subtle data-[state=on]:ring-2 data-[state=on]:ring-light-pine data-[state=on]:ring-offset-2 data-[state=on]:ring-offset-light-surface dark:border-dark-highlight-med dark:text-dark-subtle dark:data-[state=on]:ring-dark-foam dark:data-[state=on]:ring-offset-dark-surface"
        >
          <Icon name="carbon:subtract" class="size-4" aria-hidden="true" />
        </ToggleGroupItem>
        <ToggleGroupItem
          v-for="color in colors"
          :key="color.value"
          :value="color.value"
          :aria-label="color.name"
          :title="color.name"
          :style="{ backgroundColor: color.value }"
          class="size-7 cursor-pointer rounded-full border-2 border-transparent data-[state=on]:ring-2 data-[state=on]:ring-light-pine data-[state=on]:ring-offset-2 data-[state=on]:ring-offset-light-surface dark:data-[state=on]:ring-dark-foam dark:data-[state=on]:ring-offset-dark-surface"
        />
      </ToggleGroupRoot>
      <CollapsibleTrigger
        type="button"
        aria-label="Custom color"
        title="Custom color"
        :style="{ backgroundColor: customColor ?? undefined }"
        :class="
          customColor
            ? 'ring-2 ring-light-pine ring-offset-2 ring-offset-light-surface dark:ring-dark-foam dark:ring-offset-dark-surface'
            : ''
        "
        class="flex size-7 shrink-0 cursor-pointer items-center justify-center rounded-full border-2 border-light-highlight-med dark:border-dark-highlight-med"
      >
        <Icon
          v-if="!customColor"
          name="carbon:color-palette"
          class="size-4 text-light-subtle dark:text-dark-subtle"
          aria-hidden="true"
        />
      </CollapsibleTrigger>
    </div>
    <CollapsibleContent class="pt-4">
      <FormColorPicker v-model="model" :disabled="disabled" />
    </CollapsibleContent>
  </CollapsibleRoot>
</template>
