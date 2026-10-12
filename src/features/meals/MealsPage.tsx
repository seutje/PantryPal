import { useState } from 'react';
import {
  useQuery,
  useMutation,
  useQueryClient,
  keepPreviousData,
} from '@tanstack/react-query';
import { searchMeals, retireMeal } from '../../lib/ipc';
import type {
  Meal,
  MealSearch,
  MealRecord,
  MealLibrary,
} from '../../lib/models';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { MealEditor } from './MealEditor';
import { MealDetail } from './MealDetail';
import { FieldManager } from './FieldManager';
import { FieldValueControl } from './FieldValueControl';
import { ImportPage } from '../import/ImportPage';

const emptySearch: MealSearch = {
  text: '',
  ingredientId: null,
  tag: '',
  field: null,
  cooked: 'any',
  cookedFrom: null,
  cookedTo: null,
};

export function MealsPage({
  initialMeal,
  initialLibrary,
  onSelect,
  selectionBusy = false,
}: {
  initialMeal?: MealRecord | null;
  initialLibrary?: MealLibrary | null;
  onSelect?: (record: MealRecord) => void;
  selectionBusy?: boolean;
}) {
  const client = useQueryClient();
  const [search, setSearch] = useState<MealSearch>(emptySearch);
  const [view, setView] = useState<
    'list' | 'detail' | 'editor' | 'fields' | 'import'
  >(initialMeal ? 'detail' : 'list');
  const [selected, setSelected] = useState<MealRecord | null>(
    initialMeal ?? null,
  );
  const [retiring, setRetiring] = useState(false);
  const library = useQuery({
    queryKey: ['meal-library', search],
    queryFn: () =>
      searchMeals({
        ...search,
        field:
          search.field?.operator === 'equals' && search.field.value === null
            ? null
            : search.field,
      }),
    placeholderData: keepPreviousData,
  });
  const context = useQuery({
    queryKey: ['meal-library', emptySearch],
    queryFn: () => searchMeals(emptySearch),
  });
  const current =
    library.data ?? context.data ?? (onSelect ? initialLibrary : null);
  // Retired planned meals can reference ingredient labels absent from the active library.
  const data =
    current && initialLibrary && initialMeal?.meal.retired
      ? {
          ...current,
          catalog: {
            ...current.catalog,
            ingredients: [
              ...current.catalog.ingredients,
              ...initialLibrary.catalog.ingredients.filter(
                (i) =>
                  !current.catalog.ingredients.some(
                    (active) => active.id === i.id,
                  ),
              ),
            ],
          },
        }
      : current;
  const change = <K extends keyof MealSearch>(key: K, value: MealSearch[K]) =>
    setSearch((s) => ({ ...s, [key]: value }));
  const refresh = () => {
    void client.invalidateQueries({ queryKey: ['meal-library'] });
    void client.invalidateQueries({ queryKey: ['storage-status'] });
  };
  const retire = useMutation({
    mutationFn: (id: string) => retireMeal(id, true),
    onSuccess: () => {
      setRetiring(false);
      setSelected(null);
      setView('list');
      refresh();
    },
  });
  const saved = (meal: Meal) => {
    setSelected(null);
    setSearch({ ...emptySearch, text: meal.title });
    setView('list');
    refresh();
  };
  const field = data?.fields.find((f) => f.id === search.field?.fieldId);
  const results = library.error
    ? []
    : (data?.meals ?? []).filter((record) => !onSelect || !record.meal.retired);
  return (
    <>
      {!onSelect && (
        <div className="mb-4 flex items-center gap-3">
          <Button
            disabled={!data || view === 'import'}
            onClick={() => setView('import')}
          >
            Import recipe
          </Button>
        </div>
      )}
      {library.error && <p role="alert">{library.error.message}</p>}
      {library.isPending && <p role="status">Loading meals…</p>}
      {data && (
        <>
          {view === 'import' ? (
            <ImportPage
              catalog={data.catalog}
              fields={data.fields}
              onSaved={saved}
              onCancel={() => setView('list')}
            />
          ) : view === 'editor' ? (
            <MealEditor
              key={selected?.meal.id ?? 'new'}
              meal={selected?.meal}
              catalog={data.catalog}
              fields={data.fields}
              onSaved={saved}
              onCancel={() => setView(selected ? 'detail' : 'list')}
            />
          ) : view === 'fields' ? (
            <FieldManager
              fields={data.fields}
              onClose={() => setView('list')}
            />
          ) : view === 'detail' && selected ? (
            <>
              <MealDetail
                record={
                  data.meals.find(
                    (record) => record.meal.id === selected.meal.id,
                  ) ?? selected
                }
                library={data}
                onEdit={() => setView('editor')}
                onBack={() => {
                  setSelected(null);
                  setView('list');
                }}
                onRetire={() => setRetiring(true)}
              />
              {retiring && (
                <div role="group" aria-label="Confirm meal retirement">
                  <p>
                    Retire {selected.meal.title}? Its recipe and references will
                    be preserved.
                  </p>
                  <Button
                    disabled={retire.isPending}
                    onClick={() => retire.mutate(selected.meal.id)}
                  >
                    Confirm meal retirement
                  </Button>
                  <Button onClick={() => setRetiring(false)}>Keep meal</Button>
                </div>
              )}
              {retire.error && <p role="alert">{retire.error.message}</p>}
            </>
          ) : (
            <section className="space-y-4">
              {!onSelect && (
                <div className="flex gap-2">
                  <Button
                    onClick={() => {
                      setSelected(null);
                      setView('editor');
                    }}
                  >
                    New meal
                  </Button>
                  <Button onClick={() => setView('fields')}>
                    Manage custom fields
                  </Button>
                </div>
              )}
              <fieldset disabled={selectionBusy} className="space-y-4">
                <FormField label="Search meals">
                  <input
                    type="search"
                    value={search.text}
                    onChange={(e) => change('text', e.target.value)}
                    placeholder="Title, tags, ingredient names or field values"
                  />
                </FormField>
                <details className="rounded-lg border border-stone-300 p-3">
                  <summary className="cursor-pointer font-semibold">
                    Filters
                  </summary>
                  <div className="mt-3 grid gap-3 md:grid-cols-2">
                    <FormField label="Contains ingredient">
                      <select
                        value={search.ingredientId ?? ''}
                        onChange={(e) =>
                          change('ingredientId', e.target.value || null)
                        }
                      >
                        <option value="">Any ingredient</option>
                        {data.catalog.ingredients.map((i) => (
                          <option key={i.id} value={i.id}>
                            {i.name}
                            {i.retired ? ' (retired)' : ''}
                          </option>
                        ))}
                      </select>
                    </FormField>
                    <FormField label="Tag">
                      <input
                        value={search.tag}
                        onChange={(e) => change('tag', e.target.value)}
                      />
                    </FormField>
                    <FormField label="Custom field filter">
                      <select
                        value={search.field?.fieldId ?? ''}
                        onChange={(e) =>
                          change(
                            'field',
                            e.target.value
                              ? {
                                  fieldId: e.target.value,
                                  operator: 'known',
                                  value: null,
                                }
                              : null,
                          )
                        }
                      >
                        <option value="">Any field</option>
                        {data.fields.map((f) => (
                          <option key={f.id} value={f.id}>
                            {f.label}
                            {f.retired ? ' (retired)' : ''}
                          </option>
                        ))}
                      </select>
                    </FormField>
                    {field && search.field && (
                      <>
                        <FormField label="Field condition">
                          <select
                            value={search.field.operator}
                            onChange={(e) =>
                              change('field', {
                                fieldId: field.id,
                                operator: e.target.value as
                                  'equals' | 'known' | 'unknown',
                                value: null,
                              })
                            }
                          >
                            <option value="known">Known</option>
                            <option value="unknown">Unknown</option>
                            <option value="equals">Equals</option>
                          </select>
                        </FormField>
                        {search.field.operator === 'equals' && (
                          <FieldValueControl
                            field={{ ...field, retired: false }}
                            value={search.field.value}
                            onChange={(value) =>
                              change('field', {
                                fieldId: field.id,
                                operator: 'equals',
                                value,
                              })
                            }
                          />
                        )}
                      </>
                    )}
                    <FormField label="Cooked history">
                      <select
                        value={search.cooked}
                        onChange={(e) =>
                          change(
                            'cooked',
                            e.target.value as MealSearch['cooked'],
                          )
                        }
                      >
                        <option value="any">Any history</option>
                        <option value="never">Never recorded</option>
                        <option value="cooked">Cooked at least once</option>
                      </select>
                    </FormField>
                    <FormField label="Last cooked from">
                      <input
                        type="date"
                        value={search.cookedFrom ?? ''}
                        onChange={(e) =>
                          change('cookedFrom', e.target.value || null)
                        }
                      />
                    </FormField>
                    <FormField label="Last cooked through">
                      <input
                        type="date"
                        value={search.cookedTo ?? ''}
                        onChange={(e) =>
                          change('cookedTo', e.target.value || null)
                        }
                      />
                    </FormField>
                  </div>
                </details>
                <Button onClick={() => setSearch(emptySearch)}>
                  Clear filters
                </Button>
                <p role="status">
                  {results.length} meal{results.length === 1 ? '' : 's'} found
                </p>
                {results.length === 0 && (
                  <p>No meals match. Create a meal or clear your filters.</p>
                )}
                <ul className="grid gap-3 md:grid-cols-2">
                  {results.map((record) => (
                    <li
                      key={record.meal.id}
                      className="rounded-lg border border-stone-300 p-4"
                    >
                      {onSelect ? (
                        <button
                          type="button"
                          className="w-full rounded-lg p-2 text-left hover:bg-green-50 disabled:cursor-not-allowed disabled:opacity-50"
                          aria-label={`Add ${record.meal.title}`}
                          disabled={
                            selectionBusy ||
                            library.isFetching ||
                            library.isPlaceholderData
                          }
                          onClick={() => onSelect(record)}
                        >
                          <span className="block text-lg font-semibold">
                            {record.meal.title}
                          </span>
                          <span className="block">
                            {record.servingsDisplay} servings · Last cooked:{' '}
                            {record.lastCooked ?? 'Never recorded'}
                          </span>
                          <span className="block">
                            {record.meal.tags.join(', ')}
                          </span>
                        </button>
                      ) : (
                        <>
                          <h2 className="text-lg font-semibold">
                            {record.meal.title}
                          </h2>
                          <p>
                            {record.servingsDisplay} servings · Last cooked:{' '}
                            {record.lastCooked ?? 'Never recorded'}
                          </p>
                          <p>{record.meal.tags.join(', ')}</p>
                          <Button
                            className="mt-2"
                            onClick={() => {
                              setSelected(record);
                              setView('detail');
                            }}
                          >
                            View {record.meal.title}
                          </Button>
                        </>
                      )}
                    </li>
                  ))}
                </ul>
              </fieldset>
            </section>
          )}
        </>
      )}
    </>
  );
}
