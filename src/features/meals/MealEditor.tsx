import { useDraftGuard } from '../../lib/drafts';
import { useState, type ReactNode } from 'react';
import { useMutation } from '@tanstack/react-query';
import type {
  Amount,
  Catalog,
  FieldDefinition,
  Meal,
  MealDraft,
  IngredientDraft,
  Ingredient,
} from '../../lib/models';
import { saveMealDraft } from '../../lib/ipc';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { FieldValueControl } from './FieldValueControl';
import { IngredientLineEditor } from './IngredientLineEditor';
import { IngredientDialog } from '../ingredients/IngredientDialog';

const pair = (a: Amount) =>
  a.denominator === 1 ? String(a.numerator) : `${a.numerator}/${a.denominator}`;
function mealDraft(meal?: Meal): MealDraft {
  return meal
    ? {
        title: meal.title,
        description: meal.description,
        servings: meal.baseServingsText ?? pair(meal.baseServings),
        prepMinutes: meal.prepMinutes,
        cookMinutes: meal.cookMinutes,
        instructions: meal.instructions,
        notes: meal.notes,
        sourceUrl: meal.sourceUrl,
        sourceSite: meal.sourceSite,
        tags: [...meal.tags],
        ingredients: meal.ingredients.map((l) => ({
          id: l.id,
          ingredientId: l.ingredientId,
          amount: l.amountText ?? (l.quantity ? pair(l.quantity) : ''),
          unitId: l.unitId,
          rawText: l.rawText,
          sourceConvention: l.sourceConvention,
          note: l.note,
          optional: l.optional,
        })),
        customFields: meal.customFields.map((v) => ({ ...v })),
      }
    : {
        title: '',
        description: '',
        servings: '4',
        prepMinutes: null,
        cookMinutes: null,
        instructions: '',
        notes: '',
        sourceUrl: null,
        sourceSite: null,
        tags: [],
        ingredients: [],
        customFields: [],
      };
}

