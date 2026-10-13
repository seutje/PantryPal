import { DraftContext, useDraftGuard } from '../../lib/drafts';
import { Actions } from '../../components/ui/Actions';
import { useContext, useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import type { FieldDefinition, FieldInput, FieldKind } from '../../lib/models';
import { createField, updateField, retireField } from '../../lib/ipc';
import { FormField } from '../../components/ui/FormField';
import { Button } from '../../components/ui/Button';

const kindLabels: Record<FieldKind, string> = {
  boolean: 'Yes / no',
  select: 'Choice',
  number: 'Number',
  date: 'Date',
  text: 'Text',
  rating: 'Rating',
};

function DefinitionEditor({
  field,
  onDone,
}: {
  field: FieldDefinition | null;
  onDone: () => void;
}) {
  const [input, setInput] = useState<FieldInput>(
    field
      ? {
          label: field.label,
          kind: field.kind,
          description: field.description,
          options: field.options,
          minimum: field.minimum,
          maximum: field.maximum,
          unitLabel: field.unitLabel,
          sortOrder: field.sortOrder,
        }
      : {
          label: '',
          kind: 'boolean',
          description: '',
          options: [],
          minimum: null,
          maximum: null,
          unitLabel: null,
          sortOrder: 0,
        },
  );
  const [baseline] = useState(() => JSON.stringify(input));
  const save = useMutation({
    mutationFn: () =>
      field ? updateField(field.id, input) : createField(input),
    onSuccess: onDone,
  });
  const canDiscard = useDraftGuard(
    JSON.stringify(input) !== baseline,
    save.isPending,
  );
  const change = <K extends keyof FieldInput>(key: K, value: FieldInput[K]) =>
    setInput((i) => ({ ...i, [key]: value }));
  const numeric = input.kind === 'number' || input.kind === 'rating';
  return (
    <form
      className="space-y-3 rounded-lg border border-stone-300 p-4"
      onSubmit={(e) => {
        e.preventDefault();
        save.mutate();
      }}
    >
      <fieldset disabled={save.isPending} className="space-y-3">
        <legend className="sr-only">Custom field definition</legend>
        <h3 className="font-semibold">
          {field ? 'Edit custom field' : 'New custom field'}
        </h3>
        <FormField label="Field label">
          <input
            autoFocus
            required
            value={input.label}
            onChange={(e) => change('label', e.target.value)}
          />
        </FormField>
        <FormField label="Field kind">
          <select
            disabled={!!field}
            value={input.kind}
            onChange={(e) => {
              const kind = e.target.value as FieldKind;
              setInput((i) => ({
                ...i,
                kind,
                options: [],
                minimum: kind === 'rating' ? 1 : null,
                maximum: kind === 'rating' ? 5 : null,
                unitLabel: null,
              }));
            }}
          >
            {(
              ['boolean', 'select', 'number', 'date', 'text', 'rating'] as const
            ).map((k) => (
              <option key={k} value={k}>
                {kindLabels[k]}
              </option>
            ))}
          </select>
        </FormField>
        <FormField label="Field description">
          <textarea
            value={input.description}
            onChange={(e) => change('description', e.target.value)}
          />
        </FormField>
        {numeric && (
          <div className="grid gap-3 md:grid-cols-3">
            {(['minimum', 'maximum'] as const).map((key) => (
              <FormField
                key={key}
                label={key === 'minimum' ? 'Minimum' : 'Maximum'}
              >
                <input
                  disabled={!!field}
                  type="number"
                  step={input.kind === 'rating' ? 1 : 'any'}
                  required={input.kind === 'rating'}
                  value={input[key] ?? ''}
                  onChange={(e) =>
                    change(
                      key,
                      e.target.value === '' ? null : e.target.valueAsNumber,
                    )
                  }
                />
              </FormField>
            ))}
            <FormField label="Field unit label">
              <input
                disabled={!!field}
                value={input.unitLabel ?? ''}
                onChange={(e) => change('unitLabel', e.target.value || null)}
              />
            </FormField>
          </div>
        )}
        {input.kind === 'select' && (
          <div className="space-y-2">
            {input.options.map((o, index) => (
              <div key={o.id} className="flex items-end gap-2">
                <FormField label={`Choice ${index + 1}`}>
                  <input
                    required
                    value={o.label}
                    onChange={(e) =>
                      change(
                        'options',
                        input.options.map((p) =>
                          p.id === o.id ? { ...p, label: e.target.value } : p,
                        ),
                      )
                    }
                  />
                </FormField>
                {!field && (
                  <Button
                    onClick={() =>
                      change(
                        'options',
                        input.options.filter((p) => p.id !== o.id),
                      )
                    }
                  >
                    Remove choice {index + 1}
                  </Button>
                )}
              </div>
            ))}
            {!field && (
              <Button
                onClick={() =>
                  change('options', [
                    ...input.options,
                    { id: crypto.randomUUID(), label: '' },
                  ])
                }
              >
                Add choice
              </Button>
            )}
          </div>
        )}
        <FormField label="Presentation order">
          <input
            type="number"
            min="0"
            step="1"
            required
            value={input.sortOrder}
            onChange={(e) => change('sortOrder', e.target.valueAsNumber)}
          />
        </FormField>
        {field && (
          <p className="text-sm text-stone-600">
            Labels can change safely. Kind, choice identifiers and validation
            limits stay fixed to protect saved values. Create a new field for a
            different structure.
          </p>
        )}
        {save.error && <p role="alert">{save.error.message}</p>}
        <div className="flex gap-2">
          <Button type="submit" disabled={save.isPending}>
            Save field
          </Button>
          <Button
            disabled={save.isPending}
            onClick={() => {
              if (canDiscard()) onDone();
            }}
          >
            Close field editor
          </Button>
        </div>
      </fieldset>
    </form>
  );
}

export function FieldManager({
  fields,
  onClose,
}: {
  fields: FieldDefinition[];
  onClose: () => void;
}) {
  const drafts = useContext(DraftContext);
  const client = useQueryClient();
  const [editing, setEditing] = useState<FieldDefinition | null | undefined>();
  const [retiring, setRetiring] = useState<FieldDefinition | null>(null);
  const refresh = () => {
    setEditing(undefined);
    setRetiring(null);
    void client.invalidateQueries({ queryKey: ['meal-library'] });
  };
  const retire = useMutation({
    mutationFn: (id: string) => retireField(id, true),
    onSuccess: refresh,
  });
  return (
    <section className="space-y-4" aria-label="Manage custom fields">
      <div className="flex gap-2">
        <h2 className="text-xl font-semibold">Custom field definitions</h2>
        <Button
          onClick={() => {
            if (drafts.canLeave()) onClose();
          }}
        >
          Back to meals
        </Button>
      </div>
      <p>
        Unknown is the default. Retiring a field keeps existing values available
        in meal details and filters.
      </p>
      <div className={editing !== undefined ? 'pp-catalog-layout' : undefined}>
        <div className="space-y-4">
          <Button
            variant="primary"
            disabled={editing !== undefined}
            onClick={() => setEditing(null)}
          >
            New custom field
          </Button>
          <div className="pp-panel overflow-x-auto">
            <table className="w-full text-left text-sm">
              <caption className="sr-only">
                Household custom field definitions
              </caption>
              <thead>
                <tr>
                  {['Field', 'Type', 'Description', 'Actions'].map((label) => (
                    <th key={label} scope="col" className="p-3">
                      {label}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {fields.map((f) => (
                  <tr key={f.id} className="border-t border-stone-200">
                    <th scope="row" className="p-3 font-medium">
                      {f.label}
                      {f.retired && (
                        <span className="pp-badge ml-2">Retired</span>
                      )}
                    </th>
                    <td className="p-3">{kindLabels[f.kind]}</td>
                    <td className="p-3 pp-muted">
                      {f.description || 'No description'}
                    </td>
                    <td className="p-3">
                      {!f.retired && (
                        <div className="flex gap-2">
                          <Button
                            variant="quiet"
                            onClick={() => {
                              if (drafts.canLeave()) setEditing(f);
                            }}
                          >
                            Edit {f.label}
                          </Button>
                          <Actions label={'More actions for ' + f.label}>
                            <Button
                              variant="danger"
                              disabled={editing !== undefined}
                              onClick={() => setRetiring(f)}
                            >
                              Retire {f.label}
                            </Button>
                          </Actions>
                        </div>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {fields.length === 0 && (
              <p className="pp-muted">
                Add a field for details your household uses when choosing meals.
              </p>
            )}
          </div>
        </div>
        {editing !== undefined && (
          <aside className="pp-catalog-editor">
            <DefinitionEditor
              key={editing?.id ?? 'new'}
              field={editing}
              onDone={refresh}
            />
          </aside>
        )}
      </div>
      {retiring && (
        <div role="group" aria-label="Confirm field retirement">
          <p>
            Retire {retiring.label}? Existing meal values are kept. New meals
            cannot use it.
          </p>
          <Button
            variant="danger"
            disabled={retire.isPending}
            onClick={() => retire.mutate(retiring.id)}
          >
            Confirm field retirement
          </Button>
          <Button disabled={retire.isPending} onClick={() => setRetiring(null)}>
            Keep field
          </Button>
        </div>
      )}
      {retire.error && <p role="alert">{retire.error.message}</p>}
    </section>
  );
}
