<script setup lang="ts">
const { logout } = useAuth();
const userStore = useUserStore();

const { user } = storeToRefs(userStore);
</script>

<template>
  <div>
    <p v-if="user">Logged in as {{ user.email }}. <button @click="logout">Log out</button></p>
    <p v-else>Not authenticated. <NuxtLink to="/login" class="underline">Log in</NuxtLink></p>
  </div>
</template>
