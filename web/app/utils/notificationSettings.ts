import type {
  NotificationSettings,
  SaveNotificationSettingsRequest,
} from "#shared/types/notification";

export interface NotificationFormValues {
  provider: string;
  enabled: boolean;
  url: string;
  authorization: string;
}

export function notificationFormValues(
  settings: NotificationSettings | null,
): NotificationFormValues {
  return {
    provider: settings?.provider ?? "ntfy",
    enabled: settings?.enabled ?? false,
    url: settings?.url ?? "",
    authorization: "",
  };
}

export function validateNtfyTopicUrl(value: string): true | string {
  const input = value?.trim() ?? "";
  if (!input) {
    return "Enter your ntfy topic URL.";
  }
  if (new TextEncoder().encode(input).length > 2048) {
    return "The topic URL must not exceed 2048 bytes.";
  }

  try {
    const url = new URL(input);
    if (!["http:", "https:"].includes(url.protocol) || !url.hostname) {
      return "Use an HTTP or HTTPS URL.";
    }
    if (!url.pathname.replaceAll("/", "")) {
      return "Include a topic in the URL, for example https://ntfy.example.com/tasks.";
    }
    if (url.username || url.password || input.includes("?") || input.includes("#")) {
      return "Use a URL without credentials, query parameters or fragments.";
    }
    return true;
  } catch {
    return "Enter a valid topic URL.";
  }
}

export function validateNotificationAuthorization(value: string): true | string {
  if (new TextEncoder().encode(value ?? "").length > 4096) {
    return "Authorization must not exceed 4096 bytes.";
  }

  const hasControlCharacters = Array.from(value ?? "").some((character) => {
    const code = character.charCodeAt(0);
    return code === 127 || (code < 32 && code !== 9);
  });

  if (hasControlCharacters) {
    return "Authorization must be a single line without control characters.";
  }

  return true;
}

export function notificationSettingsRequest(
  values: NotificationFormValues,
  removeAuthorization: boolean,
): SaveNotificationSettingsRequest {
  if (values.provider !== "ntfy") {
    throw new Error("Choose a supported provider.");
  }

  let authorization: string | null = values.authorization || null;
  if (removeAuthorization) {
    authorization = "";
  }

  return {
    provider: values.provider,
    enabled: values.enabled,
    url: values.url.trim(),
    authorization,
  };
}
