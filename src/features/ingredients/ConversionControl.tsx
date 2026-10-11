import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { convertMeasurement, getMeasurementPreferences } from '../../lib/ipc';
import type { Unit, ConversionRequest } from '../../lib/models';

export function ConversionControl({
  units,
  ingredientId = null,
  temperature = false,
}: {
  units: Unit[];
  ingredientId?: string | null;
  temperature?: boolean;
}) {
  const options = units.filter((u) =>
    temperature ? u.dimension === 'temperature' : u.dimension !== 'temperature',
  );
  const [input, setInput] = useState<ConversionRequest>({
    amount: '1',
    fromUnit: temperature ? 'celsius' : 'g',
    toUnit: temperature ? 'fahrenheit' : 'kg',
    ingredientId,
    purpose: temperature ? 'temperature' : 'quantity',
  });
  const preferences = useQuery({
    queryKey: ['measurement-preferences'],
    queryFn: getMeasurementPreferences,
  });
  const conversion = useMutation({
    mutationFn: (input: ConversionRequest) => convertMeasurement(input),
  });
  const result = conversion.data;
  const change = (key: 'amount' | 'fromUnit' | 'toUnit', value: string) => {
    setInput({ ...input, [key]: value });
    conversion.reset();
  };
  return (
    <section
      aria-label={
        temperature ? 'Temperature conversion' : 'Quantity conversion'
      }
      className="mt-5 rounded-lg border border-stone-200 p-4"
    >
      <h3 className="mb-2 font-semibold">
        {temperature ? 'Temperature display' : 'Convert an amount'}
      </h3>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          conversion.mutate(input);
        }}
        className="flex flex-wrap items-end gap-3"
      >
        <FormField label="Amount to convert">
          <input
            required
            maxLength={120}
            value={input.amount}
            onChange={(e) => change('amount', e.target.value)}
            className="w-32"
          />
        </FormField>
        {(['fromUnit', 'toUnit'] as const).map((key) => (
          <FormField
            key={key}
            label={key === 'fromUnit' ? 'From unit' : 'To unit'}
          >
            <select
              value={input[key]}
              onChange={(e) => change(key, e.target.value)}
            >
              {!temperature && (
                <>
                  <option value="cup">cup (unspecified)</option>
                  <option value="tsp">tsp (unspecified)</option>
                  <option value="tbsp">tbsp (unspecified)</option>
                </>
              )}
              {options.map((unit) => (
                <option key={unit.id} value={unit.id}>
                  {unit.label}
                </option>
              ))}
            </select>
          </FormField>
        ))}
        <Button type="submit" disabled={conversion.isPending}>
          Convert
        </Button>
      </form>
      {conversion.isError && (
        <p role="alert" className="mt-3">
          {conversion.error.message}
        </p>
      )}
      {conversion.data?.status === 'converted' && (
        <p role="status" className="mt-3">
          {conversion.data.approximate ? 'Approximate: ' : ''}
          {conversion.data.display}{' '}
          {
            units.find(
              (u) => result?.status === 'converted' && u.id === result.toUnitId,
            )?.symbol
          }
          {conversion.data.rounded && (
            <> (rounded; exact {conversion.data.exact})</>
          )}
          {conversion.data.approximate && <> — uses your ingredient rule</>}
        </p>
      )}
      {conversion.data?.status === 'cannot_convert' && (
        <div className="mt-3">
          <p role="status">Cannot convert: {conversion.data.reason}</p>
          {conversion.data.candidates
            .filter((id) => units.some((u) => u.id === id))
            .map((id) => (
              <Button
                key={id}
                className="mr-2 mt-2"
                onClick={() =>
                  change(
                    options.some((u) => u.id === input.fromUnit)
                      ? 'toUnit'
                      : 'fromUnit',
                    id,
                  )
                }
              >
                Use {units.find((u) => u.id === id)?.label}
              </Button>
            ))}
          {input.fromUnit === 'cup' && preferences.data?.cupConvention && (
            <p className="mt-2 text-sm">
              Your review preference is {preferences.data.cupConvention}. Choose
              that cup explicitly if it matches the source.
            </p>
          )}
        </div>
      )}
      <p className="mt-2 text-sm text-stone-600">
        This is a display conversion. Original quantities stay intact.
      </p>
    </section>
  );
}
