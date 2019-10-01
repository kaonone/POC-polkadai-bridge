export function validateRequired(value: any): string | undefined {
  return !!value ? undefined : 'Is required';
}
