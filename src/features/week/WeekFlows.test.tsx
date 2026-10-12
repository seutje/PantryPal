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
  searchMeals: vi.fn(),
  retireMeal: vi.fn(),
}));
let saved: WeekView;
const entry = (id: string): PlanEntry => ({
  id,
  mealId: 'meal-planner',
  localDate: null,
  servings: { numerator: 4, denominator: 1 },
  servingsText: '4',
  quantityDisplays: ['⅓', '500', null],
  status: 'planned',
  sortOrder: 0,
  cookedEvent: null,
});
beforeEach(() => {
  vi.clearAllMocks();
  saved = structuredClone(weekFixture);
  vi.mocked(api.searchMeals).mockResolvedValue(structuredClone(saved.library));
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
        mealId: op.mealId,
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
  it('edits only occurrence servings and shows the backend-scaled ingredients', async () => {
    const user = userEvent.setup();
    saved.entries = [entry('one'), entry('two')].map((e) => ({
      ...e,
      localDate: '2027-01-01',
    }));
    const original = structuredClone(saved.library.meals[0]!.meal);
    open();
    const region = await screen.findByRole('region', {
      name: dateLabel('2027-01-01'),
    });
    await user.click(
      within(region).getAllByRole('button', { name: 'Edit servings' })[0]!,
    );
    expect(
      within(region).queryByLabelText('Plan date'),
    ).not.toBeInTheDocument();
    expect(
      within(region).getByText(/recipe's base servings stay unchanged/),
    ).toBeVisible();
    await user.clear(within(region).getByLabelText('Planned servings'));
    await user.type(within(region).getByLabelText('Planned servings'), '2');
    const response = structuredClone(saved);
    response.revision++;
    response.entries[0] = {
      ...response.entries[0]!,
      servings: { numerator: 2, denominator: 1 },
      servingsText: '2',
      quantityDisplays: ['⅙', '250', null],
    };
    vi.mocked(api.applyPlan).mockResolvedValueOnce(response);
    await user.click(
      within(region).getByRole('button', { name: 'Save servings' }),
    );
    await waitFor(() =>
      expect(api.applyPlan).toHaveBeenCalledWith(
        expect.objectContaining({
          operation: {
            kind: 'edit',
            entryId: 'one',
            localDate: '2027-01-01',
            servings: '2',
          },
        }),
      ),
    );
    await user.click(
      await within(region).findByText('Ingredients for 2 servings', {
        selector: 'summary',
      }),
    );
    expect(within(region).getByText(/250 g/)).toBeVisible();
    expect(within(region).getByText(/⅙ US cup/)).toBeVisible();
    const ingredients = within(region)
      .getByText('Ingredients for 2 servings', { selector: 'summary' })
      .closest('details')!;
    expect(within(ingredients).getByText(/Unknown amount/)).toBeVisible();
    expect(
      within(region).getByText('Ingredients for 4 servings'),
    ).toBeInTheDocument();
    expect(response.library.meals[0]!.meal).toEqual(original);
    await user.click(
      within(region).getAllByRole('button', { name: 'Edit servings' })[0]!,
    );
    await user.clear(within(region).getByLabelText('Planned servings'));
    await user.type(within(region).getByLabelText('Planned servings'), '99');
    await user.click(
      within(region).getByRole('button', { name: 'Cancel servings edit' }),
    );
    expect(api.applyPlan).toHaveBeenCalledTimes(1);
    expect(
      within(region).getByText('Ingredients for 2 servings'),
    ).toBeInTheDocument();
  });
  it('blocks repeat selection and dismissal while adding a meal', async () => {
    const user = userEvent.setup();
    let finish!: (value: WeekView) => void;
    vi.mocked(api.applyPlan).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
    open();
    const region = await screen.findByRole('region', {
      name: dateLabel('2026-12-29'),
    });
    await user.click(within(region).getByRole('button', { name: 'Add meal' }));
    const dialog = screen.getByRole('dialog');
    const choice = await within(dialog).findByRole('button', {
      name: 'Add Broccoli and rice',
    });
    await waitFor(() => expect(choice).toBeEnabled());
    await user.click(choice);
    expect(choice).toBeDisabled();
    expect(
      within(dialog).getByRole('button', { name: 'Cancel adding meal' }),
    ).toBeDisabled();
    fireEvent(dialog, new Event('cancel', { cancelable: true }));
    await user.click(choice);
    expect(dialog).toBeInTheDocument();
    expect(api.applyPlan).toHaveBeenCalledTimes(1);
    finish(structuredClone(saved));
    await waitFor(() =>
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument(),
    );
  });
  it('opens the library filters for each day and adds separate meals with keyboard selection', async () => {
    const user = userEvent.setup();
    const other = structuredClone(saved.library.meals[0]!);
    other.meal.id = 'meal-other';
    other.meal.title = 'Soup';
    other.meal.baseServingsText = '1 ½';
    other.meal.baseServings = { numerator: 3, denominator: 2 };
    saved.library.meals.push(other);
    vi.mocked(api.searchMeals).mockImplementation(async (search) => ({
      ...structuredClone(saved.library),
      meals: saved.library.meals.filter((m) =>
        m.meal.title.toLowerCase().includes(search.text.toLowerCase()),
      ),
    }));
    open();
    await screen.findByRole('button', { name: 'Add meal to week' });
    for (const day of saved.days) {
      expect(
        within(screen.getByRole('region', { name: dateLabel(day) })).getByRole(
          'button',
          { name: 'Add meal' },
        ),
      ).toBeEnabled();
    }
    const day = saved.days[0]!;
    const region = screen.getByRole('region', { name: dateLabel(day) });
    const opener = within(region).getByRole('button', { name: 'Add meal' });
    await user.click(opener);
    let dialog = await screen.findByRole('dialog', {
      name: `Add meal to ${dateLabel(day)}`,
    });
    expect(within(dialog).getByLabelText('Search meals')).toHaveFocus();
    expect(
      within(dialog).queryByRole('button', { name: 'New meal' }),
    ).not.toBeInTheDocument();
    await user.click(
      within(dialog).getByText('Filters', { selector: 'summary' }),
    );
    await user.selectOptions(
      within(dialog).getByLabelText('Contains ingredient'),
      'ingredient-onion',
    );
    await user.type(within(dialog).getByLabelText('Tag'), 'quick');
    await user.selectOptions(
      within(dialog).getByLabelText('Cooked history'),
      'never',
    );
    await waitFor(() =>
      expect(api.searchMeals).toHaveBeenLastCalledWith(
        expect.objectContaining({
          ingredientId: 'ingredient-onion',
          tag: 'quick',
          cooked: 'never',
        }),
      ),
    );
    await user.click(
      within(dialog).getByRole('button', { name: 'Clear filters' }),
    );
    await waitFor(() =>
      expect(
        within(dialog).getByRole('button', { name: 'Add Broccoli and rice' }),
      ).toBeEnabled(),
    );
    await user.click(
      within(dialog).getByRole('button', { name: 'Add Broccoli and rice' }),
    );
    await waitFor(() =>
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument(),
    );
    expect(opener).toHaveFocus();
    expect(api.applyPlan).toHaveBeenLastCalledWith(
      expect.objectContaining({
        expectedRevision: 0,
        operation: expect.objectContaining({
          kind: 'add',
          mealId: 'meal-planner',
          localDate: day,
          servings: '4',
        }),
      }),
    );
    await user.click(opener);
    dialog = await screen.findByRole('dialog');
    await user.type(within(dialog).getByLabelText('Search meals'), 'Soup');
    await waitFor(() =>
      expect(
        within(dialog).getByRole('button', { name: 'Add Soup' }),
      ).toBeEnabled(),
    );
    within(dialog).getByRole('button', { name: 'Add Soup' }).focus();
    await user.keyboard('{Enter}');
    await waitFor(() =>
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument(),
    );
    expect(api.applyPlan).toHaveBeenLastCalledWith(
      expect.objectContaining({
        expectedRevision: 1,
        operation: expect.objectContaining({
          mealId: 'meal-other',
          localDate: day,
          servings: '1 ½',
        }),
      }),
    );
    expect(
      within(region).getAllByRole('button', { name: 'Edit occurrence' }),
    ).toHaveLength(2);
    expect(saved.entries[0]!.id).not.toBe(saved.entries[1]!.id);
    expect(
      saved.entries.every(
        (e) => e.status === 'planned' && e.cookedEvent === null,
      ),
    ).toBe(true);
  });
  it('cancels without planning and keeps search and identical retry after a failed day add', async () => {
    const user = userEvent.setup();
    open();
    const region = await screen.findByRole('region', {
      name: dateLabel('2027-01-03'),
    });
    const opener = within(region).getByRole('button', { name: 'Add meal' });
    await user.click(opener);
    fireEvent(
      screen.getByRole('dialog'),
      new Event('cancel', { cancelable: true }),
    );
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(opener).toHaveFocus();
    expect(api.applyPlan).not.toHaveBeenCalled();
    await user.click(opener);
    let dialog = screen.getByRole('dialog');
    await user.click(
      within(dialog).getByRole('button', { name: 'Cancel adding meal' }),
    );
    expect(api.applyPlan).not.toHaveBeenCalled();
    vi.mocked(api.applyPlan).mockRejectedValueOnce(new Error('Try again.'));
    await user.click(opener);
    dialog = screen.getByRole('dialog');
    await user.type(within(dialog).getByLabelText('Search meals'), 'rice');
    await waitFor(() =>
      expect(
        within(dialog).getByRole('button', { name: 'Add Broccoli and rice' }),
      ).toBeEnabled(),
    );
    await user.click(
      within(dialog).getByRole('button', { name: 'Add Broccoli and rice' }),
    );
    expect(await within(dialog).findByRole('alert')).toHaveTextContent(
      'Try again.',
    );
    expect(within(dialog).getByLabelText('Search meals')).toHaveValue('rice');
    const first = vi.mocked(api.applyPlan).mock.calls[0]![0];
    await user.click(
      within(dialog).getByRole('button', { name: 'Retry plan action' }),
    );
    await waitFor(() =>
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument(),
    );
    expect(api.applyPlan).toHaveBeenLastCalledWith(first);
    expect(saved.entries).toHaveLength(1);
  });
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
