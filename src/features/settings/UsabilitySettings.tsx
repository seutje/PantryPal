import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getUsabilityPreferences,
  saveUsabilityPreferences,
  getCatalog,
} from '../../lib/ipc';
import type { UsabilityPreferences, Vocabulary } from '../../lib/models';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
export function UsabilitySettings() {
  const preferences = useQuery({
    queryKey: ['usability-preferences'],
    queryFn: getUsabilityPreferences,
  });
  const catalog = useQuery({ queryKey: ['catalog'], queryFn: getCatalog });
  return (
    <section
      className="mb-6 space-y-3 rounded-xl border border-stone-200 bg-white p-6"
      aria-label="Shopping order and import preferences"
    >
      <h2 className="text-xl font-semibold">
        Shopping order and import preferences
      </h2>
      {(preferences.error || catalog.error) && (
        <p role="alert">
          {preferences.error?.message || catalog.error?.message}
        </p>
      )}
      {preferences.data && catalog.data ? (
        <Editor
          key={JSON.stringify(preferences.data)}
          initial={preferences.data}
          sections={catalog.data.sections}
        />
      ) : (
        <p role="status">Loading preferences…</p>
      )}
    </section>
  );
}
function Editor({
  initial,
  sections,
}: {
  initial: UsabilityPreferences;
  sections: Vocabulary[];
}) {
  const client = useQueryClient();
  const [draft, setDraft] = useState(initial);
  const save = useMutation({
    mutationFn: () => saveUsabilityPreferences(draft),
    onSuccess: () => {
      void client.invalidateQueries();
    },
  });
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        save.mutate();
      }}
    >
      <fieldset disabled={save.isPending} className="space-y-3">
        <legend className="sr-only">Shopping and import settings</legend>
        <FormField label="Prefill recipe URL from clipboard when I choose Import recipe">
          <input
            type="checkbox"
            checked={draft.clipboardUrlPrefill}
            onChange={(e) =>
              setDraft({ ...draft, clipboardUrlPrefill: e.target.checked })
            }
          />
        </FormField>
        <p className="text-sm text-stone-600">
          Off by default. Reads only on that action, accepts only a public HTTPS
          URL, and never opens a website automatically.
        </p>
        <p>Shopping aisle order</p>
        <ol className="space-y-2">
          {draft.sectionOrder.map((id, index) => {
            const name =
              sections.find((s) => s.id === id)?.name ?? 'Unavailable section';
            const move = (delta: number) => {
              const order = [...draft.sectionOrder];
              const other = order[index + delta];
              if (!other) return;
              order[index + delta] = id;
              order[index] = other;
              setDraft({ ...draft, sectionOrder: order });
            };
            return (
              <li key={id} className="flex items-center justify-between gap-3">
                <span>{name}</span>
                <div className="flex gap-2">
                  <Button
                    disabled={index === 0}
                    aria-label={`Move ${name} earlier`}
                    onClick={() => move(-1)}
                  >
                    Up
                  </Button>
                  <Button
                    disabled={index === draft.sectionOrder.length - 1}
                    aria-label={`Move ${name} later`}
                    onClick={() => move(1)}
                  >
                    Down
                  </Button>
                </div>
              </li>
            );
          })}
        </ol>
        <Button type="submit">Save shopping and import preferences</Button>
      </fieldset>
      {save.isSuccess && <p role="status">Preferences saved.</p>}
      {save.error && (
        <p role="alert">{save.error.message} Your choices are kept.</p>
      )}
    </form>
  );
}
