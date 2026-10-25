export const profileFieldValidation = {
  validateOnBlur: false,
  validateOnChange: false,
  validateOnInput: false,
  validateOnModelUpdate: false,
};

export type ProfileFeedback = { kind: "success" | "error"; message: string } | null;

export function validateProfileEmail(value: string | undefined) {
  const email = (value ?? "").trim().toLowerCase();
  const bytes = new TextEncoder().encode(email).length;

  return (bytes <= 254 && /^[^\s@]+@[^\s@]+$/.test(email)) || "Enter a valid email address.";
}

export function validateProfilePassword(value: string | undefined) {
  if (!value) {
    return "Enter a password.";
  }

  return new TextEncoder().encode(value).length <= 1024 || "The password is too long.";
}

export function profileErrorStatus(error: unknown) {
  return (error as { response?: { status?: number } })?.response?.status;
}

export function profileErrorMessage(error: unknown) {
  const status = profileErrorStatus(error);
  if (status === 401) {
    return "Your session has expired. Log in again before saving.";
  }
  if (status === 412) {
    return "Your profile changed in another request. Reload this page before saving again.";
  }

  return "Could not save your changes. Please try again.";
}
