import { useEffect, useRef, useState, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import type { MealLibrary, MealRecord } from '../../lib/models';
import { dateLabel } from '../../lib/local-date';
import { Button } from '../../components/ui/Button';
import { MealsPage } from '../meals/MealsPage';

export function MealPickerDialog({
  day,
  library,
  busy,
  saving,
  onClose,
  onSelect,
  children,
}: {
  day: string;
  library: MealLibrary;
  busy: boolean;
  saving: boolean;
  onClose: () => void;
  onSelect: (record: MealRecord) => void;
  children?: ReactNode;
}) {
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
      aria-label={`Add meal to ${dateLabel(day)}`}
      aria-modal="true"
      className="m-auto max-h-[90vh] w-[min(64rem,95vw)] overflow-y-auto rounded-xl bg-white p-6 shadow-xl backdrop:bg-black/40"
      onCancel={(event) => {
        event.preventDefault();
        if (!saving) onClose();
      }}
    >
      <div className="mb-4 flex items-start justify-between gap-4">
        <div>
          <h2 className="text-xl font-semibold">
            Add meal to {dateLabel(day)}
          </h2>
          <p className="text-sm text-stone-600">
            Choose a meal to add its base servings. You can edit servings on the
            day afterwards.
          </p>
        </div>
        <Button disabled={saving} onClick={onClose}>
          Cancel adding meal
        </Button>
      </div>
      <MealsPage
        initialLibrary={library}
        onSelect={onSelect}
        selectionBusy={busy}
      />
      {saving && <p role="status">Adding meal…</p>}
      {children}
    </dialog>,
    document.body,
  );
}
