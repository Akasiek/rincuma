<script setup lang="ts">
defineProps<{ title: string }>();
</script>

<template>
  <TaskWorkspace>
    <main class="min-h-0 min-w-0 flex-1 overflow-hidden">
      <AppScrollArea :label="title" class="h-full">
        <div class="px-4 py-8 pb-28 sm:px-8 lg:px-12 lg:py-12 lg:pb-28">
          <div class="mx-auto max-w-4xl">
            <header class="mb-8">
              <h1 class="text-2xl font-black tracking-tight sm:text-3xl">{{ title }}</h1>
            </header>
            <slot />
          </div>
        </div>
      </AppScrollArea>
    </main>
  </TaskWorkspace>
</template>
