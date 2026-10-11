import { useState } from 'react';
import {
  useQuery,
  useMutation,
  useQueryClient,
  keepPreviousData,
} from '@tanstack/react-query';
import { getWeek, applyPlan } from '../../lib/ipc';
import type {
  MealLibrary,
  MealRecord,
  PlanOperation,
  PlanMutation,
} from '../../lib/models';
import { localToday, dateLabel } from '../../lib/local-date';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { AddPlanForm } from './PlanForms';
import { PlanCard } from './PlanCard';
import { MealDetail } from '../meals/MealDetail';

export function WeekPage({
  date,
  onDateChange,
  onOpenMeal,
  onShopping,
}: {
  date?: string;
  onDateChange?: (date: string) => void;
  onOpenMeal?: (record: MealRecord, library: MealLibrary) => void;
  onShopping?: (week: string) => void;
}) {
  const client = useQueryClient();
  const [anchor, setAnchor] = useState(localToday);
  const [adding, setAdding] = useState(false);
  const [detail, setDetail] = useState<MealRecord | null>(null);
  const [pending, setPending] = useState<PlanMutation | null>(null);
  const selectedDate = date ?? anchor;
  const week = useQuery({
    queryKey: ['week', selectedDate],
    queryFn: () => getWeek(selectedDate),
    placeholderData: keepPreviousData,
  });
  const mutation = useMutation({
    mutationFn: (input: PlanMutation) => applyPlan(input),
    onSuccess: (data) => {
      client.setQueryData(['week', selectedDate], data);
      void client.invalidateQueries({ queryKey: ['meal-library'] });
      void client.invalidateQueries({ queryKey: ['meal-history'] });
      setAdding(false);
      setPending(null);
    },
  });
  const busy =
    mutation.isPending ||
    !!pending ||
    week.isFetching ||
    week.isPlaceholderData;
  function navigate(next: string) {
    setAnchor(next);
    onDateChange?.(next);
    setAdding(false);
    setDetail(null);
  }
  function apply(operation: PlanOperation) {
    if (!week.data || busy) return;
    const input = {
      actionId: crypto.randomUUID(),
      weekStart: week.data.weekStart,
      expectedRevision: week.data.revision,
      operation,
    };
    setPending(input);
    mutation.mutate(input);
  }
  const data = week.data;
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-end gap-2">
        <Button
          disabled={busy || !data?.previousWeek}
          onClick={() => data?.previousWeek && navigate(data.previousWeek)}
        >
          Previous week
        </Button>
        <Button disabled={busy} onClick={() => navigate(localToday())}>
          Current week
        </Button>
        <Button
          disabled={busy || !data?.nextWeek}
          onClick={() => data?.nextWeek && navigate(data.nextWeek)}
        >
          Next week
        </Button>
        <FormField label="Week containing">
          <input
            type="date"
            value={selectedDate}
            disabled={mutation.isPending || !!pending}
            onChange={(e) => e.target.value && navigate(e.target.value)}
          />
        </FormField>
      </div>
      {week.isPending && <p role="status">Loading week…</p>}
      {week.error && (
        <p role="alert">
          {week.error.message} Choose another date to continue.
        </p>
      )}
      {data && (
        <>
          <h2 className="text-xl font-semibold">
            {dateLabel(data.weekStart)} – {dateLabel(data.weekEnd)}
          </h2>
          {detail ? (
            <MealDetail
              record={
                data.library.meals.find((m) => m.meal.id === detail.meal.id) ??
                detail
              }
              library={data.library}
              onBack={() => setDetail(null)}
              onEdit={() => onOpenMeal?.(detail, data.library)}
              onRetire={() => onOpenMeal?.(detail, data.library)}
            />
          ) : (
            <>
              <div className="flex flex-wrap gap-2">
                <Button disabled={busy} onClick={() => setAdding(true)}>
                  Add meal to week
                </Button>
                <Button
                  disabled={!onShopping || busy}
                  onClick={() => onShopping?.(data.weekStart)}
                >
                  Open shopping for this week
                </Button>
              </div>
              <p className="text-sm text-stone-600">
                Shopping includes planned and cooked meals; skipped meals are
                excluded. Scheduling and skipping do not record cooking.
              </p>
              {adding && (
                <AddPlanForm
                  meals={data.library.meals}
                  days={data.days}
                  busy={busy}
                  onApply={apply}
                  onCancel={() => setAdding(false)}
                />
              )}
              {[null, ...data.days].map((day) => {
                const entries = data.entries.filter((e) => e.localDate === day);
                return (
                  <section
                    key={day ?? 'unscheduled'}
                    className="space-y-3 rounded-lg border border-stone-200 p-4"
                    aria-label={
                      day ? dateLabel(day) : 'This week (unscheduled)'
                    }
                  >
                    <h2 className="text-lg font-semibold">
                      {day ? dateLabel(day) : 'This week (unscheduled)'}
                    </h2>
                    {entries.length === 0 && (
                      <p className="text-stone-600">No meals planned.</p>
                    )}
                    <ol className="space-y-3">
                      {entries.map((entry, index) => {
                        const meal = data.library.meals.find(
                          (m) => m.meal.id === entry.mealId,
                        );
                        return (
                          meal && (
                            <PlanCard
                              key={`${entry.id}-${data.revision}`}
                              entry={entry}
                              meal={meal}
                              fields={data.library.fields}
                              days={data.days}
                              busy={busy}
                              first={index === 0}
                              last={index === entries.length - 1}
                              onApply={apply}
                              onOpen={() =>
                                onOpenMeal
                                  ? onOpenMeal(meal, data.library)
                                  : setDetail(meal)
                              }
                              onMove={(delta) => {
                                const ids = entries.map((e) => e.id);
                                const other = ids[index + delta];
                                if (!other) return;
                                ids[index + delta] = entry.id;
                                ids[index] = other;
                                apply({
                                  kind: 'reorder',
                                  localDate: day,
                                  entryIds: ids,
                                });
                              }}
                            />
                          )
                        );
                      })}
                    </ol>
                  </section>
                );
              })}
            </>
          )}
        </>
      )}
      {mutation.error && (
        <div role="alert">
          <p>
            {mutation.error.message} Your inputs are kept. Retry uses the same
            action; reload if another edit changed this week.
          </p>
          {pending && (
            <>
              <Button
                disabled={mutation.isPending}
                onClick={() => mutation.mutate(pending)}
              >
                Retry plan action
              </Button>
              <Button
                disabled={mutation.isPending}
                onClick={() => {
                  setPending(null);
                  mutation.reset();
                  void week.refetch();
                }}
              >
                Reload week
              </Button>
            </>
          )}
        </div>
      )}
    </div>
  );
}
