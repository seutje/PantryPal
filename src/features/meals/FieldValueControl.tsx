import type { FieldDefinition, FieldValue } from '../../lib/models';
import { fieldText } from './field-text';
import { FormField } from '../../components/ui/FormField';

export function FieldValueControl({
  field,
  value,
  onChange,
}: {
  field: FieldDefinition;
  value: FieldValue | null;
  onChange: (value: FieldValue | null) => void;
}) {
  if (field.retired)
    return (
      <p>
        {field.label} (retired): {fieldText(field, value)}
      </p>
    );
  return (
    <div className="space-y-1">
      <FormField
        label={`${field.label}${field.unitLabel ? ` (${field.unitLabel})` : ''}`}
      >
        {field.kind === 'boolean' ? (
          <select
            value={value ? String(value.value) : ''}
            onChange={(e) =>
              onChange(
                e.target.value === ''
                  ? null
                  : { kind: 'boolean', value: e.target.value === 'true' },
              )
            }
          >
            <option value="">Unknown</option>
            <option value="true">Yes</option>
            <option value="false">No</option>
          </select>
        ) : field.kind === 'select' ? (
          <select
            value={value ? String(value.value) : ''}
            onChange={(e) =>
              onChange(
                e.target.value
                  ? { kind: 'select', value: e.target.value }
                  : null,
              )
            }
          >
            <option value="">Unknown</option>
            {field.options.map((o) => (
              <option key={o.id} value={o.id}>
                {o.label}
              </option>
            ))}
          </select>
        ) : field.kind === 'text' ? (
          <textarea
            rows={3}
            value={value ? String(value.value) : ''}
            onChange={(event) =>
              onChange(
                event.target.value === ''
                  ? null
                  : { kind: 'text', value: event.target.value },
              )
            }
          />
        ) : (
          <input
            type={field.kind === 'date' ? 'date' : 'number'}
            step={field.kind === 'rating' ? 1 : 'any'}
            min={field.minimum ?? undefined}
            max={field.maximum ?? undefined}
            value={value ? String(value.value) : ''}
            onChange={(e) =>
              onChange(
                e.target.value === ''
                  ? null
                  : field.kind === 'number' || field.kind === 'rating'
                    ? { kind: field.kind, value: e.target.valueAsNumber }
                    : {
                        kind: field.kind as 'date' | 'text',
                        value: e.target.value,
                      },
              )
            }
          />
        )}
      </FormField>
      {field.kind === 'text' && (
        <label className="flex gap-2 text-sm">
          <input
            type="checkbox"
            checked={value !== null}
            onChange={(e) =>
              onChange(e.target.checked ? { kind: 'text', value: '' } : null)
            }
          />
          Known (empty text is allowed)
        </label>
      )}
      {field.description && (
        <p className="text-sm text-stone-600">{field.description}</p>
      )}
    </div>
  );
}
