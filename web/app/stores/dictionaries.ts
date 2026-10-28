export const useDictionariesStore = defineStore("dictionaries", () => {
  const tags = ref<Tag[]>([]);
  const projects = ref<Project[]>([]);
  const loaded = ref(false);

  function setDictionaries(newTags: Tag[], newProjects: Project[]) {
    tags.value = newTags;
    projects.value = newProjects;
    loaded.value = true;
  }

  function addProject(project: Project) {
    projects.value = projects.value
      .filter((item) => item.id !== project.id)
      .concat({ ...projects.value.find((item) => item.id === project.id), ...project })
      .sort((left, right) => left.name.localeCompare(right.name) || left.id - right.id);
  }

  function addTag(tag: Tag) {
    tags.value = tags.value
      .filter((item) => item.id !== tag.id)
      .concat({ ...tags.value.find((item) => item.id === tag.id), ...tag })
      .sort((left, right) => left.name.localeCompare(right.name) || left.id - right.id);
  }

  function clear() {
    tags.value = [];
    projects.value = [];
    loaded.value = false;
  }

  return {
    tags,
    projects,
    loaded,
    setDictionaries,
    addProject,
    addTag,
    clear,
  };
});
