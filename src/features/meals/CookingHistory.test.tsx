import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import * as api from '../../lib/ipc';
import type { MealHistory } from '../../lib/models';
import { CookingHistory } from './CookingHistory';
vi.mock('../../lib/ipc', () => ({
  getMealHistory: vi.fn(),
  changeMealHistory: vi.fn(),
}));
let history: MealHistory;
beforeEach(() => {
  vi.clearAllMocks();
  history = { mealId: 'meal', lastCooked: null, events: [] };
  vi.mocked(api.getMealHistory).mockImplementation(async () =>
    structuredClone(history),
  );
  vi.mocked(api.changeMealHistory).mockImplementation(async (input) => {
    const op = input.operation;
    if (op.kind === 'record')
      history.events.push({
        id: op.eventId,
        mealId: 'meal',
        localDate: op.localDate,
        planEntryId: null,
        originPlanEntryId: null,
        voided: false,
        revision: 1,
      });
    else
      history.events = history.events.map((e) =>
        e.id === op.eventId
          ? {
              ...e,
              localDate: op.kind === 'correct' ? op.localDate : e.localDate,
              voided: op.kind === 'undo',
              revision: e.revision + 1,
            }
          : e,
      );
    history.lastCooked =
      history.events.find((e) => !e.voided)?.localDate ?? null;
    return structuredClone(history);
  });
});
function open() {
  render(
    <AppProviders>
      <CookingHistory mealId="meal" />
    </AppProviders>,
  );
}
describe('cooking history controls', () => {
  it('backfills a local date, explicitly corrects it, and confirms undo', async () => {
    const user = userEvent.setup();
    open();
    const date = await screen.findByLabelText('Date actually cooked');
    fireEvent.change(date, { target: { value: '2020-02-29' } });
    await user.click(
      screen.getByRole('button', { name: 'Record a past cooked meal' }),
    );
    await screen.findByText('Last eaten: 2020-02-29');
    expect(api.changeMealHistory).toHaveBeenCalledWith(
      expect.objectContaining({
        mealId: 'meal',
        operation: expect.objectContaining({
          kind: 'record',
          localDate: '2020-02-29',
        }),
      }),
    );
    await user.click(
      screen.getByRole('button', { name: 'Correct date 2020-02-29' }),
    );
    fireEvent.change(screen.getByLabelText('Correct cooked date'), {
      target: { value: '2020-03-01' },
    });
    await user.click(
      screen.getByRole('button', { name: 'Confirm date correction' }),
    );
    await screen.findByText('Last eaten: 2020-03-01');
    await user.click(
      screen.getByRole('button', { name: 'Undo record 2020-03-01' }),
    );
    expect(api.changeMealHistory).toHaveBeenCalledTimes(2);
    await user.click(
      screen.getByRole('button', { name: 'Confirm history undo' }),
    );
    expect(
      await screen.findByText('Last eaten: Never recorded'),
    ).toBeInTheDocument();
    expect(screen.getByText('2020-03-01 · cancelled')).toBeInTheDocument();
  });
  it('keeps a failed date and retries the same history action', async () => {
    const user = userEvent.setup();
    vi.mocked(api.changeMealHistory).mockRejectedValueOnce(
      new Error('This record changed.'),
    );
    open();
    fireEvent.change(await screen.findByLabelText('Date actually cooked'), {
      target: { value: '2020-01-01' },
    });
    await user.click(
      screen.getByRole('button', { name: 'Record a past cooked meal' }),
    );
    expect(await screen.findByRole('alert')).toHaveTextContent('date is kept');
    expect(screen.getByLabelText('Date actually cooked')).toHaveValue(
      '2020-01-01',
    );
    const first = vi.mocked(api.changeMealHistory).mock.calls[0]![0];
    await user.click(
      screen.getByRole('button', { name: 'Retry history action' }),
    );
    await waitFor(() =>
      expect(api.changeMealHistory).toHaveBeenLastCalledWith(first),
    );
  });
});
