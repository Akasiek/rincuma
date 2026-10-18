import type { InjectionKey, Ref } from "vue";
import type { RouteLocationNormalized } from "vue-router";

export type UnsavedChangesGuard = {
  dirty: Ref<boolean>;
  saving: Ref<boolean>;
  shouldConfirm?: (to: RouteLocationNormalized, from: RouteLocationNormalized) => boolean;
};

export type UnsavedChangesController = {
  register: (guard: UnsavedChangesGuard) => () => void;
  confirm: (guards: UnsavedChangesGuard[]) => Promise<boolean>;
};

export const unsavedChangesKey: InjectionKey<UnsavedChangesController> = Symbol("unsavedChanges");
export const confirmNavigationKey: InjectionKey<() => Promise<boolean>> =
  Symbol("confirmNavigation");
