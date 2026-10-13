import { useDraftGuard } from '../../lib/drafts';
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
  const [baseline] = useState(() =>
    JSON.stringify([label, amount, unitId, sectionId]),
  );
  const canDiscard = useDraftGuard(
    JSON.stringify([label, amount, unitId, sectionId]) !== baseline,
    busy,
  );
  const [itemId] = useState(() => crypto.randomUUID());
  return (
    <form
      className="pp-panel space-y-3"
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
      <h2 className="text-lg font-semibold">
        {item ? 'Edit amount: ' + item.label : 'Add manual item'}
      </h2>
      {item && (
        <div className="rounded-lg bg-stone-50 p-3 space-y-2">
          <p className="pp-muted">
            Current shopping amount: <strong>{item.display}</strong>
          </p>
          {item.key && (
            <p className="pp-muted">
              Exact generated basis:{' '}
              {item.numerator && item.denominator
                ? item.denominator === '1'
                  ? item.numerator
                  : item.numerator + '/' + item.denominator
                : item.amountText || 'Unknown'}{' '}
              {view.units.find((unit) => unit.id === item.unitId)?.label ?? ''}
            </p>
          )}
        </div>
      )}
      {!item && (
        <FormField label="Item name">
          <input
            autoFocus
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
          autoFocus={!!item}
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
      <div className="pp-save-bar">
        <Button type="submit" disabled={busy}>
          Save shopping item
        </Button>
        <Button
          disabled={busy}
          onClick={() => {
            if (canDiscard()) onCancel();
          }}
        >
          Cancel
        </Button>
      </div>
    </form>
  );
}
