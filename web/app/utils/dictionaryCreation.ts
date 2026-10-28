export interface DictionaryFormValues {
  name: string;
  description: string;
  color: string | null;
}

export function emptyDictionaryForm(): DictionaryFormValues {
  return { name: "", description: "", color: null };
}

export function validateDictionaryName(value: string | undefined): string | true {
  const name = value?.trim() ?? "";

  if (!name) {
    return "Enter a name.";
  }
  if (new TextEncoder().encode(name).length > 255) {
    return "Name is too long.";
  }

  return true;
}

export function validateDictionaryColor(value: string | null): string | true {
  if (!value || /^#[0-9a-f]{6}$/i.test(value)) {
    return true;
  }

  return "Choose a valid color.";
}
