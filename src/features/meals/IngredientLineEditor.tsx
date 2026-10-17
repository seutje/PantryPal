import { useRef, useState } from 'react';
import type { Catalog, Ingredient, IngredientDraft } from '../../lib/models';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { IngredientSearch } from './IngredientSearch';

export function IngredientLineEditor({
  line,
  index,
  count,
  catalog,
  ingredients,
  busy,
  review = false,
  onChange,
  onMove,
  onAdd,
  onSelect,
  onBusyChange,
  onRemove,
}: {
  line: IngredientDraft;
  index: number;
  count: number;
  catalog: Catalog;
  ingredients: Ingredient[];
  busy: boolean;
  review?: boolean;
  onChange: (changes: Partial<IngredientDraft>) => void;
  onMove: (delta: number) => void;
  onAdd: () => void;
  onSelect: (ingredient: Ingredient) => void;
  onBusyChange: (busy: boolean) => void;
  onRemove: () => void;
}) {
  const lastSearchWording = useRef('');
  const [confirmRemove, setConfirmRemove] = useState(false);
  return (
    <fieldset disabled={busy} className="pp-panel space-y-3">
      <legend className="px-2 text-sm font-semibold">
        Ingredient {index + 1}
      </legend>
      {review && (
        <p className="pp-muted whitespace-pre-wrap">
          Original: {line.rawText || 'No wording provided'}
        </p>
      )}
      <div className="pp-editor-grid">
        <IngredientSearch
          selected={ingredients.find((item) => item.id === line.ingredientId)}
          onEdit={(text) => {
            const rawText =
              line.rawText === lastSearchWording.current ? text : line.rawText;
            lastSearchWording.current = text;
            onChange({ ingredientId: null, rawText });
          }}
          onSelect={onSelect}
          onBusyChange={onBusyChange}
        />
        <FormField label="Amount">
          <input
            value={line.amount}
            placeholder="1 ½ or unknown"
            onChange={(event) => onChange({ amount: event.target.value })}
          />
        </FormField>
        <FormField label="Unit">
          <select
            value={line.unitId ?? ''}
            onChange={(event) => {
              const unit = catalog.units.find(
                (u) => u.id === event.target.value,
              );
              onChange({
                unitId: event.target.value || null,
                sourceConvention:
                  unit && unit.system !== 'neutral' ? unit.system : null,
              });
            }}
          >
            <option value="">Unspecified / unknown</option>
            {catalog.units
              .filter(
                (u) => u.dimension !== 'temperature' || u.id === line.unitId,
              )
              .map((u) => (
                <option key={u.id} value={u.id}>
                  {u.label} ({u.system})
                </option>
              ))}
          </select>
        </FormField>
      </div>
      <details>
        <summary className="pp-muted">Find or add ingredient</summary>
        <div className="space-y-2 mt-2">
          <Button variant="quiet" onClick={onAdd}>
            Add ingredient
          </Button>
        </div>
      </details>
      {!line.amount && (
        <span className="pp-badge pp-badge-warning">Unknown amount</span>
      )}
      {!line.ingredientId && (
        <span className="pp-badge pp-badge-warning">
          Choose an ingredient or keep unresolved wording
        </span>
      )}
      <details>
        <summary className="text-sm font-semibold">
          Original wording, preparation and ordering
        </summary>
        <div className="space-y-3 mt-3">
          <FormField label="Original wording">
            <textarea
              value={line.rawText}
              onChange={(event) => onChange({ rawText: event.target.value })}
            />
          </FormField>
          <p className="pp-muted">
            Source convention: {line.sourceConvention ?? 'Unspecified'}.
            Regional units remain distinct.
          </p>
          <FormField label="Preparation note">
            <input
              value={line.note}
              onChange={(event) => onChange({ note: event.target.value })}
            />
          </FormField>
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={line.optional}
              onChange={(event) => onChange({ optional: event.target.checked })}
            />
            Optional ingredient
          </label>
          <div className="flex flex-wrap gap-2">
            <Button disabled={index === 0} onClick={() => onMove(-1)}>
              Move ingredient {index + 1} up
            </Button>
            <Button disabled={index === count - 1} onClick={() => onMove(1)}>
              Move ingredient {index + 1} down
            </Button>
            <Button variant="danger" onClick={() => setConfirmRemove(true)}>
              Remove ingredient {index + 1}
            </Button>
          </div>
          {confirmRemove && (
            <div
              role="group"
              aria-label="Confirm ingredient removal"
              className="space-y-2"
            >
              <p>
                Remove this line from the meal? Saving will remove its wording
                and quantity.
              </p>
              <Button onClick={() => setConfirmRemove(false)}>
                Keep ingredient
              </Button>
              <Button variant="danger" onClick={onRemove}>
                Confirm removal
              </Button>
            </div>
          )}
        </div>
      </details>
    </fieldset>
  );
}
