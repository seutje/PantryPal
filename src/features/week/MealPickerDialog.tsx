import { useEffect, useRef, useState, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import type { MealLibrary, MealRecord } from '../../lib/models';
import { dateLabel } from '../../lib/local-date';
import { Button } from '../../components/ui/Button';
import { DayChooser } from './PlanForms';
import { FormField } from '../../components/ui/FormField';
import { MealsPage } from '../meals/MealsPage';

export function MealPickerDialog({
  day,
  days,
  library,
  busy,
  saving,
  onClose,
  onSelect,
  children,
}: {
  day: string | null;
  days: string[];
  library: MealLibrary;
  busy: boolean;
  saving: boolean;
  onClose: () => void;
  onSelect: (record: MealRecord, servings: string, day: string | null) => void;
  children?: ReactNode;
}) {
  const [selected, setSelected] = useState<MealRecord | null>(null);
  const [servings, setServings] = useState('');
  const [destination, setDestination] = useState(day);
  const reviewPanel = useRef<HTMLFormElement>(null);
  const dayTitle = day ? dateLabel(day) : 'this week';
  const dialog = useRef<HTMLDialogElement>(null);
  const [opener] = useState(() =>
    document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null,
  );
  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    element.querySelector<HTMLInputElement>('input[type="search"]')?.focus();
    return () => {
      element.close();
      opener?.focus();
    };
  }, [opener]);
  return createPortal(
    <dialog
      ref={dialog}
      aria-label={`Add meal to ${dayTitle}`}
      aria-modal="true"
      className="m-auto max-h-[90vh] w-[min(64rem,95vw)] overflow-y-auto rounded-xl bg-white p-6 shadow-xl backdrop:bg-black/40"
      onCancel={(event) => {
        event.preventDefault();
        if (!saving) onClose();
      }}
    >
      <div className="mb-4 flex items-start justify-between gap-4">
        <div>
          <h2 className="text-xl font-semibold">Add meal to {dayTitle}</h2>
          <p className="text-sm text-stone-600">
            Choose a meal, review its servings and day, then add it to your
            plan.
          </p>
        </div>
        <Button disabled={saving} onClick={onClose}>
          Cancel adding meal
        </Button>
      </div>
      <MealsPage
        initialLibrary={library}
        onSelect={(record) => {
          setSelected(record);
          setServings(record.meal.baseServingsText || record.servingsDisplay);
          requestAnimationFrame(() => {
            reviewPanel.current?.scrollIntoView?.({ block: 'nearest' });
            reviewPanel.current
              ?.querySelector<HTMLInputElement>('input')
              ?.focus();
          });
        }}
        selectionBusy={busy}
      />
      {selected && (
        <form
          ref={reviewPanel}
          className="pp-panel mt-4 space-y-3"
          onSubmit={(event) => {
            event.preventDefault();
            if (!busy) onSelect(selected, servings, destination);
          }}
        >
          <h3 className="text-lg font-semibold">
            Selected: {selected.meal.title}
          </h3>
          <fieldset disabled={busy} className="grid gap-3 sm:grid-cols-2">
            <DayChooser
              days={days}
              value={destination}
              onChange={setDestination}
            />
            <FormField label="Planned servings">
              <input
                required
                value={servings}
                onChange={(event) => setServings(event.target.value)}
              />
            </FormField>
          </fieldset>
          <p className="pp-muted">
            Fractions are allowed. Each addition is a separate occurrence.
          </p>
          <div className="pp-save-bar">
            <Button type="submit" disabled={busy}>
              Add planned meal
            </Button>
            <Button disabled={saving} onClick={onClose}>
              Cancel adding
            </Button>
          </div>
        </form>
      )}
      {saving && <p role="status">Adding meal…</p>}
      {children}
    </dialog>,
    document.body,
  );
}
