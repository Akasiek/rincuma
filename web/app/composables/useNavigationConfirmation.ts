import {
  confirmNavigationKey,
  unsavedChangesKey,
  type UnsavedChangesGuard,
} from "~/utils/navigation";

export function useNavigationConfirmation() {
  const router = useRouter();
  const userStore = useUserStore();
  const guards = new Set<UnsavedChangesGuard>();
  const pendingGuards = shallowRef<UnsavedChangesGuard[]>([]);
  const open = ref(false);
  const saving = computed(() => pendingGuards.value.some((guard) => guard.saving.value));
  let resolveConfirmation: ((leave: boolean) => void) | undefined;

  function finish(leave: boolean) {
    if (leave && saving.value) {
      return;
    }

    const resolve = resolveConfirmation;
    resolveConfirmation = undefined;

    open.value = false;
    pendingGuards.value = [];

    resolve?.(leave);
  }

  async function confirm(candidates: UnsavedChangesGuard[]) {
    const changed = candidates.filter((guard) => guard.dirty.value || guard.saving.value);
    if (changed.length === 0) {
      return true;
    }

    finish(false);
    pendingGuards.value = changed;
    open.value = true;

    return new Promise<boolean>((resolve) => {
      resolveConfirmation = resolve;
    });
  }

  function register(guard: UnsavedChangesGuard) {
    guards.add(guard);
    return () => {
      guards.delete(guard);
      if (pendingGuards.value.includes(guard)) {
        finish(false);
      }
    };
  }

  function provideConfirmation() {
    const confirmNavigation = () => confirm([...guards]);
    provide(unsavedChangesKey, { register, confirm });
    provide(confirmNavigationKey, confirmNavigation);
  }

  function registerNavigationGuard() {
    const removeRouteGuard = router.beforeEach((to, from) => {
      if (!userStore.isLoggedIn) {
        return true;
      }
      return confirm([...guards].filter((guard) => guard.shouldConfirm?.(to, from) ?? true));
    });

    onScopeDispose(() => {
      removeRouteGuard();
      finish(false);
    });
  }

  function watchConfirmationState() {
    watch(open, (value) => {
      if (!value) {
        finish(false);
      }
    });

    watchEffect(() => {
      const unchanged = pendingGuards.value.every(
        (guard) => !guard.dirty.value && !guard.saving.value,
      );
      if (open.value && unchanged) {
        finish(true);
      }
    });
  }

  provideConfirmation();
  registerNavigationGuard();
  watchConfirmationState();

  return { open, saving, finish };
}
