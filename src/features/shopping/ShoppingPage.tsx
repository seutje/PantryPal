import { ShoppingForm } from './ShoppingForm';
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getWeek,
  getShopping,
  changeShopping,
  copyShopping,
  NativeError,
} from '../../lib/ipc';
import type { ShoppingMutation, ShoppingOperation } from '../../lib/models';
import { localToday } from '../../lib/local-date';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { EmptyState } from '../../components/EmptyState';

export function ShoppingPage({ weekStart }: { weekStart?: string | null }) {
  const client = useQueryClient();
  const [date, setDate] = useState(weekStart ?? localToday);
  const [adding, setAdding] = useState(false);
  const [editing, setEditing] = useState<string | null>(null);
  const [pending, setPending] = useState<ShoppingMutation | null>(null);
  const [copied, setCopied] = useState(false);
  const week = useQuery({
    queryKey: ['week', date],
    queryFn: () => getWeek(date),
  });
  const selected = week.data?.weekStart;
  const shopping = useQuery({
    queryKey: ['shopping', selected],
    queryFn: () => getShopping(selected!),
    enabled: !!selected,
  });
  const mutation = useMutation({
    mutationFn: (input: ShoppingMutation) => changeShopping(input),
    onSuccess: (view) => {
      client.setQueryData(['shopping', view.weekStart], view);
      setPending(null);
      setAdding(false);
      setEditing(null);
      setCopied(false);
    },
    onError: (error) => {
      if (error instanceof NativeError && error.code === 'validation')
        setPending(null);
    },
  });
  const copy = useMutation({
    mutationFn: () => copyShopping(selected!, shopping.data!.revision),
    onSuccess: () => setCopied(true),
  });
  const busy = mutation.isPending || !!pending;
  const run = (operation: ShoppingOperation) => {
    if (!shopping.data || busy) return;
    const input = {
      actionId: crypto.randomUUID(),
      weekStart: shopping.data.weekStart,
      expectedRevision: shopping.data.revision,
      operation,
    };
    setPending(input);
    mutation.mutate(input);
  };
  const reload = () => {
    setPending(null);
    mutation.reset();
    copy.reset();
    setCopied(false);
    setEditing(null);
    setAdding(false);
    void shopping.refetch();
    void week.refetch();
  };
  const data = shopping.data;
  const groups = data
    ? [...new Set(data.items.map((item) => item.section))]
    : [];
  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-end gap-3">
        <FormField label="Shopping week">
          <input
            type="date"
            value={date}
            disabled={busy || adding || !!editing}
            onChange={(event) => {
              if (event.target.value) {
                setDate(event.target.value);
                setCopied(false);
                copy.reset();
              }
            }}
          />
        </FormField>
        <Button
          disabled={!data || busy || adding || !!editing}
          onClick={() => run({ kind: 'refresh' })}
        >
          {data?.generated ? 'Refresh list' : 'Generate list'}
        </Button>
        <Button
          disabled={!data || busy || adding || !!editing}
          onClick={() => setAdding(true)}
        >
          Add manual item
        </Button>
        <Button
          disabled={
            !data ||
            data.stale ||
            busy ||
            adding ||
            !!editing ||
            copy.isPending ||
            !data.items.some((item) => !item.excluded && !item.orphaned)
          }
          onClick={() => {
            setCopied(false);
            copy.mutate();
          }}
        >
          Copy list
        </Button>
        <Button disabled={mutation.isPending} onClick={reload}>
          Reload
        </Button>
      </div>
      <p className="text-sm text-stone-600">
        Planned and cooked meals count once per occurrence, including the
        unscheduled pool. Skipped meals are excluded. Refresh preserves your
        edits. Change section order in Ingredients.
      </p>
      {selected && <p>Week of {selected}</p>}
      {(week.isError || shopping.isError) && (
        <p role="alert">
          Could not load this week. Your saved list has not been reset. Use
          Reload to retry.
        </p>
      )}
      {(week.isPending || shopping.isFetching) && (
        <p role="status">Loading shopping list…</p>
      )}
      {data?.stale && (
        <p
          role="status"
          className="rounded-lg border border-amber-400 bg-amber-50 p-3"
        >
          Shopping list needs refresh. Meals, plans or measurement settings have
          changed.
        </p>
      )}
      {mutation.isError && (
        <div role="alert" className="space-y-2">
          <p>{mutation.error.message}</p>
          <p>
            {pending
              ? 'Your action is retained for retry. Reload to review the latest saved list.'
              : 'Correct the input and save again.'}
          </p>
          <Button
            disabled={mutation.isPending || !pending}
            onClick={() => pending && mutation.mutate(pending)}
          >
            Retry action
          </Button>
        </div>
      )}
      {copy.isError && <p role="alert">{copy.error.message}</p>}
      {copied && <p role="status">Copied. Paste into your notes or message.</p>}
      {adding && data && (
        <ShoppingForm
          key="manual"
          view={data}
          busy={busy}
          onCancel={() => setAdding(false)}
          onSave={run}
        />
      )}
      {data && data.items.length === 0 && (
        <EmptyState title="A list for your next shop">
          Add meals to this week and select Generate list, or add a manual item.
        </EmptyState>
      )}
      {data &&
        groups.map((section) => (
          <section key={section} aria-label={section} className="space-y-3">
            <h2 className="text-xl font-semibold">{section}</h2>
            {data.items
              .filter((item) => item.section === section)
              .map((item) => (
                <article
                  key={item.id}
                  className="rounded-xl border border-stone-200 bg-white p-4 space-y-2"
                >
                  <div className="flex flex-wrap items-center gap-3">
                    <label className="flex items-center gap-2">
                      <input
                        type="checkbox"
                        checked={item.checked}
                        disabled={busy || adding || !!editing}
                        onChange={(event) =>
                          run({
                            kind: 'check',
                            itemId: item.id,
                            checked: event.target.checked,
                          })
                        }
                      />
                      <span>{item.label}</span>
                    </label>
                    <span className="font-medium">{item.display}</span>
                    {item.adjustment && item.key && (
                      <span className="text-sm">Amount overridden</span>
                    )}
                    {!item.key && <span className="text-sm">Manual item</span>}
                    {item.approximate && (
                      <span className="text-sm text-amber-800">
                        Approximate
                      </span>
                    )}
                    {item.rounded && (
                      <span className="text-sm text-amber-800">
                        Rounded display
                      </span>
                    )}
                    {item.excluded && (
                      <span className="text-sm">Excluded from copy</span>
                    )}
                    {item.orphaned && (
                      <span className="text-sm text-amber-800">
                        Source no longer present; retained for review and
                        omitted from copy
                      </span>
                    )}
                  </div>
                  {item.warnings.map((warning) => (
                    <p key={warning} className="text-sm text-amber-800">
                      {warning}
                    </p>
                  ))}
                  <div className="flex flex-wrap gap-2">
                    <Button
                      disabled={busy || adding || !!editing}
                      onClick={() => setEditing(item.id)}
                    >
                      Edit amount for {item.label}
                    </Button>
                    <Button
                      disabled={busy || adding || !!editing}
                      onClick={() =>
                        run({
                          kind: 'exclude',
                          itemId: item.id,
                          excluded: !item.excluded,
                        })
                      }
                    >
                      {item.excluded ? 'Include' : 'Exclude'} {item.label}
                    </Button>
                    {item.adjustment && item.key && (
                      <Button
                        disabled={busy || adding || !!editing}
                        onClick={() => {
                          if (
                            window.confirm(
                              'Reset this amount to the generated quantity?',
                            )
                          )
                            run({
                              kind: 'reset',
                              itemId: item.id,
                              confirmed: true,
                            });
                        }}
                      >
                        Reset amount for {item.label}
                      </Button>
                    )}
                  </div>
                  {editing === item.id && (
                    <ShoppingForm
                      key={item.id}
                      item={item}
                      view={data}
                      busy={busy}
                      onCancel={() => setEditing(null)}
                      onSave={run}
                    />
                  )}
                  {item.sources.length > 0 && (
                    <details>
                      <summary>
                        Recipe quantities ({item.sources.length})
                      </summary>
                      <ul className="list-disc pl-5">
                        {item.sources.map((source) => (
                          <li key={`${source.entryId}:${source.line.id}`}>
                            {source.mealTitle}: {source.line.rawText} · planned{' '}
                            {source.plannedServings.numerator}/
                            {source.plannedServings.denominator} servings, base{' '}
                            {source.baseServings.numerator}/
                            {source.baseServings.denominator}
                            {source.line.optional ? ' · optional' : ''}
                          </li>
                        ))}
                      </ul>
                    </details>
                  )}
                </article>
              ))}
          </section>
        ))}
      {data && data.items.some((item) => !item.excluded && !item.orphaned) && (
        <details>
          <summary>Copy preview</summary>
          <pre className="whitespace-pre-wrap rounded-lg bg-stone-100 p-4 text-sm">
            {data.text}
          </pre>
        </details>
      )}
    </div>
  );
}
