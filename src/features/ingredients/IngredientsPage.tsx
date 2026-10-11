import { EmptyState } from '../../components/EmptyState';
import { useQuery } from '@tanstack/react-query';
import { getCatalog, nativeAvailable } from '../../lib/ipc';

export function IngredientsPage() {
  const catalog = useQuery({
    queryKey: ['catalog'],
    queryFn: getCatalog,
    enabled: nativeAvailable(),
  });
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
  if (catalog.data.ingredients.length === 0)
    return (
      <EmptyState title="No active ingredients">
        Ingredient editing will be available in a later update.
      </EmptyState>
    );
  return (
    <section
      aria-label="Ingredient catalog"
      className="rounded-xl border border-stone-200 bg-white p-6"
    >
      <p className="mb-4 text-sm text-stone-600">
        Your local catalog. Ingredient editing will be available in a later
        update.
      </p>
      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm">
          <caption className="sr-only">
            Saved ingredients and their preferred shopping units
          </caption>
          <thead className="border-b border-stone-200">
            <tr>
              <th scope="col" className="p-3">
                Ingredient
              </th>
              <th scope="col" className="p-3">
                Category
              </th>
              <th scope="col" className="p-3">
                Shopping section
              </th>
              <th scope="col" className="p-3">
                Preferred unit
              </th>
            </tr>
          </thead>
          <tbody>
            {catalog.data.ingredients.map((ingredient) => (
              <tr key={ingredient.id} className="border-b border-stone-100">
                <th scope="row" className="p-3 font-medium">
                  {ingredient.name}
                </th>
                <td className="p-3">
                  {catalog.data.categories.find(
                    (category) => category.id === ingredient.categoryId,
                  )?.name ?? 'Unspecified'}
                </td>
                <td className="p-3">
                  {catalog.data.sections.find(
                    (section) => section.id === ingredient.sectionId,
                  )?.name ?? 'Unspecified'}
                </td>
                <td className="p-3">
                  {catalog.data.units.find(
                    (unit) => unit.id === ingredient.preferredUnitId,
                  )?.symbol ?? 'Unspecified'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
