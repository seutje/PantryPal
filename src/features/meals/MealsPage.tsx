import { EmptyState } from '../../components/EmptyState';
import { Button } from '../../components/ui/Button';

export function MealsPage() {
  return (
    <>
      <div className="mb-4 flex items-center gap-3">
        <Button disabled aria-describedby="import-availability">
          Import recipe
        </Button>
        <p id="import-availability" className="text-sm text-stone-600">
          Recipe importing is coming in a later update.
        </p>
      </div>
      <EmptyState title="Your favourite meals, in one place">
        Meal creation and editing will be available in a later update.
      </EmptyState>
    </>
  );
}
