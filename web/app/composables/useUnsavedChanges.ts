import { unsavedChangesKey, type UnsavedChangesGuard } from "~/utils/navigation";

export function useUnsavedChanges(guard: UnsavedChangesGuard) {
  const controller = inject(unsavedChangesKey);
  if (!controller) {
    throw new Error("Unsaved changes confirmation must be provided by the app.");
  }
  const unregister = controller.register(guard);

  onScopeDispose(unregister);

  return () => controller.confirm([guard]);
}
