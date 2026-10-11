import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import {
  getCatalog,
  getMeasurementPreferences,
  saveMeasurementPreferences,
} from '../../lib/ipc';
import type { MeasurementPreferences } from '../../lib/models';
import { ConversionControl } from '../ingredients/ConversionControl';

function PreferenceForm({ initial }: { initial: MeasurementPreferences }) {
  const [input, setInput] = useState(initial);
  const client = useQueryClient();
  const save = useMutation({
    mutationFn: (input: MeasurementPreferences) =>
      saveMeasurementPreferences(input),
    onSuccess: async (data) => {
      client.setQueryData(['measurement-preferences'], data);
    },
  });
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        save.mutate(input);
      }}
      className="space-y-4"
    >
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
        <FormField label="Ambiguous cup review preference">
          <select
            value={input.cupConvention ?? ''}
            onChange={(e) => {
              setInput({
                ...input,
                cupConvention: (e.target.value ||
                  null) as MeasurementPreferences['cupConvention'],
              });
              save.reset();
            }}
          >
            <option value="">Always ask</option>
            <option value="metric">Metric cup — 250 mL</option>
            <option value="us">US cup — about 236.588 mL</option>
            <option value="imperial">Imperial cup — about 284.131 mL</option>
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
        Combine compatible shopping quantities when grocery generation is
        available
      </label>
      <p className="text-sm text-stone-600">
        These are display and review preferences. An unspecified cup still
        requires an explicit choice; source units are preserved.
      </p>
      <Button type="submit" disabled={save.isPending}>
        Save measurement preferences
      </Button>
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
        <PreferenceForm initial={prefs.data} />
      ) : (
        <p role="status">Loading measurement preferences…</p>
      )}
      {catalog.data && (
        <ConversionControl units={catalog.data.units} temperature />
      )}
    </section>
  );
}
