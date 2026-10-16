import { useDraftGuard } from '../../lib/drafts';
import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import {
  getCatalog,
  getMeasurementPreferences,
  saveMeasurementPreferences,
} from '../../lib/ipc';
import type { MeasurementPreferences, Unit } from '../../lib/models';
import { ConversionControl } from '../ingredients/ConversionControl';

function PreferenceForm({
  initial,
  units,
}: {
  initial: MeasurementPreferences;
  units: Unit[];
}) {
  const cups = ['metric', 'us', 'imperial']
    .map((system) => units.find((unit) => unit.id === `cup-${system}`))
    .filter((unit): unit is Unit => !!unit);
  const cupLabel = (unit: Unit) =>
    unit.factorNumerator !== null && unit.factorDenominator
      ? `${unit.label}: ${(unit.factorNumerator / unit.factorDenominator).toLocaleString('en-CA', { maximumFractionDigits: 3 })} mL`
      : unit.label;
  const [input, setInput] = useState(initial);
  const client = useQueryClient();
  const save = useMutation({
    mutationFn: (input: MeasurementPreferences) =>
      saveMeasurementPreferences(input),
    onSuccess: async (data) => {
      client.setQueryData(['measurement-preferences'], data);
    },
  });
  useDraftGuard(
    JSON.stringify(input) !== JSON.stringify(initial),
    save.isPending,
  );
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        save.mutate(input);
      }}
      className="space-y-4"
    >
      <fieldset disabled={save.isPending} className="space-y-4">
        <legend className="sr-only">Measurement display preferences</legend>
        <div className="grid gap-4 sm:grid-cols-2">
          <FormField label="Measurement mode">
            <select
              value={input.mode}
              onChange={(e) => {
                setInput({
                  ...input,
                  mode: e.target.value as MeasurementPreferences['mode'],
                });
                save.reset();
              }}
            >
              <option value="mixed">Mixed / ingredient-specific</option>
              <option value="metric">Metric</option>
              <option value="us">US customary</option>
            </select>
          </FormField>
          <FormField label="Decimal places">
            <input
              type="number"
              min={0}
              max={6}
              required
              value={input.decimalPlaces}
              onChange={(e) => {
                setInput({ ...input, decimalPlaces: Number(e.target.value) });
                save.reset();
              }}
            />
          </FormField>
        </div>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={input.fractions}
            onChange={(e) => {
              setInput({ ...input, fractions: e.target.checked });
              save.reset();
            }}
          />
          Display exact fractions
        </label>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={input.aggregateCompatible}
            onChange={(e) => {
              setInput({ ...input, aggregateCompatible: e.target.checked });
              save.reset();
            }}
          />
          Combine compatible shopping quantities
        </label>
        <p className="text-sm text-stone-600">
          Imports use US cups and spoons in mixed/US mode and metric variants in
          metric mode. Explicit source units and original wording are preserved.
        </p>
        <div className="pp-panel text-sm">
          <h3 className="font-semibold mb-2">Cup reference</h3>
          <ul>
            {cups.map((cup) => (
              <li key={cup.id}>{cupLabel(cup)}</li>
            ))}
          </ul>
          <p className="pp-muted">Review imported units before saving.</p>
        </div>
        <Button type="submit" disabled={save.isPending}>
          Save measurement preferences
        </Button>
      </fieldset>
      {save.isError && <p role="alert">{save.error.message}</p>}
      {save.isSuccess && <p role="status">Measurement preferences saved.</p>}
    </form>
  );
}
export function MeasurementSettings() {
  const prefs = useQuery({
    queryKey: ['measurement-preferences'],
    queryFn: getMeasurementPreferences,
  });
  const catalog = useQuery({ queryKey: ['catalog'], queryFn: getCatalog });
  return (
    <section
      aria-label="Measurements"
      className="mb-6 rounded-xl border border-stone-200 bg-white p-6"
    >
      <h2 className="mb-4 text-xl font-semibold">Measurements</h2>
      {prefs.isError ? (
        <p role="alert">
          Measurement preferences could not be opened. Retry after restarting
          PantryPal.
        </p>
      ) : prefs.data ? (
        <PreferenceForm
          initial={prefs.data}
          units={catalog.data?.units ?? []}
        />
      ) : (
        <p role="status">Loading measurement preferences…</p>
      )}
      <details className="mt-5">
        <summary className="font-semibold">Temperature conversion</summary>
        {catalog.data && (
          <ConversionControl units={catalog.data.units} temperature />
        )}
      </details>
    </section>
  );
}
