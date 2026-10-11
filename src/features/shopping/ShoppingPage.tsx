import { EmptyState } from '../../components/EmptyState';

export function ShoppingPage({ weekStart }: { weekStart?: string | null }) {
  return (
    <EmptyState title="A list for your next shop">
      {weekStart && <>Selected week: {weekStart}. </>}
      Your groceries will be grouped here once shopping-list generation is
      available.
    </EmptyState>
  );
}
