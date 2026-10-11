import {
  render,
  screen,
  within,
  waitFor,
  fireEvent,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import * as api from '../../lib/ipc';
import type { WeekView, PlanEntry } from '../../lib/models';
import { weekFixture } from '../../test/week-fixture';
import { WeekPage } from './WeekPage';
import { dateLabel } from '../../lib/local-date';
vi.mock('../../lib/ipc', () => ({
  getWeek: vi.fn(),
  applyPlan: vi.fn(),
  getMealHistory: vi.fn(),
}));
let saved: WeekView;
const entry = (id: string): PlanEntry => ({
  id,
  mealId: 'meal-planner',
  localDate: null,
  servings: { numerator: 4, denominator: 1 },
  servingsText: '4',
  status: 'planned',
  sortOrder: 0,
  cookedEvent: null,
});
beforeEach(() => {
  vi.clearAllMocks();
  saved = structuredClone(weekFixture);
  vi.mocked(api.getWeek).mockImplementation(async () => structuredClone(saved));
  vi.mocked(api.getMealHistory).mockResolvedValue({
    mealId: 'meal-planner',
    lastCooked: null,
    events: [],
  });
  vi.mocked(api.applyPlan).mockImplementation(async (input) => {
    const op = input.operation;
    if (op.kind === 'add')
      saved.entries.push({
        ...entry(op.entryId),
        localDate: op.localDate,
        servingsText: op.servings,
      });
    if (op.kind === 'edit')
      saved.entries = saved.entries.map((e) =>
        e.id === op.entryId
          ? { ...e, localDate: op.localDate, servingsText: op.servings }
          : e,
      );
    if (op.kind === 'remove')
      saved.entries = saved.entries.filter((e) => e.id !== op.entryId);
    if (op.kind === 'reorder')
      saved.entries = op.entryIds.map((id, index) => ({
        ...saved.entries.find((e) => e.id === id)!,
        sortOrder: index,
      }));
    if (op.kind === 'set_status')
      saved.entries = saved.entries.map((e) =>
        e.id === op.entryId
          ? {
              ...e,
              status: op.status,
              cookedEvent:
                op.status === 'cooked'
                  ? {
                      id: 'event',
                      mealId: e.mealId,
                      localDate: op.cookedDate!,
                      planEntryId: e.id,
                      originPlanEntryId: e.id,
                      voided: false,
                      revision: 1,
                    }
                  : e.cookedEvent
                    ? { ...e.cookedEvent, voided: true, revision: 2 }
                    : null,
            }
          : e,
      );
    saved.revision++;
    return structuredClone(saved);
  });
});
function open(props: Parameters<typeof WeekPage>[0] = { date: '2027-01-01' }) {
  return render(
    <AppProviders>
      <WeekPage {...props} />
    </AppProviders>,
  );
}
describe('weekly planning flows', () => {
  it('plans repeated meals with individual servings, date/pool moves and keyboard ordering', async () => {
    const user = userEvent.setup();
    open();
    await user.click(
      await screen.findByRole('button', { name: 'Add meal to week' }),
    );
    await user.selectOptions(
      screen.getByLabelText('Meal to plan'),
      'meal-planner',
    );
    await user.clear(screen.getByLabelText('Planned servings'));
    await user.type(screen.getByLabelText('Planned servings'), '1 ½');
    await user.click(screen.getByRole('button', { name: 'Add planned meal' }));
    await waitFor(() =>
      expect(api.applyPlan).toHaveBeenCalledWith(
        expect.objectContaining({
          expectedRevision: 0,
          weekStart: '2026-12-28',
          operation: expect.objectContaining({
            kind: 'add',
            mealId: 'meal-planner',
            localDate: null,
            servings: '1 ½',
          }),
        }),
      ),
    );
    await user.click(
      await screen.findByRole('button', { name: 'Add meal to week' }),
    );
    await user.selectOptions(
      screen.getByLabelText('Meal to plan'),
      'meal-planner',
    );
    await user.click(screen.getByRole('button', { name: 'Add planned meal' }));
    await waitFor(() =>
      expect(
        screen.getAllByRole('button', { name: 'Edit occurrence' }),
      ).toHaveLength(2),
    );
    const ids = saved.entries.map((e) => e.id);
    expect(ids[0]).not.toBe(ids[1]);
    screen.getAllByRole('button', { name: 'Move down' })[0]!.focus();
    await user.keyboard('{Enter}');
    await waitFor(() =>
      expect(api.applyPlan).toHaveBeenLastCalledWith(
        expect.objectContaining({
          operation: {
            kind: 'reorder',
            localDate: null,
            entryIds: [ids[1], ids[0]],
          },
        }),
      ),
    );
    await user.click(
      screen.getAllByRole('button', { name: 'Edit occurrence' })[0]!,
    );
    await user.selectOptions(screen.getByLabelText('Plan date'), '2027-01-01');
    await user.click(
      screen.getByRole('button', { name: 'Save planned occurrence' }),
    );
    await waitFor(() =>
      expect(api.applyPlan).toHaveBeenLastCalledWith(
        expect.objectContaining({
          operation: expect.objectContaining({
            kind: 'edit',
            localDate: '2027-01-01',
          }),
        }),
      ),
    );
    expect(
      within(
        screen.getByRole('region', { name: dateLabel('2027-01-01') }),
      ).getByRole('heading', { name: 'Broccoli and rice' }),
    ).toBeInTheDocument();
  });
  it('records cooking deliberately, confirms undo/removal and skips without a cooking date', async () => {
    const user = userEvent.setup();
    saved.entries = [entry('one')];
    open();
    await user.click(
      await screen.findByRole('button', { name: 'Skip occurrence' }),
    );
    await waitFor(() =>
      expect(api.applyPlan).toHaveBeenCalledWith(
        expect.objectContaining({
          operation: {
            kind: 'set_status',
            entryId: 'one',
            status: 'skipped',
            cookedDate: null,
            confirmed: false,
          },
        }),
      ),
    );
    await user.click(screen.getByRole('button', { name: 'Return to planned' }));
    await user.click(
      await screen.findByRole('button', { name: 'Mark cooked' }),
    );
    expect(api.applyPlan).toHaveBeenCalledTimes(2);
    fireEvent.change(screen.getByLabelText('Date actually cooked'), {
      target: { value: '2020-12-31' },
    });
    await user.click(screen.getByRole('button', { name: 'Record cooked' }));
    await waitFor(() =>
      expect(api.applyPlan).toHaveBeenLastCalledWith(
        expect.objectContaining({
          operation: {
            kind: 'set_status',
            entryId: 'one',
            status: 'cooked',
            cookedDate: '2020-12-31',
            confirmed: false,
          },
        }),
      ),
    );
    await user.click(screen.getByRole('button', { name: 'Undo cooked' }));
    expect(api.applyPlan).toHaveBeenCalledTimes(3);
    await user.click(
      screen.getByRole('button', { name: 'Confirm cooking undo' }),
    );
    await screen.findByRole('button', { name: 'Mark cooked' });
    await user.click(screen.getByRole('button', { name: 'Remove occurrence' }));
    expect(api.applyPlan).toHaveBeenCalledTimes(4);
    await user.click(
      screen.getByRole('button', { name: 'Confirm plan removal' }),
    );
    await waitFor(() =>
      expect(
        screen.queryByRole('button', { name: 'Edit occurrence' }),
      ).not.toBeInTheDocument(),
    );
  });
  it('retries an uncertain save with the identical action and keeps the form contents', async () => {
    const user = userEvent.setup();
    vi.mocked(api.applyPlan).mockRejectedValueOnce(new Error('Try again.'));
    open();
    await user.click(
      await screen.findByRole('button', { name: 'Add meal to week' }),
    );
    await user.selectOptions(
      screen.getByLabelText('Meal to plan'),
      'meal-planner',
    );
    await user.type(screen.getByLabelText('Planned servings'), ' ½');
    await user.click(screen.getByRole('button', { name: 'Add planned meal' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'inputs are kept',
    );
    expect(screen.getByLabelText('Planned servings')).toHaveValue('4 ½');
    const first = vi.mocked(api.applyPlan).mock.calls[0]![0];
    await user.click(screen.getByRole('button', { name: 'Retry plan action' }));
    await waitFor(() => expect(api.applyPlan).toHaveBeenLastCalledWith(first));
  });
  it('navigates with backend week dates and hands off the selected week and recipe', async () => {
    const user = userEvent.setup();
    saved.entries = [entry('one')];
    const onOpenMeal = vi.fn(),
      onShopping = vi.fn(),
      onDateChange = vi.fn();
    open({ date: '2027-01-01', onOpenMeal, onShopping, onDateChange });
    await user.click(
      await screen.findByRole('button', { name: 'Open recipe' }),
    );
    expect(onOpenMeal).toHaveBeenCalledWith(
      weekFixture.library.meals[0],
      weekFixture.library,
    );
    await user.click(
      screen.getByRole('button', { name: 'Open shopping for this week' }),
    );
    expect(onShopping).toHaveBeenCalledWith('2026-12-28');
    await user.click(screen.getByRole('button', { name: 'Next week' }));
    expect(onDateChange).toHaveBeenCalledWith('2027-01-04');
  });
});
