import { useDraftGuard } from '../../lib/drafts';
import { useState } from 'react';
import type { PlanEntry, PlanOperation } from '../../lib/models';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';
import { dateLabel, localToday } from '../../lib/local-date';

export function DayChooser({
  days,
  value,
  onChange,
  label = 'Plan date',
}: {
  days: string[];
  value: string | null;
  onChange: (date: string | null) => void;
  label?: string;
}) {
  return (
    <FormField label={label}>
      <select
        value={value ?? ''}
        onChange={(e) => onChange(e.target.value || null)}
      >
        <option value="">This week (unscheduled)</option>
        {days.map((day) => (
          <option key={day} value={day}>
            {dateLabel(day)}
          </option>
        ))}
      </select>
    </FormField>
  );
}
export function EditPlanForm({
  entry,
  days,
  onApply,
  onCancel,
  servingsOnly = false,
}: {
  entry: PlanEntry;
  days: string[];
  onApply: (op: PlanOperation) => void;
  onCancel: () => void;
  servingsOnly?: boolean;
}) {
  const [date, setDate] = useState(entry.localDate);
  const [servings, setServings] = useState(entry.servingsText);
  const canDiscard = useDraftGuard(
    date !== entry.localDate || servings !== entry.servingsText,
  );
  return (
    <form
      className="space-y-3"
      onSubmit={(e) => {
        e.preventDefault();
        onApply({ kind: 'edit', entryId: entry.id, localDate: date, servings });
      }}
    >
      {!servingsOnly && (
        <DayChooser days={days} value={date} onChange={setDate} />
      )}
      <FormField label="Planned servings">
        <input
          autoFocus
          required
          value={servings}
          onChange={(e) => setServings(e.target.value)}
        />
      </FormField>
      <p className="text-sm text-stone-600">
        These servings apply only to this planned meal. Its ingredient amounts
        scale accordingly; the recipe's base servings stay unchanged.
      </p>
      <div className="flex gap-2">
        <Button type="submit">
          {servingsOnly ? 'Save servings' : 'Save planned occurrence'}
        </Button>
        <Button
          onClick={() => {
            if (canDiscard()) onCancel();
          }}
        >
          {servingsOnly ? 'Cancel servings edit' : 'Cancel occurrence edit'}
        </Button>
      </div>
    </form>
  );
}
export function RecordCookedForm({
  entry,
  onApply,
  onCancel,
}: {
  entry: PlanEntry;
  onApply: (op: PlanOperation) => void;
  onCancel: () => void;
}) {
  const [date, setDate] = useState(localToday);
  return (
    <form
      className="space-y-3"
      onSubmit={(e) => {
        e.preventDefault();
        onApply({
          kind: 'set_status',
          entryId: entry.id,
          status: 'cooked',
          cookedDate: date,
          confirmed: false,
        });
      }}
    >
      <FormField label="Date actually cooked">
        <input
          type="date"
          required
          max={localToday()}
          value={date}
          onChange={(e) => setDate(e.target.value)}
        />
      </FormField>
      <p>Record this occurrence once, on the day you actually cooked it.</p>
      <div className="flex gap-2">
        <Button type="submit">Record cooked</Button>
        <Button onClick={onCancel}>Cancel cooking record</Button>
      </div>
    </form>
  );
}
