import { ShoppingRow } from './ShoppingRow';
import { ShoppingForm } from './ShoppingForm';
import { DraftContext, useDraftGuard } from '../../lib/drafts';
import { useContext, useEffect, useRef, useState } from 'react';
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
  const drafts = useContext(DraftContext);
  const editOpener = useRef<HTMLElement | null>(null);
  const endEdit = () => {
    setEditing(null);
    editOpener.current?.focus();
  };
  const client = useQueryClient();
  const [date, setDate] = useState(weekStart ?? localToday);
  const [adding, setAdding] = useState(false);
  const [editing, setEditing] = useState<string | null>(null);
  useEffect(() => {
    if (!editing && !adding) editOpener.current?.focus();
  }, [editing, adding]);
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
  useDraftGuard(!!pending, mutation.isPending);
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
    if (!drafts.canLeave()) return;
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
    ? [
        ...new Set(
          data.items
            .filter((item) => !item.excluded && !item.orphaned)
            .map((item) => item.section),
        ),
      ]
    : [];
  return (
    <div className="space-y-5">
      <div className="pp-toolbar">
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
          onClick={() => {
            editOpener.current = document.activeElement as HTMLElement;
            setAdding(true);
          }}
        >
          Add manual item
        </Button>
        <Button
          variant="primary"
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
        <Button variant="quiet" disabled={mutation.isPending} onClick={reload}>
          Reload
        </Button>
      </div>
      <p className="text-sm text-stone-600">
        Planned and cooked meals count once per occurrence, including the
        unscheduled pool. Skipped meals are excluded. Refresh preserves your
        edits. Change section order in Settings.
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
          changed. Refresh before copying; your edits and checkmarks are kept.
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
      {data && (
        <div className={editing ? 'pp-catalog-layout' : undefined}>
          <div className="space-y-5">
            {groups.map((section) => (
              <section
                key={section}
                aria-label={section}
                className="overflow-visible rounded-lg border border-stone-200"
              >
                <h2 className="bg-stone-100 px-4 py-3 font-semibold">
                  {section}
                </h2>
                {data.items
                  .filter(
                    (item) =>
                      item.section === section &&
                      !item.excluded &&
                      !item.orphaned,
                  )
                  .map((item) => (
                    <ShoppingRow
                      key={item.id}
                      item={item}
                      disabled={busy || adding || !!editing}
                      onEdit={() => {
                        editOpener.current =
                          document.activeElement as HTMLElement;
                        setEditing(item.id);
                      }}
                      onApply={run}
                    />
                  ))}
              </section>
            ))}
            {data.items.some((item) => item.excluded || item.orphaned) && (
              <details className="pp-panel">
                <summary className="font-semibold">
                  Excluded and retained items (
                  {
                    data.items.filter((item) => item.excluded || item.orphaned)
                      .length
                  }
                  )
                </summary>
                <div className="mt-3">
                  {data.items
                    .filter((item) => item.excluded || item.orphaned)
                    .map((item) => (
                      <ShoppingRow
                        key={item.id}
                        item={item}
                        disabled={busy || adding || !!editing}
                        onEdit={() => {
                          editOpener.current =
                            document.activeElement as HTMLElement;
                          setEditing(item.id);
                        }}
                        onApply={run}
                      />
                    ))}
                </div>
              </details>
            )}
          </div>
          {editing && data.items.find((item) => item.id === editing) && (
            <aside className="pp-catalog-editor">
              <ShoppingForm
                key={editing}
                item={data.items.find((item) => item.id === editing)!}
                view={data}
                busy={busy}
                onCancel={endEdit}
                onSave={run}
              />
            </aside>
          )}
        </div>
      )}
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
