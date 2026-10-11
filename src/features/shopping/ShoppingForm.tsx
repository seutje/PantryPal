import { useState } from 'react';
import type {
  ShoppingItem,
  ShoppingView,
  ShoppingOperation,
} from '../../lib/models';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';

export function ShoppingForm({
  item,
  view,
  busy,
  onCancel,
  onSave,
}: {
  item?: ShoppingItem;
  view: ShoppingView;
  busy: boolean;
  onCancel: () => void;
  onSave: (operation: ShoppingOperation) => void;
}) {
  const [label, setLabel] = useState('');
  const [amount, setAmount] = useState(
    item?.adjustment?.amount ??
      (item?.numerator && item.denominator
        ? `${item.numerator}/${item.denominator}`
        : (item?.amountText ?? '')),
  );
  const [unitId, setUnitId] = useState(
    item?.adjustment ? (item.adjustment.unitId ?? '') : (item?.unitId ?? ''),
  );
  const [sectionId, setSectionId] = useState('');
  const [itemId] = useState(() => crypto.randomUUID());
  return (
    <form
      className="space-y-3 rounded-lg border border-stone-300 p-4"
      aria-label={item ? `Edit ${item.label}` : 'Manual shopping item'}
      onSubmit={(event) => {
        event.preventDefault();
        onSave(
          item
            ? {
                kind: 'adjust',
                itemId: item.id,
                amount,
                unitId: unitId || null,
              }
            : {
                kind: 'manual',
                itemId,
                label,
                amount,
                unitId: unitId || null,
                sectionId: sectionId || null,
              },
        );
      }}
    >
      {!item && (
        <FormField label="Item name">
          <input
            required
            maxLength={200}
            value={label}
            disabled={busy}
            onChange={(event) => setLabel(event.target.value)}
          />
        </FormField>
      )}
      <FormField label="Shopping amount">
        <input
          maxLength={120}
          placeholder="e.g. 1 1/2, to taste, or unknown"
          value={amount}
          disabled={busy}
          onChange={(event) => setAmount(event.target.value)}
        />
      </FormField>
      <FormField label="Shopping unit">
        <select
          value={unitId}
          disabled={busy}
          onChange={(event) => setUnitId(event.target.value)}
        >
          <option value="">No unit / unknown</option>
          {view.units
            .filter((unit) => unit.dimension !== 'temperature')
            .map((unit) => (
              <option key={unit.id} value={unit.id}>
                {unit.label}
              </option>
            ))}
        </select>
      </FormField>
      {!item && (
        <FormField label="Shopping section">
          <select
            value={sectionId}
            disabled={busy}
            onChange={(event) => setSectionId(event.target.value)}
          >
            <option value="">Other</option>
            {view.sections
              .filter((section) => !section.retired)
              .map((section) => (
                <option key={section.id} value={section.id}>
                  {section.name}
                </option>
              ))}
          </select>
        </FormField>
      )}
      <p className="text-sm">
        Unknown amounts remain text. Overrides survive refresh until you reset
        them.
      </p>
      <div className="flex gap-2">
        <Button type="submit" disabled={busy}>
          Save shopping item
        </Button>
        <Button disabled={busy} onClick={onCancel}>
          Cancel
        </Button>
      </div>
    </form>
  );
}
