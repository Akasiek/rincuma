export default defineNuxtRouteMiddleware(async () => {
	const { isLoggedIn } = useUserStore()
	const { loadDictionaries } = useDictionaries()

	if (isLoggedIn) {
		await loadDictionaries()
	}
})