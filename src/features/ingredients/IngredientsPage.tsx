import { useRef, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { EmptyState } from '../../components/EmptyState';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { getCatalog, nativeAvailable } from '../../lib/ipc';
import type { Ingredient } from '../../lib/models';
import { IngredientEditor } from './IngredientEditor';
import { VocabularyManager } from './VocabularyManager';
import { ConversionControl } from './ConversionControl';

export function IngredientsPage() {
  const catalog = useQuery({
    queryKey: ['catalog'],
    queryFn: getCatalog,
    enabled: nativeAvailable(),
  });
  const [search, setSearch] = useState('');
  const [editing, setEditing] = useState<Ingredient | null | undefined>(
    undefined,
  );
  const addButton = useRef<HTMLButtonElement>(null);
  const close = () => {
    setEditing(undefined);
    addButton.current?.focus();
  };
  if (!nativeAvailable())
    return (
      <EmptyState title="Ingredient catalog">
        Open PantryPal on your desktop to view your local ingredients.
      </EmptyState>
    );
  if (catalog.isPending)
    return <p role="status">Loading your ingredient catalog…</p>;
  if (catalog.isError)
    return (
      <p role="alert">
        Your ingredient catalog could not be opened. Your data has not been
        reset. Restart PantryPal to retry.
      </p>
    );
  const data = catalog.data;
  const visible = data.ingredients.filter((i) =>
    [i.name, ...i.aliases].some((s) =>
      s.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase()),
    ),
  );
  return (
    <>
      {editing !== undefined && (
        <IngredientEditor
          key={editing?.id ?? 'new'}
          ingredient={editing}
          catalog={data}
          onClose={close}
          onSelect={setEditing}
        />
      )}
      <section
        aria-label="Ingredient catalog"
        className="rounded-xl border border-stone-200 bg-white p-6"
      >
        <div className="mb-4 flex flex-wrap items-end justify-between gap-3">
          <FormField label="Search ingredients or aliases">
            <input
              type="search"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </FormField>
          <Button ref={addButton} onClick={() => setEditing(null)}>
            Add ingredient
          </Button>
        </div>
        {visible.length === 0 ? (
          <p>No ingredients match. Add an ingredient or change your search.</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <caption className="sr-only">
                Saved ingredients and their preferred shopping units
              </caption>
              <thead className="border-b border-stone-200">
                <tr>
                  {[
                    'Ingredient',
                    'Category',
                    'Shopping section',
                    'Preferred unit',
                    'Actions',
                  ].map((label) => (
                    <th key={label} scope="col" className="p-3">
                      {label}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {visible.map((i) => (
                  <tr key={i.id} className="border-b border-stone-100">
                    <th scope="row" className="p-3 font-medium">
                      {i.name}
                    </th>
                    <td className="p-3">
                      {data.categories.find((c) => c.id === i.categoryId)
                        ?.name ?? 'Unspecified'}
                    </td>
                    <td className="p-3">
                      {data.sections.find((s) => s.id === i.sectionId)?.name ??
                        'Unspecified'}
                    </td>
                    <td className="p-3">
                      {data.units.find((u) => u.id === i.preferredUnitId)
                        ?.symbol ?? 'Unspecified'}
                    </td>
                    <td className="p-3">
                      <Button onClick={() => setEditing(i)}>
                        Edit {i.name}
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
      <VocabularyManager kind="category" records={data.categories} />
      <VocabularyManager kind="section" records={data.sections} />
      <ConversionControl units={data.units} />
    </>
  );
}
