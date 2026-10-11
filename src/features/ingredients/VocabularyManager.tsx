import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import {
  createVocabulary,
  updateVocabulary,
  retireVocabulary,
} from '../../lib/ipc';
import type { Vocabulary, VocabularyKind } from '../../lib/models';

function VocabularyForm({
  kind,
  record,
  onClose,
}: {
  kind: VocabularyKind;
  record: Vocabulary | null;
  onClose: () => void;
}) {
  const [name, setName] = useState(record?.name ?? '');
  const [order, setOrder] = useState(record?.sortOrder ?? 0);
  const [confirm, setConfirm] = useState(false);
  const client = useQueryClient();
  const done = async () => {
    await client.invalidateQueries({ queryKey: ['catalog'] });
    onClose();
  };
  const save = useMutation({
    mutationFn: () =>
      record
        ? updateVocabulary(kind, record.id, name, order)
        : createVocabulary(kind, name, order),
    onSuccess: done,
  });
  const retire = useMutation({
    mutationFn: () => retireVocabulary(kind, record!.id, true),
    onSuccess: done,
  });
  const pending = save.isPending || retire.isPending;
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        save.mutate();
      }}
      className="mt-3 space-y-3 rounded border border-stone-200 p-4"
    >
      <FormField label={kind === 'category' ? 'Category name' : 'Section name'}>
        <input
          required
          maxLength={120}
          disabled={pending}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </FormField>
      {kind === 'section' && (
        <FormField label="Shopping section order">
          <input
            type="number"
            required
            min={0}
            step={1}
            disabled={pending}
            value={order}
            onChange={(e) => setOrder(Number(e.target.value))}
          />
        </FormField>
      )}
      <div className="flex gap-3">
        <Button type="submit" disabled={pending}>
          Save {kind}
        </Button>
        <Button disabled={pending} onClick={onClose}>
          Cancel {kind}
        </Button>
      </div>
      {record && (
        <>
          <p className="text-sm">
            Retirement hides this choice from new assignments and preserves
            existing ingredient references.
          </p>
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={confirm}
              onChange={(e) => setConfirm(e.target.checked)}
            />
            Confirm retirement of {record.name}
          </label>
          <Button
            disabled={!confirm || pending}
            onClick={() => retire.mutate()}
          >
            Retire {kind}
          </Button>
        </>
      )}
      {(save.isError || retire.isError) && (
        <p role="alert">{save.error?.message ?? retire.error?.message}</p>
      )}
    </form>
  );
}
export function VocabularyManager({
  kind,
  records,
}: {
  kind: VocabularyKind;
  records: Vocabulary[];
}) {
  const [editing, setEditing] = useState<Vocabulary | null | undefined>(
    undefined,
  );
  return (
    <details className="mt-4 rounded-xl border border-stone-200 bg-white p-4">
      <summary className="cursor-pointer font-medium">
        Manage {kind === 'category' ? 'categories' : 'shopping sections'}
      </summary>
      <ul className="my-3 space-y-2">
        {records
          .filter((v) => !v.retired)
          .map((v) => (
            <li key={v.id} className="flex items-center justify-between gap-3">
              <span>
                {v.name}
                {kind === 'section' ? ` (order ${v.sortOrder})` : ''}
              </span>
              <Button onClick={() => setEditing(v)}>Edit {v.name}</Button>
            </li>
          ))}
      </ul>
      {editing === undefined ? (
        <Button onClick={() => setEditing(null)}>Add {kind}</Button>
      ) : (
        <VocabularyForm
          key={editing?.id ?? 'new'}
          kind={kind}
          record={editing}
          onClose={() => setEditing(undefined)}
        />
      )}
    </details>
  );
}
