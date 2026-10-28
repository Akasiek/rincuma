<script setup lang="ts">
import {
  ColorAreaArea,
  ColorAreaRoot,
  ColorAreaThumb,
  ColorFieldInput,
  ColorFieldRoot,
  ColorSliderRoot,
  ColorSliderThumb,
  ColorSliderTrack,
  ColorSwatch,
  colorToHex,
  convertToHsb,
  parseColor,
  type Color,
} from "reka-ui";

const model = defineModel<string | null>({ default: null });
defineProps<{ disabled?: boolean }>();
const id = useId();
const color = ref(convertToHsb(parseColor(model.value ?? "#56949f")));

watch(model, (value) => {
  if (value && value.toLowerCase() !== colorToHex(color.value).toLowerCase()) {
    color.value = convertToHsb(parseColor(value));
  }
});

function updateColor(value: Color) {
  color.value = convertToHsb({ ...value, alpha: 1 });
  model.value = colorToHex(color.value);
}
</script>

<template>
  <div class="space-y-3">
    <ColorAreaRoot
      v-slot="{ style }"
      :model-value="color"
      :disabled="disabled"
      color-space="hsb"
      x-channel="saturation"
      y-channel="brightness"
      @update:color="updateColor"
    >
      <ColorAreaArea
        :style="style"
        class="relative h-28 w-full rounded-md"
        aria-label="Saturation and brightness"
      >
        <ColorAreaThumb
          class="size-4 rounded-full border-2 border-white shadow-md ring-1 ring-black/30 focus-visible:outline-2 focus-visible:outline-offset-2"
        />
      </ColorAreaArea>
    </ColorAreaRoot>
    <ColorSliderRoot
      :model-value="color"
      :disabled="disabled"
      color-space="hsb"
      channel="hue"
      class="relative flex h-5 w-full touch-none items-center select-none"
      @update:color="updateColor"
    >
      <ColorSliderTrack class="h-3 grow rounded-full" />
      <ColorSliderThumb
        aria-label="Hue"
        class="block size-5 rounded-full border-2 border-white shadow-md ring-1 ring-black/30 focus-visible:outline-2 focus-visible:outline-offset-2"
      />
    </ColorSliderRoot>
    <div class="flex items-end gap-3">
      <ColorSwatch
        :color="color"
        class="mb-1 size-9 shrink-0 rounded-md border border-black/10 bg-(--reka-color-swatch-color)"
      />
      <ColorFieldRoot
        :model-value="color"
        :disabled="disabled"
        class="min-w-0 flex-1"
        @update:color="updateColor"
      >
        <label
          :for="id"
          class="mb-1 block font-sans text-xs text-light-subtle dark:text-dark-subtle"
        >
          Hex color
        </label>
        <ColorFieldInput
          :id="id"
          class="w-full rounded-md border-2 border-light-highlight-med bg-light-base px-3 py-2 font-sans text-sm dark:border-dark-highlight-med dark:bg-dark-base"
          @keydown.enter.prevent
        />
      </ColorFieldRoot>
    </div>
  </div>
</template>
