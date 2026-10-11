import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import type {
  Amount,
  Catalog,
  FieldDefinition,
  Meal,
  MealDraft,
  IngredientDraft,
} from '../../lib/models';
import { saveMealDraft } from '../../lib/ipc';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { FieldValueControl } from './FieldValueControl';

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
}: {
  meal?: Meal | undefined;
  catalog: Catalog;
  fields: FieldDefinition[];
  onSaved: (meal: Meal) => void;
  onCancel: () => void;
}) {
  const [draft, setDraft] = useState(() => mealDraft(meal));
  const [tags, setTags] = useState(draft.tags.join('\n'));
  const [remove, setRemove] = useState<string | null>(null);
  const [discard, setDiscard] = useState(false);
  const save = useMutation({
    mutationFn: () =>
      saveMealDraft(meal?.id ?? null, meal?.revision ?? null, {
        ...draft,
        tags: tags
          .split('\n')
          .map((t) => t.trim())
          .filter(Boolean),
      }),
    onSuccess: onSaved,
  });
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
  return (
    <form
      className="space-y-5"
      onSubmit={(e) => {
        e.preventDefault();
        save.mutate();
      }}
    >
      <h2 className="text-xl font-semibold">
        {meal ? 'Edit meal' : 'New meal'}
      </h2>
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
      <section className="space-y-3" aria-label="Ingredient lines">
        <h3 className="text-lg font-semibold">Ingredients</h3>
        <p className="text-sm text-stone-600">
          Enter fractions such as 1 1/2, or leave amounts unknown / to taste.
          Choose an ingredient explicitly; unresolved wording is kept. Regional
          units are distinct.
        </p>
        {draft.ingredients.map((line, index) => (
          <fieldset
            key={line.id}
            className="space-y-3 rounded-lg border border-stone-300 p-4"
          >
            <legend className="px-1 font-semibold">
              Ingredient {index + 1}
            </legend>
            <div className="grid gap-3 md:grid-cols-3">
              <FormField label="Amount">
                <input
                  value={line.amount}
                  onChange={(e) =>
                    lineChange(line.id, { amount: e.target.value })
                  }
                />
              </FormField>
              <FormField label="Unit">
                <select
                  value={line.unitId ?? ''}
                  onChange={(e) => {
                    const unit = catalog.units.find(
                      (u) => u.id === e.target.value,
                    );
                    lineChange(line.id, {
                      unitId: e.target.value || null,
                      sourceConvention:
                        unit && unit.system !== 'neutral' ? unit.system : null,
                    });
                  }}
                >
                  <option value="">Unspecified / unknown</option>
                  {catalog.units
                    .filter(
                      (u) =>
                        u.dimension !== 'temperature' || u.id === line.unitId,
                    )
                    .map((u) => (
                      <option key={u.id} value={u.id}>
                        {u.label} ({u.system})
                      </option>
                    ))}
                </select>
              </FormField>
              <FormField label="Ingredient">
                <select
                  value={line.ingredientId ?? ''}
                  onChange={(e) =>
                    lineChange(line.id, {
                      ingredientId: e.target.value || null,
                    })
                  }
                >
                  <option value="">Unresolved — keep wording</option>
                  {catalog.ingredients
                    .filter((i) => !i.retired || i.id === line.ingredientId)
                    .map((i) => (
                      <option key={i.id} value={i.id}>
                        {i.name}
                        {i.retired ? ' (retired)' : ''}
                      </option>
                    ))}
                </select>
              </FormField>
            </div>
            <FormField label="Original wording">
              <textarea
                value={line.rawText}
                onChange={(e) =>
                  lineChange(line.id, { rawText: e.target.value })
                }
              />
            </FormField>
            <FormField label="Preparation note">
              <input
                value={line.note}
                onChange={(e) => lineChange(line.id, { note: e.target.value })}
              />
            </FormField>
            <label className="flex gap-2">
              <input
                type="checkbox"
                checked={line.optional}
                onChange={(e) =>
                  lineChange(line.id, { optional: e.target.checked })
                }
              />
              Optional ingredient
            </label>
            <div className="flex flex-wrap gap-2">
              <Button disabled={index === 0} onClick={() => move(index, -1)}>
                Move ingredient {index + 1} up
              </Button>
              <Button
                disabled={index === draft.ingredients.length - 1}
                onClick={() => move(index, 1)}
              >
                Move ingredient {index + 1} down
              </Button>
              <Button onClick={() => setRemove(line.id)}>
                Remove ingredient {index + 1}
              </Button>
            </div>
            {remove === line.id && (
              <div
                className="space-x-2"
                role="group"
                aria-label="Confirm ingredient removal"
              >
                <p>
                  Remove this line from the meal? Saving will remove its wording
                  and quantity.
                </p>
                <Button
                  onClick={() => {
                    change(
                      'ingredients',
                      draft.ingredients.filter((l) => l.id !== line.id),
                    );
                    setRemove(null);
                  }}
                >
                  Confirm removal
                </Button>
                <Button onClick={() => setRemove(null)}>Keep ingredient</Button>
              </div>
            )}
          </fieldset>
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
      <FormField label="Instructions">
        <textarea
          rows={6}
          value={draft.instructions}
          onChange={(e) => change('instructions', e.target.value)}
        />
      </FormField>
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
            onChange={(e) => change('sourceUrl', e.target.value || null)}
          />
        </FormField>
        <FormField label="Source site">
          <input
            value={draft.sourceSite ?? ''}
            onChange={(e) => change('sourceSite', e.target.value || null)}
          />
        </FormField>
      </div>
      <FormField label="Tags (one per line)">
        <textarea value={tags} onChange={(e) => setTags(e.target.value)} />
      </FormField>
      <section className="grid gap-3 md:grid-cols-2" aria-label="Custom fields">
        {fields
          .filter(
            (f) =>
              !f.retired || draft.customFields.some((v) => v.fieldId === f.id),
          )
          .map((f) => (
            <FieldValueControl
              key={f.id}
              field={f}
              value={
                draft.customFields.find((v) => v.fieldId === f.id)?.value ??
                null
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
      {save.error && (
        <p role="alert">
          {save.error.message} Your draft is still here. If another edit changed
          this meal, cancel and reopen it to load the saved version.
        </p>
      )}
      <div className="flex gap-2">
        <Button type="submit" disabled={save.isPending}>
          Save meal
        </Button>
        <Button disabled={save.isPending} onClick={() => setDiscard(true)}>
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
  );
}
