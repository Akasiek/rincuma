export const useDictionariesStore = defineStore("dictionaries", () => {
	const tags = ref<Tag[]>([])
	const projects = ref<Project[]>([])
	const loaded = ref(false)

	function setDictionaries(
		newTags: Tag[],
		newProjects: Project[],
	) {
		tags.value = newTags
		projects.value = newProjects
		loaded.value = true
	}

	function clear() {
		tags.value = []
		projects.value = []
		loaded.value = false
	}

	return {
		tags,
		projects,
		loaded,
		setDictionaries,
		clear,
	}
})