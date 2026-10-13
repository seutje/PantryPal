import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import type { Catalog, Ingredient } from '../../lib/models';
import { IngredientEditor } from './IngredientEditor';

export function IngredientDialog({
  catalog,
  onClose,
  onSaved,
  onSelect,
  selectExisting = false,
}: {
  catalog: Catalog;
  onClose: () => void;
  onSaved?: (ingredient: Ingredient) => void;
  onSelect: (ingredient: Ingredient) => void;
  selectExisting?: boolean;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [opener] = useState(() =>
    document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null,
  );
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    const element = dialog.current!;
    // Native modal dialogs supply inert background, keyboard focus containment
    // and top-layer rendering. The portal keeps its form outside the meal form.
    element.showModal();
    return () => {
      element.close();
      opener?.focus();
    };
  }, [opener]);
  return createPortal(
    <dialog
      ref={dialog}
      aria-label="Add ingredient"
      aria-modal="true"
      className="m-auto max-h-[90vh] w-[min(48rem,95vw)] overflow-y-auto rounded-xl bg-white p-0 shadow-xl backdrop:bg-black/40"
      onCancel={(event) => {
        event.preventDefault();
        if (!busy)
          dialog.current
            ?.querySelector<HTMLButtonElement>('[data-cancel-editor]')
            ?.click();
      }}
    >
      {selectExisting && (
        <p className="px-6 pt-6 text-sm text-stone-600">
          Saving adds the ingredient to your catalog and selects it in this
          recipe. Your recipe is saved separately after review.
        </p>
      )}
      <IngredientEditor
        ingredient={null}
        catalog={catalog}
        onClose={onClose}
        onSelect={onSelect}
        {...(onSaved ? { onSaved } : {})}
        selectExisting={selectExisting}
        onBusyChange={setBusy}
      />
    </dialog>,
    document.body,
  );
}
