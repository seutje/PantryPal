import type { MealLibrary, MealRecord } from '../../lib/models';
import { Button } from '../../components/ui/Button';
import { fieldText } from './field-text';
import { CookingHistory } from './CookingHistory';

export function MealDetail({
  record,
  library,
  onEdit,
  onBack,
  onRetire,
}: {
  record: MealRecord;
  library: MealLibrary;
  onEdit: () => void;
  onBack: () => void;
  onRetire: () => void;
}) {
  const { meal } = record;
  return (
    <article className="space-y-4">
      <div className="flex flex-wrap gap-2">
        <Button onClick={onBack}>Back to meals</Button>
        <Button disabled={meal.retired} onClick={onEdit}>
          Edit meal
        </Button>
        <Button disabled={meal.retired} onClick={onRetire}>
          Retire meal
        </Button>
      </div>
      <h2 className="text-2xl font-semibold">{meal.title}</h2>
      <p>{meal.description}</p>
      <p>
        {record.servingsDisplay} servings · Preparation:{' '}
        {meal.prepMinutes ?? 'unknown'} min · Cooking:{' '}
        {meal.cookMinutes ?? 'unknown'} min
      </p>
      <p>Last cooked: {record.lastCooked ?? 'Never recorded'}</p>
      <p>Tags: {meal.tags.join(', ') || 'None'}</p>
      <h3 className="text-lg font-semibold">Ingredients</h3>
      <ol className="list-decimal space-y-3 pl-6">
        {meal.ingredients.map((line, index) => {
          const ingredient = library.catalog.ingredients.find(
            (i) => i.id === line.ingredientId,
          );
          const unit = library.catalog.units.find((u) => u.id === line.unitId);
          return (
            <li key={line.id}>
              <p>
                {record.quantityDisplays[index] ?? 'Unknown amount'}{' '}
                {unit?.label ?? 'unspecified unit'} —{' '}
                {ingredient?.name ?? 'Unresolved ingredient'}
                {line.optional ? ' (optional)' : ''}
              </p>
              {!line.quantity && (
                <p className="text-sm text-amber-800">
                  Quantity needs review for shopping; no amount will be
                  invented.
                </p>
              )}
              {unit?.dimension === 'informal' && (
                <p className="text-sm text-amber-800">
                  Informal unit; conversion needs review.
                </p>
              )}
              <p className="whitespace-pre-wrap">
                Original: {line.rawText || '(no wording entered)'}
              </p>
              <p>
                Entered amount: {line.amountText ?? '(legacy exact quantity)'}
              </p>
              {line.sourceConvention && (
                <p>Source convention: {line.sourceConvention}</p>
              )}
              {line.note && <p>{line.note}</p>}
            </li>
          );
        })}
      </ol>
      <h3 className="text-lg font-semibold">Instructions</h3>
      <p className="whitespace-pre-wrap">
        {meal.instructions || 'No instructions yet.'}
      </p>
      <h3 className="text-lg font-semibold">Notes</h3>
      <p className="whitespace-pre-wrap">{meal.notes || 'None'}</p>
      <p>
        Source: {meal.sourceSite || 'Unspecified'} {meal.sourceUrl}
      </p>
      <p>
        Entered servings: {meal.baseServingsText ?? '(legacy exact quantity)'}
      </p>
      <h3 className="text-lg font-semibold">Custom fields</h3>
      <dl className="grid gap-2">
        {library.fields
          .filter(
            (f) =>
              !f.retired || meal.customFields.some((v) => v.fieldId === f.id),
          )
          .map((f) => (
            <div key={f.id}>
              <dt className="font-medium">
                {f.label}
                {f.retired ? ' (retired)' : ''}
              </dt>
              <dd>
                {fieldText(
                  f,
                  meal.customFields.find((v) => v.fieldId === f.id)?.value ??
                    null,
                )}
                {f.unitLabel ? ` ${f.unitLabel}` : ''}
              </dd>
            </div>
          ))}
      </dl>
      <CookingHistory mealId={meal.id} />
    </article>
  );
}
