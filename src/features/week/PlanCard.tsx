import { useState } from 'react';
import type {
  FieldDefinition,
  MealRecord,
  PlanEntry,
  PlanOperation,
} from '../../lib/models';
import { Button } from '../../components/ui/Button';
import { EditPlanForm, RecordCookedForm } from './PlanForms';
import { fieldText } from '../meals/field-text';

export function PlanCard({
  entry,
  meal,
  fields,
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
  days: string[];
  busy: boolean;
  onApply: (operation: PlanOperation) => void;
  onOpen: () => void;
  onMove: (delta: number) => void;
  first: boolean;
  last: boolean;
}) {
  const [mode, setMode] = useState<
    'view' | 'edit' | 'cook' | 'remove' | 'undo'
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
    <li className="rounded-lg border border-stone-300 bg-white p-4">
      <fieldset disabled={busy} className="space-y-3">
        <legend className="sr-only">
          {meal.meal.title} planned occurrence
        </legend>
        <h3 className="text-lg font-semibold">
          {meal.meal.title}
          {meal.meal.retired ? ' (retired)' : ''}
        </h3>
        <p>
          <span className="rounded bg-stone-100 px-2 py-1 font-medium">
            {entry.status}
          </span>{' '}
          · {entry.servingsText} servings
        </p>
        <p>
          Preparation: {meal.meal.prepMinutes ?? 'unknown'} min · Cooking:{' '}
          {meal.meal.cookMinutes ?? 'unknown'} min
        </p>
        <p>Last eaten: {meal.lastCooked ?? 'Never recorded'}</p>
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
              <p key={v.field.id}>
                {v.field.label}: {fieldText(v.field, v.value)}
              </p>
            ),
        )}
        {mode === 'edit' ? (
          <EditPlanForm
            entry={entry}
            days={days}
            onApply={onApply}
            onCancel={() => setMode('view')}
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
              kept and it will return to planned.
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
        ) : (
          <>
            <div className="flex flex-wrap gap-2">
              <Button onClick={onOpen}>Open recipe</Button>
              <Button onClick={() => setMode('edit')}>Edit occurrence</Button>
              <Button onClick={() => setMode('remove')}>
                Remove occurrence
              </Button>
            </div>
            <div className="flex flex-wrap gap-2">
              {entry.cookedEvent && !entry.cookedEvent.voided ? (
                <Button onClick={() => setMode('undo')}>Undo cooked</Button>
              ) : (
                <Button onClick={() => setMode('cook')}>Mark cooked</Button>
              )}
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
            </div>
            <div className="flex flex-wrap gap-2">
              <Button disabled={first} onClick={() => onMove(-1)}>
                Move up
              </Button>
              <Button disabled={last} onClick={() => onMove(1)}>
                Move down
              </Button>
            </div>
          </>
        )}
      </fieldset>
    </li>
  );
}
