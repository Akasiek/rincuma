export type NavigationSection = "projects" | "tags";

export interface NavigationPreferences {
  projects_open: boolean | null;
  tags_open: boolean | null;
}
