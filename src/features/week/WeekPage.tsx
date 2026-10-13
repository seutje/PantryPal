import { DraftContext, useDraftGuard } from '../../lib/drafts';
import { useContext, useEffect, useRef, useState } from 'react';
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
import { PlanCard } from './PlanCard';
import { MealDetail } from '../meals/MealDetail';
import { MealPickerDialog } from './MealPickerDialog';

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
  const drafts = useContext(DraftContext);
  const focusOccurrence = useRef<string | null>(null);
  const addButton = useRef<HTMLButtonElement>(null);
  const client = useQueryClient();
  const [anchor, setAnchor] = useState(localToday);
  const [pickerDay, setPickerDay] = useState<string | null | undefined>(
    undefined,
  );
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
    onSuccess: (data, request) => {
      if (request.operation.kind !== 'add' && 'entryId' in request.operation)
        focusOccurrence.current = request.operation.entryId;
      client.setQueryData(['week', selectedDate], data);
      void client.invalidateQueries({ queryKey: ['meal-library'] });
      void client.invalidateQueries({ queryKey: ['meal-history'] });

      setPickerDay(undefined);
      setPending(null);
    },
  });
  const busy =
    mutation.isPending ||
    !!pending ||
    week.isFetching ||
    week.isPlaceholderData;
  useDraftGuard(!!pending, mutation.isPending);
  useEffect(() => {
    if (!focusOccurrence.current) return;
    const card = Array.from(
      document.querySelectorAll<HTMLElement>('[data-plan-id]'),
    ).find((element) => element.dataset.planId === focusOccurrence.current);
    (
      card?.querySelector<HTMLButtonElement>('.pp-plan-title') ??
      addButton.current
    )?.focus();
    focusOccurrence.current = null;
  }, [week.data?.revision]);
  function navigate(next: string) {
    if (!drafts.canLeave()) return;
    setAnchor(next);
    onDateChange?.(next);

    setPickerDay(undefined);
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
  const planError = mutation.error && (
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
  );
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
                <Button
                  ref={addButton}
                  disabled={busy}
                  variant="primary"
                  onClick={() => setPickerDay(null)}
                >
                  Add meal to week
                </Button>
                <Button
                  disabled={!onShopping || busy}
                  onClick={() => {
                    if (drafts.canLeave()) onShopping?.(data.weekStart);
                  }}
                >
                  Open shopping for this week
                </Button>
              </div>
              <p className="text-sm text-stone-600">
                Shopping includes planned and cooked meals; skipped meals are
                excluded. Scheduling and skipping do not record cooking.
              </p>
              {[...data.days, null].map((day) => {
                const entries = data.entries.filter((e) => e.localDate === day);
                return (
                  <section
                    key={day ?? 'unscheduled'}
                    className="pp-agenda-day"
                    data-today={day === localToday()}
                    aria-label={
                      day ? dateLabel(day) : 'This week (unscheduled)'
                    }
                  >
                    <div className="flex flex-col items-start gap-2">
                      <h2 className="text-lg font-semibold">
                        {day ? dateLabel(day) : 'This week (unscheduled)'}
                      </h2>
                      {
                        <Button
                          variant="quiet"
                          disabled={busy}
                          onClick={() => {
                            setPickerDay(day);
                          }}
                        >
                          Add meal
                        </Button>
                      }
                    </div>
                    <div className="min-w-0">
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
                                catalog={data.library.catalog}
                                days={data.days}
                                busy={busy}
                                first={index === 0}
                                last={index === entries.length - 1}
                                onApply={apply}
                                onOpen={() => {
                                  if (drafts.canLeave()) {
                                    if (onOpenMeal)
                                      onOpenMeal(meal, data.library);
                                    else setDetail(meal);
                                  }
                                }}
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
                    </div>
                  </section>
                );
              })}
            </>
          )}
        </>
      )}
      {pickerDay !== undefined && data && (
        <MealPickerDialog
          day={pickerDay}
          days={data.days}
          library={data.library}
          busy={busy}
          saving={mutation.isPending}
          onClose={() => setPickerDay(undefined)}
          onSelect={(record, servings, day) =>
            apply({
              kind: 'add',
              entryId: crypto.randomUUID(),
              mealId: record.meal.id,
              localDate: day,
              servings,
            })
          }
        >
          {planError}
        </MealPickerDialog>
      )}
      {pickerDay === undefined && planError}
    </div>
  );
}
