import { useDraftGuard } from '../../lib/drafts';
import { useEffect, useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import {
  createIngredient,
  updateIngredient,
  retireIngredient,
  ingredientCandidates,
} from '../../lib/ipc';
import type { Catalog, Ingredient, IngredientInput } from '../../lib/models';
import { ConversionControl } from './ConversionControl';
import { ConversionRules } from './ConversionRules';

const blank: IngredientInput = {
  name: '',
  categoryId: null,
  sectionId: null,
  preferredUnitId: null,
  aliases: [],
  seasonMonths: [],
  notes: '',
};
const months = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
];
export function IngredientEditor({
  ingredient,
  catalog,
  onClose,
  onSelect,
  onSaved,
  onBusyChange,
  selectExisting = false,
}: {
  ingredient: Ingredient | null;
  catalog: Catalog;
  onClose: () => void;
  onSelect: (ingredient: Ingredient) => void;
  onSaved?: (ingredient: Ingredient) => void;
  onBusyChange?: (busy: boolean) => void;
  selectExisting?: boolean;
}) {
  const [input, setInput] = useState<IngredientInput>(ingredient ?? blank);
  const [baseline] = useState(() => JSON.stringify(input));
  const [aliases, setAliases] = useState(ingredient?.aliases.join('\n') ?? '');
  const [candidates, setCandidates] = useState<Ingredient[]>([]);
  const [confirm, setConfirm] = useState(false);
  const client = useQueryClient();
  const refresh = async () => {
    await Promise.all([
      client.invalidateQueries({ queryKey: ['catalog'] }),
      client.invalidateQueries({ queryKey: ['storage-status'] }),
      client.invalidateQueries({ queryKey: ['meal-library'] }),
    ]);
  };
  const save = useMutation({
    mutationFn: async () => {
      const data: IngredientInput = {
        name: input.name,
        categoryId: input.categoryId,
        sectionId: input.sectionId,
        preferredUnitId: input.preferredUnitId,
        seasonMonths: input.seasonMonths,
        notes: input.notes,
        aliases: aliases
          .split('\n')
          .map((s) => s.trim())
          .filter(Boolean),
      };
      const matches = await ingredientCandidates(
        [data.name, ...data.aliases],
        ingredient?.id ?? null,
      );
      if (matches.length) return { matches };
      const saved = ingredient
        ? await updateIngredient(ingredient.id, data)
        : await createIngredient(data);
      return { saved };
    },
    onSuccess: async (result) => {
      if (result.matches) {
        setCandidates(result.matches);
      } else {
        await refresh();
        if (onSaved) onSaved(result.saved);
        else onClose();
      }
    },
  });
  const retire = useMutation({
    mutationFn: () => retireIngredient(ingredient!.id, true),
    onSuccess: async () => {
      await refresh();
      onClose();
    },
  });
  const change = <K extends keyof IngredientInput>(
    key: K,
    value: IngredientInput[K],
  ) => {
    setInput({ ...input, [key]: value });
    setCandidates([]);
    save.reset();
  };
  const pending = save.isPending || retire.isPending;
  const canDiscard = useDraftGuard(
    JSON.stringify(input) !== baseline ||
      aliases !== (ingredient?.aliases.join('\n') ?? ''),
    pending,
  );
  useEffect(() => {
    onBusyChange?.(pending);
  }, [pending, onBusyChange]);
  return (
    <section
      aria-label={ingredient ? 'Edit ingredient' : 'New ingredient'}
      className="pp-panel"
    >
      <h2 className="mb-4 text-xl font-semibold">
        {ingredient ? `Edit ${ingredient.name}` : 'New ingredient'}
      </h2>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          e.stopPropagation();
          setCandidates([]);
          save.mutate();
        }}
        className="space-y-4"
      >
        <fieldset disabled={pending} className="space-y-4">
          <FormField label="Ingredient name">
            <input
              autoFocus
              required
              maxLength={120}
              value={input.name}
              onChange={(e) => change('name', e.target.value)}
            />
          </FormField>
          <div className="grid gap-4">
            {(['categoryId', 'sectionId'] as const).map((key) => (
              <FormField
                key={key}
                label={key === 'categoryId' ? 'Category' : 'Shopping section'}
              >
                <select
                  value={input[key] ?? ''}
                  onChange={(e) => change(key, e.target.value || null)}
                >
                  <option value="">Unspecified</option>
                  {(key === 'categoryId'
                    ? catalog.categories
                    : catalog.sections
                  )
                    .filter((v) => !v.retired || v.id === input[key])
                    .map((v) => (
                      <option key={v.id} value={v.id}>
                        {v.name}
                        {v.retired ? ' (retired)' : ''}
                      </option>
                    ))}
                </select>
              </FormField>
            ))}
            <FormField label="Preferred shopping unit">
              <select
                value={input.preferredUnitId ?? ''}
                onChange={(e) =>
                  change('preferredUnitId', e.target.value || null)
                }
              >
                <option value="">Use measurement preference</option>
                {catalog.units
                  .filter(
                    (u) =>
                      u.dimension !== 'temperature' ||
                      u.id === ingredient?.preferredUnitId,
                  )
                  .map((u) => (
                    <option key={u.id} value={u.id}>
                      {u.label}
                    </option>
                  ))}
              </select>
            </FormField>
          </div>
          <FormField label="Aliases (one per line)">
            <textarea
              rows={3}
              value={aliases}
              onChange={(e) => {
                setAliases(e.target.value);
                setCandidates([]);
                save.reset();
              }}
            />
          </FormField>
          <details>
            <summary className="font-semibold">Seasonality and notes</summary>
            <div className="space-y-3 mt-3">
              <fieldset>
                <legend className="mb-2 text-sm font-medium">
                  Season months
                </legend>
                <p className="mb-2 text-sm text-stone-600">
                  Leave empty for unspecified availability. These are household
                  notes.
                </p>
                <label className="mb-3 flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={input.seasonMonths.length === 12}
                    onChange={(e) =>
                      change(
                        'seasonMonths',
                        e.target.checked ? months.map((_, i) => i + 1) : [],
                      )
                    }
                  />
                  Year-round
                </label>
                <div className="grid grid-cols-3 gap-2 sm:grid-cols-4">
                  {months.map((month, i) => (
                    <label
                      key={month}
                      className="flex items-center gap-2 text-sm"
                    >
                      <input
                        type="checkbox"
                        checked={input.seasonMonths.includes(i + 1)}
                        onChange={(e) =>
                          change(
                            'seasonMonths',
                            e.target.checked
                              ? [...input.seasonMonths, i + 1]
                              : input.seasonMonths.filter((m) => m !== i + 1),
                          )
                        }
                      />
                      {month}
                    </label>
                  ))}
                </div>
              </fieldset>
              <FormField label="Ingredient notes">
                <textarea
                  rows={3}
                  maxLength={10000}
                  value={input.notes}
                  onChange={(e) => change('notes', e.target.value)}
                />
              </FormField>
            </div>
          </details>
        </fieldset>
        {candidates.length > 0 && (
          <div role="alert" className="rounded-lg bg-amber-50 p-3">
            <p>
              A name or alias already belongs to an ingredient. Choose another
              name, or choose the existing ingredient.
            </p>
            {candidates.map((match) => (
              <div key={match.id} className="mt-2">
                {match.retired ? (
                  <p>
                    {match.name} (retired; its names remain reserved for
                    historical records)
                  </p>
                ) : (
                  <Button onClick={() => onSelect(match)}>
                    {selectExisting ? 'Use existing' : 'Open existing'}{' '}
                    {match.name}
                  </Button>
                )}
              </div>
            ))}
          </div>
        )}
        {save.isError && <p role="alert">{save.error.message}</p>}
        <div className="pp-save-bar">
          <Button type="submit" disabled={pending || candidates.length > 0}>
            Save ingredient
          </Button>
          <Button
            data-cancel-editor
            disabled={pending}
            onClick={() => {
              if (canDiscard()) onClose();
            }}
          >
            Cancel
          </Button>
        </div>
      </form>
      {ingredient && (
        <>
          <ConversionRules ingredientId={ingredient.id} units={catalog.units} />
          <details className="mt-4">
            <summary className="font-semibold">Test conversion</summary>
            <ConversionControl
              ingredientId={ingredient.id}
              units={catalog.units}
            />
          </details>
          <details className="mt-5">
            <summary className="cursor-pointer font-medium">
              Retire ingredient
            </summary>
            <p className="my-3 text-sm">
              Hide this ingredient from the catalog. Existing recipes and its
              reserved names remain intact.
            </p>
            <label className="mb-3 flex items-center gap-2">
              <input
                type="checkbox"
                checked={confirm}
                onChange={(e) => setConfirm(e.target.checked)}
              />
              Confirm retirement of {ingredient.name}
            </label>
            <Button
              variant="danger"
              disabled={!confirm || pending}
              onClick={() => retire.mutate()}
            >
              Retire {ingredient.name}
            </Button>
            {retire.isError && <p role="alert">{retire.error.message}</p>}
          </details>
        </>
      )}
    </section>
  );
}
