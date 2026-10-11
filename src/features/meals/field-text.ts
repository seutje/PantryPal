import type { FieldDefinition, FieldValue } from '../../lib/models';
export function fieldText(field: FieldDefinition, value: FieldValue | null) {
  if (!value) return 'Unknown';
  if (value.kind === 'boolean') return value.value ? 'Yes' : 'No';
  if (value.kind === 'select')
    return (
      field.options.find((o) => o.id === value.value)?.label ?? value.value
    );
  return String(value.value);
}
