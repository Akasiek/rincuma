<script setup lang="ts">
defineOptions({ name: "EditProfilePage" });
useHead({ title: "Edit profile" });

const detailsDirty = ref(false);
const passwordDirty = ref(false);
const detailsSaving = ref(false);
const passwordSaving = ref(false);
const saving = computed(() => detailsSaving.value || passwordSaving.value);

useUnsavedChanges({
  dirty: computed(() => detailsDirty.value || passwordDirty.value),
  saving,
  shouldConfirm: (to, from) => to.path !== from.path,
});
</script>

<template>
  <AppContentPage title="Edit profile">
    <div class="max-w-2xl space-y-8">
      <ProfileDetailsForm
        v-model:dirty="detailsDirty"
        v-model:saving="detailsSaving"
        :disabled="saving"
      />
      <ProfilePasswordForm
        v-model:dirty="passwordDirty"
        v-model:saving="passwordSaving"
        :disabled="saving"
      />
    </div>
  </AppContentPage>
</template>
