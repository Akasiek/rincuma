export type SelectOptionValue = string | number | null;
export type SelectModelValue = SelectOptionValue | (string | number)[];

export interface SelectOption {
  value: SelectOptionValue;
  label: string;
  disabled?: boolean;
}
