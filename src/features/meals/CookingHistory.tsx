import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getMealHistory, changeMealHistory } from '../../lib/ipc';
import type { CookedEvent, HistoryMutation } from '../../lib/models';
import { localToday } from '../../lib/local-date';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';

export function CookingHistory({ mealId }: { mealId: string }) {
  const client = useQueryClient();
  const history = useQuery({
    queryKey: ['meal-history', mealId],
    queryFn: () => getMealHistory(mealId),
  });
  const [date, setDate] = useState(localToday);
  const [editing, setEditing] = useState<CookedEvent | null>(null);
  const [undo, setUndo] = useState<CookedEvent | null>(null);
  const [pending, setPending] = useState<HistoryMutation | null>(null);
  const change = useMutation({
    mutationFn: (input: HistoryMutation) => changeMealHistory(input),
    onSuccess: (data) => {
      client.setQueryData(['meal-history', mealId], data);
      void client.invalidateQueries({ queryKey: ['meal-library'] });
      void client.invalidateQueries({ queryKey: ['week'] });
      setEditing(null);
      setUndo(null);
      setPending(null);
    },
  });
  function submit(input: HistoryMutation) {
    setPending(input);
    change.mutate(input);
  }
  return (
    <section className="pp-panel space-y-3" aria-label="Cooking history">
      <h3 className="text-lg font-semibold">Cooking history</h3>
      <p>
        Only actual cooked records update last eaten. Undo keeps a cancelled
        record. Dates follow this computer’s local calendar.
      </p>
      {history.isPending && <p role="status">Loading history…</p>}
      {history.error && <p role="alert">{history.error.message}</p>}
      {history.data && (
        <>
          <p>Last eaten: {history.data.lastCooked ?? 'Never recorded'}</p>
          <form
            className="flex flex-wrap items-end gap-2"
            onSubmit={(e) => {
              e.preventDefault();
              submit({
                actionId: crypto.randomUUID(),
                mealId,
                operation: editing
                  ? {
                      kind: 'correct',
                      eventId: editing.id,
                      expectedRevision: editing.revision,
                      localDate: date,
                      confirmed: true,
                    }
                  : {
                      kind: 'record',
                      eventId: crypto.randomUUID(),
                      localDate: date,
                    },
              });
            }}
          >
            <FormField
              label={editing ? 'Correct cooked date' : 'Date actually cooked'}
            >
              <input
                type="date"
                disabled={change.isPending || !!pending}
                required
                max={localToday()}
                value={date}
                onChange={(e) => setDate(e.target.value)}
              />
            </FormField>
            <Button type="submit" disabled={change.isPending || !!pending}>
              {editing
                ? 'Confirm date correction'
                : 'Record a past cooked meal'}
            </Button>
            {editing && (
              <Button
                disabled={change.isPending || !!pending}
                onClick={() => {
                  setEditing(null);
                  setDate(localToday());
                }}
              >
                Cancel correction
              </Button>
            )}
          </form>
          {editing && (
            <p>
              Changing this date updates the linked plan’s cooked record when
              present.
            </p>
          )}
          <ul className="space-y-2">
            {history.data.events.map((event) => (
              <li
                key={event.id}
                className="flex flex-wrap items-center justify-between gap-3 border-b border-stone-200 py-3"
              >
                <p>
                  {event.localDate}{' '}
                  {event.voided
                    ? '· cancelled'
                    : event.planEntryId
                      ? '· planned occurrence'
                      : '· cooking record'}
                  {!event.planEntryId && event.originPlanEntryId
                    ? ' (plan entry removed)'
                    : ''}
                </p>
                {!event.voided && (
                  <div className="flex flex-wrap gap-2">
                    <Button
                      variant="quiet"
                      disabled={change.isPending || !!pending}
                      onClick={() => {
                        setEditing(event);
                        setDate(event.localDate);
                        setUndo(null);
                      }}
                    >
                      Correct date {event.localDate}
                    </Button>
                    <Button
                      variant="quiet"
                      disabled={change.isPending || !!pending}
                      onClick={() => setUndo(event)}
                    >
                      Undo record {event.localDate}
                    </Button>
                  </div>
                )}
              </li>
            ))}
          </ul>
          {history.data.events.length === 0 && <p>No cooking recorded yet.</p>}
          {undo && (
            <div role="group" aria-label="Confirm history undo">
              <p>
                Undo cooking recorded on {undo.localDate}? It will no longer
                count towards last eaten. A linked plan returns to planned.
              </p>
              <Button
                variant="danger"
                disabled={change.isPending || !!pending}
                onClick={() =>
                  submit({
                    actionId: crypto.randomUUID(),
                    mealId,
                    operation: {
                      kind: 'undo',
                      eventId: undo.id,
                      expectedRevision: undo.revision,
                      confirmed: true,
                    },
                  })
                }
              >
                Confirm history undo
              </Button>
              <Button
                disabled={change.isPending || !!pending}
                onClick={() => setUndo(null)}
              >
                Keep record
              </Button>
            </div>
          )}
        </>
      )}
      {change.error && (
        <div role="alert">
          <p>
            {change.error.message} Your date is kept. Retry uses the same action
            to prevent a duplicate record.
          </p>
          {pending && (
            <>
              <Button
                disabled={change.isPending}
                onClick={() => change.mutate(pending)}
              >
                Retry history action
              </Button>
              <Button
                onClick={() => {
                  setPending(null);
                  change.reset();
                  setEditing(null);
                  setUndo(null);
                  void history.refetch();
                }}
              >
                Reload history
              </Button>
            </>
          )}
        </div>
      )}
    </section>
  );
}
