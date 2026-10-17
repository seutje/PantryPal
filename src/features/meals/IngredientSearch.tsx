import { useId, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  createIngredient,
  ingredientCandidates,
  searchIngredients,
} from '../../lib/ipc';
import type { Ingredient } from '../../lib/models';
import { FormField } from '../../components/ui/FormField';

const normalize = (text: string) =>
  text.trim().replace(/\s+/g, ' ').toLocaleLowerCase();

export function IngredientSearch({
  selected,
  onEdit,
  onSelect,
  onBusyChange,
}: {
  selected: Ingredient | undefined;
  onEdit: (text: string) => void;
  onSelect: (ingredient: Ingredient) => void;
  onBusyChange: (busy: boolean) => void;
}) {
  const id = useId();
  const client = useQueryClient();
  const [text, setText] = useState<string | null>(null);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');
  const value = selected?.name ?? text ?? '';
  const query = value.trim();
  const matches = useQuery({
    queryKey: ['ingredient-search', query],
    queryFn: () => searchIngredients(query),
    enabled: open && !!query && query.length <= 120,
    retry: false,
  });
  const results = matches.data ?? [];
  const exact = results.find((item) =>
    [item.name, ...item.aliases].some(
      (name) => normalize(name) === normalize(query),
    ),
  );
  const ready = !!query && !matches.isFetching && matches.isSuccess;
  const canCreate = ready && !exact;
  const optionCount = results.length + (canCreate ? 1 : 0);
  const choose = (ingredient: Ingredient) => {
    onSelect(ingredient);
    setText(null);
    setOpen(false);
    setActive(-1);
    setError('');
  };
  const create = async () => {
    if (!canCreate || pending) return;
    setPending(true);
    onBusyChange(true);
    setError('');
    try {
      // Recheck exact identities, including retired names, before writing.
      const existing = await ingredientCandidates([query], null);
      if (existing.length) {
        const match = existing[0]!;
        if (match.retired)
          throw new Error(
            'This name belongs to a retired ingredient. Restore or rename it in Ingredients.',
          );
        choose(match);
        return;
      }
      const ingredient = await createIngredient({
        name: query,
        categoryId: null,
        sectionId: null,
        preferredUnitId: null,
        aliases: [],
        seasonMonths: [],
        notes: '',
      });
      choose(ingredient);
      await Promise.all([
        client.invalidateQueries({ queryKey: ['catalog'] }),
        client.invalidateQueries({ queryKey: ['meal-library'] }),
        client.invalidateQueries({ queryKey: ['storage-status'] }),
        client.invalidateQueries({ queryKey: ['ingredient-search'] }),
      ]);
    } catch (cause) {
      setError(
        cause instanceof Error
          ? cause.message
          : 'Could not add ingredient. Try again.',
      );
    } finally {
      setPending(false);
      onBusyChange(false);
    }
  };
  const activate = (index: number) => {
    const item = results[index];
    if (item) choose(item);
    else if (index === results.length) void create();
  };
  return (
    <div
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) setOpen(false);
      }}
    >
      <div className="pp-ingredient-anchor">
        <FormField label="Ingredient">
          <input
            role="combobox"
            aria-autocomplete="list"
            aria-expanded={open && !!query}
            aria-controls={id}
            aria-activedescendant={
              open && active >= 0 ? `${id}-${active}` : undefined
            }
            aria-describedby={`${id}-help`}
            autoComplete="off"
            maxLength={120}
            placeholder="Search or type a new ingredient"
            value={value}
            readOnly={pending}
            onFocus={() => setOpen(true)}
            onChange={(event) => {
              setText(event.target.value);
              onEdit(event.target.value);
              setOpen(true);
              setActive(-1);
              setError('');
            }}
            onKeyDown={(event) => {
              if (event.nativeEvent.isComposing) return;
              if (event.key === 'Escape') {
                event.preventDefault();
                setOpen(false);
              }
              if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
                event.preventDefault();
                setOpen(true);
                if (optionCount)
                  setActive(
                    (current) =>
                      ((current < 0 && event.key === 'ArrowUp' ? 0 : current) +
                        (event.key === 'ArrowDown' ? 1 : optionCount - 1) +
                        optionCount) %
                      optionCount,
                  );
              }
              if (event.key === 'Enter') {
                event.preventDefault();
                if (!open || !ready || pending) return;
                if (active >= 0) activate(active);
                else if (exact) choose(exact);
                else if (!results.length) void create();
              }
            }}
          />
        </FormField>
        {open && !!query && (
          <div className="pp-ingredient-popover">
            {matches.isFetching && <p role="status">Finding ingredients…</p>}
            {matches.error && (
              <p role="alert">
                Search failed: {matches.error.message}. Retry by editing the
                search.
              </p>
            )}
            {ready && (
              <div
                id={id}
                role="listbox"
                aria-label="Ingredient suggestions"
                className="space-y-1 pp-ingredient-options"
              >
                {results.map((item, index) => (
                  <button
                    type="button"
                    role="option"
                    id={`${id}-${index}`}
                    aria-selected={active === index}
                    tabIndex={-1}
                    key={item.id}
                    ref={(element) => {
                      if (active === index)
                        element?.scrollIntoView?.({ block: 'nearest' });
                    }}
                    className="block w-full text-left rounded p-2 hover:bg-stone-100 focus:bg-stone-100"
                    onMouseDown={(event) => event.preventDefault()}
                    onClick={() => choose(item)}
                  >
                    {item.name}
                  </button>
                ))}
                {canCreate && (
                  <button
                    type="button"
                    role="option"
                    id={`${id}-${results.length}`}
                    aria-selected={active === results.length}
                    tabIndex={-1}
                    ref={(element) => {
                      if (active === results.length)
                        element?.scrollIntoView?.({ block: 'nearest' });
                    }}
                    className="block w-full text-left rounded p-2 hover:bg-stone-100 focus:bg-stone-100"
                    disabled={pending}
                    onMouseDown={(event) => event.preventDefault()}
                    onClick={() => void create()}
                  >
                    Add “{query}” as a new ingredient
                  </button>
                )}
              </div>
            )}
            {ready && results.length > 0 && !exact && (
              <p className="pp-muted text-xs">
                Possible matches found. Choose one, or add a different
                ingredient.
              </p>
            )}
          </div>
        )}
      </div>
      <p id={`${id}-help`} className="pp-muted text-xs">
        Choose a suggestion, or press Enter to add when no matches exist. New
        ingredients are saved immediately.
      </p>
      {pending && <p role="status">Adding ingredient…</p>}
      {error && <p role="alert">{error} Your wording is still here.</p>}
    </div>
  );
}
