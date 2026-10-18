<script setup lang="ts">
const {
  type = "button",
  variant = "secondary",
  size = "md",
  fullWidth = false,
  disabled = false,
  loading = false,
} = defineProps<{
  type?: "button" | "submit" | "reset";
  variant?: "primary" | "secondary" | "danger";
  size?: "sm" | "md";
  fullWidth?: boolean;
  disabled?: boolean;
  loading?: boolean;
}>();

const secondaryClasses =
  "border-light-highlight-med hover:bg-light-highlight-med focus-visible:bg-light-highlight-med dark:border-dark-highlight-med dark:hover:bg-dark-highlight-med dark:focus-visible:bg-dark-highlight-med";

const variantClasses = {
  primary:
    "border-light-pine bg-light-pine text-light-base hover:bg-light-pine/90 focus-visible:bg-light-pine/90 dark:border-dark-pine dark:bg-dark-pine dark:text-dark-text dark:hover:bg-dark-pine/90 dark:focus-visible:bg-dark-pine/90",
  secondary: secondaryClasses,
  danger: `${secondaryClasses} text-light-love dark:text-dark-love`,
};

const sizeClasses = {
  sm: "text-sm",
  md: "text-base",
};
</script>

<template>
  <button
    :type="type"
    :disabled="disabled || loading"
    :aria-busy="loading || undefined"
    :class="[
      variantClasses[variant],
      sizeClasses[size],
      { 'w-full': fullWidth },
      loading ? 'disabled:cursor-wait' : 'disabled:cursor-not-allowed',
    ]"
    class="inline-flex cursor-pointer items-center justify-center gap-2 rounded-md border-2 px-4 py-1.5 font-serif transition-colors disabled:opacity-50"
  >
    <slot />
  </button>
</template>
