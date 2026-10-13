import { useDraftGuard } from '../../lib/drafts';
import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import {
  listConversionRules,
  saveConversionRule,
  retireConversionRule,
} from '../../lib/ipc';
import type { ConversionRule, RuleInput, Unit } from '../../lib/models';

function RuleEditor({
  ingredientId,
  rule,
  units,
  onClose,
}: {
  ingredientId: string;
  rule: ConversionRule | null;
  units: Unit[];
  onClose: () => void;
}) {
  const [input, setInput] = useState<RuleInput>({
    ingredientId,
    fromUnitId: rule?.fromUnitId ?? 'piece',
    fromAmount: rule
      ? rule.originalFromText || `${rule.fromNumerator}/${rule.fromDenominator}`
      : '1',
    toUnitId: rule?.toUnitId ?? 'g',
    toAmount: rule
      ? rule.originalToText || `${rule.toNumerator}/${rule.toDenominator}`
      : '',
    approximate: rule?.approximate ?? true,
    notes: rule?.notes ?? '',
  });
  const [baseline] = useState(() => JSON.stringify(input));
  const [confirmed, setConfirmed] = useState(false);
  const client = useQueryClient();
  const done = async () => {
    await client.invalidateQueries({
      queryKey: ['conversion-rules', ingredientId],
    });
    onClose();
  };
  const save = useMutation({
    mutationFn: () => saveConversionRule(rule?.id ?? null, input),
    onSuccess: done,
  });
  const retire = useMutation({
    mutationFn: () => retireConversionRule(rule!.id, true),
    onSuccess: done,
  });
  const pending = save.isPending || retire.isPending;
  const canDiscard = useDraftGuard(JSON.stringify(input) !== baseline, pending);
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        save.mutate();
      }}
      className="my-4 space-y-3 rounded-lg border border-stone-200 p-4"
    >
      <h4 className="font-semibold">
        {rule ? 'Edit' : 'New'} ingredient conversion rule
      </h4>
      <fieldset disabled={pending} className="flex flex-wrap gap-3">
        {(['from', 'to'] as const).map((side) => (
          <div key={side} className="flex flex-wrap gap-2">
            <FormField
              label={`${side === 'from' ? 'Rule source' : 'Rule equivalent'} amount`}
            >
              <input
                required
                maxLength={120}
                className="w-28"
                value={input[`${side}Amount`]}
                onChange={(e) =>
                  setInput({ ...input, [`${side}Amount`]: e.target.value })
                }
              />
            </FormField>
            <FormField
              label={`${side === 'from' ? 'Rule source' : 'Rule equivalent'} unit`}
            >
              <select
                value={input[`${side}UnitId`]}
                onChange={(e) =>
                  setInput({ ...input, [`${side}UnitId`]: e.target.value })
                }
              >
                {units
                  .filter(
                    (u) =>
                      u.dimension !== 'temperature' &&
                      u.dimension !== 'informal',
                  )
                  .map((u) => (
                    <option key={u.id} value={u.id}>
                      {u.label}
                    </option>
                  ))}
              </select>
            </FormField>
          </div>
        ))}
      </fieldset>
      <label className="flex items-center gap-2">
        <input
          type="checkbox"
          checked={input.approximate}
          disabled={pending}
          onChange={(e) =>
            setInput({ ...input, approximate: e.target.checked })
          }
        />
        Approximate conversion
      </label>
      <FormField label="Rule notes / package size">
        <textarea
          required
          rows={2}
          maxLength={10000}
          disabled={pending}
          value={input.notes}
          onChange={(e) => setInput({ ...input, notes: e.target.value })}
        />
      </FormField>
      <p className="text-sm text-stone-600">
        Describe the applicable size, preparation or density. The rule applies
        only to this ingredient, in both directions. Keep one applicable rule
        per conversion.
      </p>
      <div className="flex flex-wrap gap-3">
        <Button type="submit" disabled={pending}>
          Save conversion rule
        </Button>
        <Button
          disabled={pending}
          onClick={() => {
            if (canDiscard()) onClose();
          }}
        >
          Cancel rule
        </Button>
      </div>
      {rule && (
        <>
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={confirmed}
              onChange={(e) => setConfirmed(e.target.checked)}
            />
            Confirm retirement of this rule
          </label>
          <Button
            variant="danger"
            disabled={!confirmed || pending}
            onClick={() => retire.mutate()}
          >
            Retire conversion rule
          </Button>
        </>
      )}
      {(save.isError || retire.isError) && (
        <p role="alert">{save.error?.message ?? retire.error?.message}</p>
      )}
    </form>
  );
}
export function ConversionRules({
  ingredientId,
  units,
}: {
  ingredientId: string;
  units: Unit[];
}) {
  const rules = useQuery({
    queryKey: ['conversion-rules', ingredientId],
    queryFn: () => listConversionRules(ingredientId),
  });
  const [editing, setEditing] = useState<ConversionRule | null | undefined>(
    undefined,
  );
  return (
    <details className="mt-5">
      <summary className="cursor-pointer font-medium">
        Ingredient conversion rules
      </summary>
      <p className="my-3 text-sm">
        Configure piece weights, ingredient densities or package sizes when
        known. No conversion is assumed.
      </p>
      {rules.isError ? (
        <p role="alert">Conversion rules could not be opened.</p>
      ) : rules.isPending ? (
        <p role="status">Loading rules…</p>
      ) : (
        <>
          {rules.data.length === 0 && <p>No ingredient-specific rules.</p>}
          <ul className="my-3 space-y-2">
            {rules.data.map((rule) => (
              <li key={rule.id} className="flex flex-wrap items-center gap-3">
                <span>
                  {rule.originalFromText ||
                    `${rule.fromNumerator}/${rule.fromDenominator}`}{' '}
                  {units.find((u) => u.id === rule.fromUnitId)?.symbol}{' '}
                  {rule.approximate ? '≈' : '='}{' '}
                  {rule.originalToText ||
                    `${rule.toNumerator}/${rule.toDenominator}`}{' '}
                  {units.find((u) => u.id === rule.toUnitId)?.symbol} —{' '}
                  {rule.notes}
                </span>
                <Button
                  aria-label={`Edit conversion rule ${rule.originalFromText || 'amount'} ${units.find((u) => u.id === rule.fromUnitId)?.symbol ?? ''}`}
                  variant="quiet"
                  disabled={editing !== undefined}
                  onClick={() => {
                    setEditing(rule);
                  }}
                >
                  Edit rule
                </Button>
              </li>
            ))}
          </ul>
          {editing === undefined ? (
            <Button onClick={() => setEditing(null)}>
              Add conversion rule
            </Button>
          ) : (
            <RuleEditor
              key={editing?.id ?? 'new'}
              ingredientId={ingredientId}
              rule={editing}
              units={units}
              onClose={() => setEditing(undefined)}
            />
          )}
        </>
      )}
    </details>
  );
}
