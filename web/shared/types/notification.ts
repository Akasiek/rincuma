export type NotificationProvider = "ntfy";

export interface NotificationSettings {
  provider: string;
  enabled: boolean;
  url: string;
  has_authorization: boolean;
}

export interface SaveNotificationSettingsRequest {
  provider: NotificationProvider;
  enabled: boolean;
  url: string;
  authorization?: string | null;
}
