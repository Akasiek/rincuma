<template>
  <aside
    class="hidden h-full min-h-0 shrink-0 border-r border-light-highlight-med bg-light-surface md:block md:w-58 lg:w-64 xl:w-72 dark:border-dark-highlight-med dark:bg-dark-surface"
  >
    <NavigationContent />
  </aside>
  <div
    class="flex shrink-0 items-center justify-between border-b border-light-highlight-med bg-light-surface px-4 py-3 md:hidden dark:border-dark-highlight-med dark:bg-dark-surface"
  >
    <span class="font-serif text-xl font-black">Rincuma</span>
    <NavigationMobileMenu />
  </div>
</template>
