import type {
  NotificationSettings,
  SaveNotificationSettingsRequest,
} from "#shared/types/notification";

export function useNotificationSettings() {
  const requestFetch = useRequestFetch();

  function getSettings() {
    return requestFetch<NotificationSettings | null>("/api/notifications/settings");
  }

  function saveSettings(body: SaveNotificationSettingsRequest) {
    return requestFetch<NotificationSettings>("/api/notifications/settings", {
      method: "PUT",
      body,
    });
  }

  return { getSettings, saveSettings };
}
