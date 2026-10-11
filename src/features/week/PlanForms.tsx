import { useState } from 'react';
import type { MealRecord, PlanEntry, PlanOperation } from '../../lib/models';
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
export function AddPlanForm({
  meals,
  days,
  busy,
  onApply,
  onCancel,
}: {
  meals: MealRecord[];
  days: string[];
  busy: boolean;
  onApply: (op: PlanOperation) => void;
  onCancel: () => void;
}) {
  const [mealId, setMealId] = useState('');
  const [date, setDate] = useState<string | null>(null);
  const [servings, setServings] = useState('');
  const [text, setText] = useState('');
  return (
    <form
      className="rounded-lg border border-stone-300 p-4"
      onSubmit={(e) => {
        e.preventDefault();
        onApply({
          kind: 'add',
          entryId: crypto.randomUUID(),
          mealId,
          localDate: date,
          servings,
        });
      }}
    >
      <fieldset disabled={busy} className="space-y-3">
        <legend className="font-semibold">Add meal to week</legend>
        <FormField label="Find a meal to plan">
          <input
            type="search"
            value={text}
            onChange={(e) => setText(e.target.value)}
          />
        </FormField>
        <FormField label="Meal to plan">
          <select
            required
            value={mealId}
            onChange={(e) => {
              setMealId(e.target.value);
              const meal = meals.find(
                (m) => m.meal.id === e.target.value,
              )?.meal;
              setServings(
                meal?.baseServingsText ??
                  (meal
                    ? `${meal.baseServings.numerator}/${meal.baseServings.denominator}`
                    : ''),
              );
            }}
          >
            <option value="">Choose a meal</option>
            {meals
              .filter(
                (m) =>
                  !m.meal.retired &&
                  (m.meal.id === mealId ||
                    m.meal.title
                      .toLocaleLowerCase()
                      .includes(text.toLocaleLowerCase())),
              )
              .map((m) => (
                <option key={m.meal.id} value={m.meal.id}>
                  {m.meal.title}
                </option>
              ))}
          </select>
        </FormField>
        <div className="grid gap-3 md:grid-cols-2">
          <DayChooser days={days} value={date} onChange={setDate} />
          <FormField label="Planned servings">
            <input
              required
              value={servings}
              onChange={(e) => setServings(e.target.value)}
            />
          </FormField>
        </div>
        <p className="text-sm text-stone-600">
          Fractions are allowed. Adding this meal again creates a separate
          occurrence.
        </p>
        {!meals.some((record) => !record.meal.retired) && (
          <p>Create a meal in Meals before adding it to your week.</p>
        )}
        <div className="flex gap-2">
          <Button type="submit">Add planned meal</Button>
          <Button onClick={onCancel}>Cancel adding</Button>
        </div>
      </fieldset>
    </form>
  );
}
export function EditPlanForm({
  entry,
  days,
  onApply,
  onCancel,
}: {
  entry: PlanEntry;
  days: string[];
  onApply: (op: PlanOperation) => void;
  onCancel: () => void;
}) {
  const [date, setDate] = useState(entry.localDate);
  const [servings, setServings] = useState(entry.servingsText);
  return (
    <form
      className="space-y-3"
      onSubmit={(e) => {
        e.preventDefault();
        onApply({ kind: 'edit', entryId: entry.id, localDate: date, servings });
      }}
    >
      <DayChooser days={days} value={date} onChange={setDate} />
      <FormField label="Planned servings">
        <input
          required
          value={servings}
          onChange={(e) => setServings(e.target.value)}
        />
      </FormField>
      <p className="text-sm text-stone-600">
        Changing the schedule does not change the date actually cooked.
      </p>
      <div className="flex gap-2">
        <Button type="submit">Save planned occurrence</Button>
        <Button onClick={onCancel}>Cancel occurrence edit</Button>
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
