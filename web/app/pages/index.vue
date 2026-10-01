<script setup lang="ts">
const { logout } = useAuth();
const userStore = useUserStore();
const dictsStore = useDictionariesStore();

const { projects, tags, loaded: dictsLoaded } = storeToRefs(dictsStore);
const { user } = storeToRefs(userStore);
</script>

<template>
  <div>
    <template v-if="user">
      <p>Logged in as {{ user.email }}. <button @click="logout">Log out</button></p>

      <template v-if="dictsLoaded">
        <hr class="my-5" />

        <h2>Projects</h2>
        <ul>
          <li v-for="project in projects">
            {{ project.name }}
          </li>
        </ul>

        <hr class="my-5" />

        <h2>Tags</h2>
        <ul>
          <li v-for="tag in tags">
            {{ tag.name }}
          </li>
        </ul>
      </template>
    </template>
    <template v-else>
      <p>Not authenticated. <NuxtLink to="/login" class="underline">Log in</NuxtLink></p>
    </template>
  </div>
</template>
