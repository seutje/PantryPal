import { useState } from 'react';
import type {
  FieldDefinition,
  Catalog,
  MealRecord,
  PlanEntry,
  PlanOperation,
} from '../../lib/models';
import { Actions } from '../../components/ui/Actions';
import { Button } from '../../components/ui/Button';
import { EditPlanForm, RecordCookedForm } from './PlanForms';
import { fieldText } from '../meals/field-text';

export function PlanCard({
  entry,
  meal,
  fields,
  catalog,
  days,
  busy,
  onApply,
  onOpen,
  onMove,
  first,
  last,
}: {
  entry: PlanEntry;
  meal: MealRecord;
  fields: FieldDefinition[];
  catalog: Catalog;
  days: string[];
  busy: boolean;
  onApply: (operation: PlanOperation) => void;
  onOpen: () => void;
  onMove: (delta: number) => void;
  first: boolean;
  last: boolean;
}) {
  const [mode, setMode] = useState<
    'view' | 'edit' | 'servings' | 'cook' | 'remove' | 'undo'
  >('view');
  const notable = meal.meal.customFields
    .filter((v) => v.value !== null)
    .map((v) => ({
      field: fields.find((f) => f.id === v.fieldId),
      value: v.value,
    }))
    .filter((v) => v.field && !v.field.retired)
    .slice(0, 3);
  return (
    <li className="pp-plan-card" data-plan-id={entry.id}>
      <fieldset disabled={busy} className="space-y-2">
        <legend className="sr-only">
          {meal.meal.title} planned occurrence
        </legend>
        <div className="pp-plan-header">
          <div>
            <h3 className="text-lg font-semibold">
              <Button
                variant="quiet"
                className="pp-plan-title"
                aria-label="Open recipe"
                onClick={onOpen}
              >
                {meal.meal.title}
              </Button>
              {meal.meal.retired ? ' (retired)' : ''}
            </h3>
            <p>
              <span className="pp-badge">{entry.status}</span> ·{' '}
              {entry.servingsText} servings
            </p>
          </div>
          {mode === 'view' && (
            <div className="pp-plan-actions">
              <div className="flex flex-wrap items-center gap-2">
                {entry.cookedEvent && !entry.cookedEvent.voided ? (
                  <Button onClick={() => setMode('undo')}>Undo cooked</Button>
                ) : (
                  <Button onClick={() => setMode('cook')}>Mark cooked</Button>
                )}
                <Actions label="More occurrence actions">
                  <Button onClick={() => setMode('servings')}>
                    Edit servings
                  </Button>
                  <Button onClick={() => setMode('edit')}>
                    Edit occurrence
                  </Button>
                  <Button variant="danger" onClick={() => setMode('remove')}>
                    Remove occurrence
                  </Button>
                  {entry.status === 'skipped' ? (
                    <Button
                      onClick={() =>
                        onApply({
                          kind: 'set_status',
                          entryId: entry.id,
                          status: 'planned',
                          cookedDate: null,
                          confirmed: false,
                        })
                      }
                    >
                      Return to planned
                    </Button>
                  ) : (
                    entry.status === 'planned' && (
                      <Button
                        onClick={() =>
                          onApply({
                            kind: 'set_status',
                            entryId: entry.id,
                            status: 'skipped',
                            cookedDate: null,
                            confirmed: false,
                          })
                        }
                      >
                        Skip occurrence
                      </Button>
                    )
                  )}
                  <Button disabled={first} onClick={() => onMove(-1)}>
                    Move up
                  </Button>
                  <Button disabled={last} onClick={() => onMove(1)}>
                    Move down
                  </Button>
                </Actions>
              </div>
            </div>
          )}
        </div>
        <div className="pp-plan-meta">
          <p className="pp-muted">
            Preparation: {meal.meal.prepMinutes ?? 'unknown'} min · Cooking:{' '}
            {meal.meal.cookMinutes ?? 'unknown'} min
          </p>
          <p className="pp-muted">
            Last eaten: {meal.lastCooked ?? 'Never recorded'}
          </p>
        </div>
        {entry.cookedEvent && !entry.cookedEvent.voided && (
          <p>This occurrence cooked: {entry.cookedEvent.localDate}</p>
        )}
        {entry.status === 'cooked' &&
          (!entry.cookedEvent || entry.cookedEvent.voided) && (
            <p className="text-amber-800">
              History not recorded for this legacy status. Record cooking
              explicitly to update last eaten.
            </p>
          )}
        {notable.map(
          (v) =>
            v.field && (
              <span className="pp-badge" key={v.field.id}>
                {v.field.label}: {fieldText(v.field, v.value)}
              </span>
            ),
        )}

        {mode === 'edit' || mode === 'servings' ? (
          <EditPlanForm
            entry={entry}
            days={days}
            onApply={onApply}
            onCancel={() => setMode('view')}
            servingsOnly={mode === 'servings'}
          />
        ) : mode === 'cook' ? (
          <RecordCookedForm
            entry={entry}
            onApply={onApply}
            onCancel={() => setMode('view')}
          />
        ) : mode === 'remove' ? (
          <div role="group" aria-label="Confirm plan removal">
            <p>
              Remove this planned occurrence? Its recipe and cooking history
              will be kept.
            </p>
            <Button
              variant="danger"
              onClick={() =>
                onApply({ kind: 'remove', entryId: entry.id, confirmed: true })
              }
            >
              Confirm plan removal
            </Button>
            <Button onClick={() => setMode('view')}>Keep planned meal</Button>
          </div>
        ) : mode === 'undo' ? (
          <div role="group" aria-label="Confirm cooking undo">
            <p>
              Undo cooking for this occurrence? Its cancelled record will be
              kept and it will return to planned. It will no longer count toward
              last eaten.
            </p>
            <Button
              onClick={() =>
                onApply({
                  kind: 'set_status',
                  entryId: entry.id,
                  status: 'planned',
                  cookedDate: null,
                  confirmed: true,
                })
              }
            >
              Confirm cooking undo
            </Button>
            <Button onClick={() => setMode('view')}>Keep cooked</Button>
          </div>
        ) : null}
        <details className="pp-muted">
          <summary className="cursor-pointer font-medium">
            Ingredients for {entry.servingsText} servings
          </summary>
          <p className="mt-2 text-sm text-stone-600">
            Recipe base: {meal.servingsDisplay} servings. Shopping uses these
            planned amounts after refresh.
          </p>
          <ul className="mt-2 space-y-2">
            {meal.meal.ingredients.map((line, index) => (
              <li key={line.id}>
                {entry.quantityDisplays[index] ??
                  (line.amountText || 'Unknown amount')}{' '}
                {catalog.units.find((u) => u.id === line.unitId)?.symbol ?? ''}{' '}
                —{' '}
                {catalog.ingredients.find((i) => i.id === line.ingredientId)
                  ?.name ?? 'Unresolved ingredient'}
                {line.optional ? ' (optional)' : ''}
                {line.note ? ` · ${line.note}` : ''}
              </li>
            ))}
          </ul>
        </details>
      </fieldset>
    </li>
  );
}