export function MealEditor({
  meal,
  catalog,
  fields,
  onSaved,
  onCancel,
  initialDraft,
  submitDraft,
  beforeActions,
  submitDisabled = false,
}: {
  meal?: Meal | undefined;
  catalog: Catalog;
  fields: FieldDefinition[];
  onSaved: (meal: Meal) => void;
  onCancel: () => void;
  initialDraft?: MealDraft;
  submitDraft?: (draft: MealDraft) => Promise<Meal>;
  beforeActions?: ReactNode;
  submitDisabled?: boolean;
}) {
  const [draft, setDraft] = useState(() => initialDraft ?? mealDraft(meal));
  const [baseline] = useState(() => JSON.stringify(draft));
  const [tags, setTags] = useState(draft.tags.join('\n'));
  const [discard, setDiscard] = useState(false);
  const [addingIngredient, setAddingIngredient] = useState<string | null>(null);
  const [addedIngredients, setAddedIngredients] = useState<Ingredient[]>([]);
  const [ingredientPending, setIngredientPending] = useState(false);
  const ingredients = [
    ...catalog.ingredients,
    ...addedIngredients.filter(
      (i) => !catalog.ingredients.some((known) => known.id === i.id),
    ),
  ];
  const save = useMutation({
    mutationFn: () =>
      (
        submitDraft ??
        ((input: MealDraft) =>
          saveMealDraft(meal?.id ?? null, meal?.revision ?? null, input))
      )({
        ...draft,
        tags: tags
          .split('\n')
          .map((t) => t.trim())
          .filter(Boolean),
      }),
    onSuccess: onSaved,
  });
  useDraftGuard(
    JSON.stringify(draft) !== baseline ||
      tags !== draft.tags.join('\n') ||
      !!initialDraft,
    save.isPending || ingredientPending,
  );
  const change = <K extends keyof MealDraft>(key: K, value: MealDraft[K]) =>
    setDraft((d) => ({ ...d, [key]: value }));
  const lineChange = (id: string, changes: Partial<IngredientDraft>) =>
    setDraft((d) => ({
      ...d,
      ingredients: d.ingredients.map((l) =>
        l.id === id ? { ...l, ...changes } : l,
      ),
    }));
  const move = (index: number, delta: number) => {
    const lines = [...draft.ingredients];
    const current = lines[index];
    const other = lines[index + delta];
    if (!current || !other) return;
    lines[index] = other;
    lines[index + delta] = current;
    change('ingredients', lines);
  };
  const selectAdded = (ingredient: Ingredient) => {
    setAddedIngredients((items) => [
      ...items.filter((i) => i.id !== ingredient.id),
      ingredient,
    ]);
    if (addingIngredient)
      lineChange(addingIngredient, { ingredientId: ingredient.id });
    setAddingIngredient(null);
  };
  return (
    <>
      {addingIngredient && (
        <IngredientDialog
          catalog={{ ...catalog, ingredients }}
          onClose={() => setAddingIngredient(null)}
          onSaved={selectAdded}
          onSelect={selectAdded}
          selectExisting
        />
      )}
      <form
        className="space-y-5 pp-meal-editor"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="pp-toolbar">
          <h2 className="text-xl font-semibold">
            {meal ? 'Edit meal' : 'New meal'}
          </h2>
          <span className="pp-badge">Draft: save when ready</span>
        </div>
        <nav
          aria-label="Recipe sections"
          className="flex flex-wrap gap-4 pp-muted"
        >
          <a href="#recipe-basics">Basics</a>
          <a href="#recipe-ingredients">Ingredients</a>
          <a href="#recipe-instructions">Instructions</a>
        </nav>
        <fieldset
          disabled={save.isPending || ingredientPending}
          className="space-y-5"
        >
          <section
            id="recipe-basics"
            className="pp-panel pp-form-section space-y-3"
          >
            <h3 className="font-semibold">Basics</h3>
            <div className="grid gap-3 md:grid-cols-2">
              <FormField label="Title">
                <input
                  autoFocus
                  required
                  maxLength={200}
                  value={draft.title}
                  onChange={(e) => change('title', e.target.value)}
                />
              </FormField>
              <FormField label="Base servings">
                <input
                  required
                  value={draft.servings}
                  onChange={(e) => change('servings', e.target.value)}
                />
              </FormField>
              <FormField label="Preparation minutes">
                <input
                  type="number"
                  min="0"
                  step="1"
                  value={draft.prepMinutes ?? ''}
                  onChange={(e) =>
                    change(
                      'prepMinutes',
                      e.target.value === '' ? null : e.target.valueAsNumber,
                    )
                  }
                />
              </FormField>
              <FormField label="Cooking minutes">
                <input
                  type="number"
                  min="0"
                  step="1"
                  value={draft.cookMinutes ?? ''}
                  onChange={(e) =>
                    change(
                      'cookMinutes',
                      e.target.value === '' ? null : e.target.valueAsNumber,
                    )
                  }
                />
              </FormField>
            </div>
            <FormField label="Description">
              <textarea
                value={draft.description}
                onChange={(e) => change('description', e.target.value)}
              />
            </FormField>
          </section>
          <section
            id="recipe-ingredients"
            className="space-y-3 pp-form-section"
            aria-label="Ingredient lines"
          >
            <h3 className="text-lg font-semibold">Ingredients</h3>
            <p className="text-sm text-stone-600">
              Enter fractions such as 1 1/2, or leave amounts unknown / to
              taste. Choose an ingredient explicitly; unresolved wording is
              kept. Regional units are distinct.
            </p>
            {draft.ingredients.map((line, index) => (
              <IngredientLineEditor
                key={line.id}
                line={line}
                index={index}
                count={draft.ingredients.length}
                catalog={catalog}
                ingredients={ingredients}
                busy={save.isPending}
                review={initialDraft !== undefined}
                onChange={(changes) => lineChange(line.id, changes)}
                onMove={(delta) => move(index, delta)}
                onAdd={() => setAddingIngredient(line.id)}
                onBusyChange={setIngredientPending}
                onSelect={(ingredient) => {
                  setAddedIngredients((items) => [
                    ...items.filter((item) => item.id !== ingredient.id),
                    ingredient,
                  ]);
                  lineChange(line.id, { ingredientId: ingredient.id });
                }}
                onRemove={() =>
                  change(
                    'ingredients',
                    draft.ingredients.filter((item) => item.id !== line.id),
                  )
                }
              />
            ))}
            <Button
              onClick={() =>
                change('ingredients', [
                  ...draft.ingredients,
                  {
                    id: crypto.randomUUID(),
                    ingredientId: null,
                    amount: '',
                    unitId: null,
                    rawText: '',
                    sourceConvention: null,
                    note: '',
                    optional: false,
                  },
                ])
              }
            >
              Add ingredient line
            </Button>
          </section>
          <section
            id="recipe-instructions"
            className="pp-panel pp-form-section"
          >
            <FormField label="Instructions">
              <textarea
                rows={6}
                value={draft.instructions}
                onChange={(e) => change('instructions', e.target.value)}
              />
            </FormField>
          </section>
          <details className="pp-panel">
            <summary className="font-semibold">Notes, tags and source</summary>
            <div className="space-y-3 mt-3">
              <FormField label="Notes">
                <textarea
                  value={draft.notes}
                  onChange={(e) => change('notes', e.target.value)}
                />
              </FormField>
              <div className="grid gap-3 md:grid-cols-2">
                <FormField label="Source link (https)">
                  <input
                    value={draft.sourceUrl ?? ''}
                    readOnly={initialDraft !== undefined}
                    onChange={(e) =>
                      change('sourceUrl', e.target.value || null)
                    }
                  />
                </FormField>
                <FormField label="Source site">
                  <input
                    value={draft.sourceSite ?? ''}
                    readOnly={initialDraft !== undefined}
                    onChange={(e) =>
                      change('sourceSite', e.target.value || null)
                    }
                  />
                </FormField>
              </div>
              <FormField label="Tags (one per line)">
                <textarea
                  value={tags}
                  onChange={(e) => setTags(e.target.value)}
                />
              </FormField>
            </div>
          </details>
          <details className="pp-panel">
            <summary className="font-semibold">Household fields</summary>
            <section
              className="mt-3 grid gap-3 md:grid-cols-2"
              aria-label="Custom fields"
            >
              {fields
                .filter(
                  (f) =>
                    !f.retired ||
                    draft.customFields.some((v) => v.fieldId === f.id),
                )
                .map((f) => (
                  <FieldValueControl
                    key={f.id}
                    field={f}
                    value={
                      draft.customFields.find((v) => v.fieldId === f.id)
                        ?.value ?? null
                    }
                    onChange={(value) =>
                      change('customFields', [
                        ...draft.customFields.filter((v) => v.fieldId !== f.id),
                        { fieldId: f.id, value },
                      ])
                    }
                  />
                ))}
            </section>
          </details>
        </fieldset>
        {save.error && (
          <p role="alert">
            {save.error.message} Your draft is still here. If another edit
            changed this meal, cancel and reopen it to load the saved version.
          </p>
        )}
        {beforeActions}
        <div className="pp-save-bar">
          <Button
            type="submit"
            disabled={save.isPending || ingredientPending || submitDisabled}
          >
            Save meal
          </Button>
          <Button
            disabled={save.isPending || ingredientPending}
            onClick={() => setDiscard(true)}
          >
            Cancel editing
          </Button>
        </div>
        {discard && (
          <div role="group" aria-label="Discard meal draft">
            <p>Discard unsaved changes?</p>
            <Button onClick={onCancel}>Discard changes</Button>
            <Button onClick={() => setDiscard(false)}>Keep editing</Button>
          </div>
        )}
      </form>
    </>
  );
}
