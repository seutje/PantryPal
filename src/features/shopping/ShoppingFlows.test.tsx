import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { ShoppingPage } from './ShoppingPage';
import {
  getShopping,
  changeShopping,
  copyShopping,
  NativeError,
} from '../../lib/ipc';
import type { ShoppingItem, ShoppingView } from '../../lib/models';
import { catalogFixture } from '../../test/catalog-fixture';
import { weekFixture } from '../../test/week-fixture';

vi.mock('../../lib/ipc', async (original) => ({
  ...(await original<typeof import('../../lib/ipc')>()),
  getWeek: vi.fn(async () => structuredClone(weekFixture)),
  getShopping: vi.fn(),
  changeShopping: vi.fn(),
  copyShopping: vi.fn(),
}));
function row(): ShoppingItem {
  return {
    id: 'broccoli',
    key: 'ingredient:broccoli:g',
    ingredientId: 'ingredient-broccoli',
    label: 'Broccoli',
    sectionId: 'section-produce',
    numerator: '500',
    denominator: '1',
    unitId: 'g',
    amountText: '',
    display: '500 g',
    rounded: false,
    section: 'Produce',
    approximate: true,
    warnings: ['Ingredient-specific approximation.'],
    ruleIds: ['density'],
    sources: [],
    adjustment: null,
    checked: false,
    excluded: false,
    orphaned: false,
    legacy: null,
  };
}
function fixture(): ShoppingView {
  return {
    weekStart: '2026-12-28',
    revision: 1,
    generated: true,
    stale: false,
    items: [row()],
    sections: catalogFixture.sections,
    units: catalogFixture.units,
    text: 'Shopping · week of 2026-12-28\r\n\r\nProduce\r\n[ ] Broccoli — 500 g (approximate)',
  };
}
function show() {
  render(
    <AppProviders>
      <ShoppingPage weekStart="2026-12-28" />
    </AppProviders>,
  );
}
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(getShopping).mockResolvedValue(fixture());
  vi.mocked(copyShopping).mockResolvedValue();
});
describe('shopping editing and copying', () => {
  it('preserves an explicit unknown override with no unit and requires confirmation to reset it', async () => {
    const user = userEvent.setup();
    const saved = fixture();
    saved.items[0]!.adjustment = { amount: 'unknown', unitId: null };
    saved.items[0]!.display = 'unknown';
    vi.mocked(getShopping).mockResolvedValueOnce(saved);
    show();
    await screen.findByRole('checkbox', { name: 'Broccoli' });
    await user.click(
      screen.getByRole('button', { name: 'Edit amount for Broccoli' }),
    );
    expect(screen.getByLabelText('Shopping amount')).toHaveValue('unknown');
    expect(screen.getByLabelText('Shopping unit')).toHaveValue('');
    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    const confirm = vi
      .spyOn(window, 'confirm')
      .mockReturnValueOnce(false)
      .mockReturnValueOnce(true);
    await user.click(
      screen.getByRole('button', { name: 'Reset amount for Broccoli' }),
    );
    expect(changeShopping).not.toHaveBeenCalled();
    vi.mocked(changeShopping).mockResolvedValueOnce(fixture());
    await user.click(
      screen.getByRole('button', { name: 'Reset amount for Broccoli' }),
    );
    expect(changeShopping).toHaveBeenCalledWith(
      expect.objectContaining({
        operation: { kind: 'reset', itemId: 'broccoli', confirmed: true },
      }),
    );
    confirm.mockRestore();
  });
  it('refreshes explicitly, preserves a checked override and copies backend text for the selected week', async () => {
    const user = userEvent.setup();
    const saved = fixture();
    saved.items[0]!.checked = true;
    vi.mocked(changeShopping).mockResolvedValueOnce(saved);
    show();
    const checkbox = await screen.findByRole('checkbox', { name: 'Broccoli' });
    await user.click(checkbox);
    expect(changeShopping).toHaveBeenLastCalledWith(
      expect.objectContaining({
        weekStart: '2026-12-28',
        expectedRevision: 1,
        operation: { kind: 'check', itemId: 'broccoli', checked: true },
      }),
    );
    saved.revision = 2;
    saved.items[0]!.adjustment = { amount: '1 1/2', unitId: 'kg' };
    saved.items[0]!.display = '1½ kg';
    vi.mocked(changeShopping).mockResolvedValueOnce(saved);
    await user.click(
      screen.getByRole('button', { name: 'Edit amount for Broccoli' }),
    );
    const form = screen.getByRole('form', { name: 'Edit Broccoli' });
    await user.clear(within(form).getByLabelText('Shopping amount'));
    await user.type(within(form).getByLabelText('Shopping amount'), '1 1/2');
    await user.selectOptions(
      within(form).getByLabelText('Shopping unit'),
      'kg',
    );
    await user.click(
      within(form).getByRole('button', { name: 'Save shopping item' }),
    );
    expect(changeShopping).toHaveBeenLastCalledWith(
      expect.objectContaining({
        operation: {
          kind: 'adjust',
          itemId: 'broccoli',
          amount: '1 1/2',
          unitId: 'kg',
        },
      }),
    );
    vi.mocked(changeShopping).mockResolvedValueOnce(saved);
    await user.click(screen.getByRole('button', { name: 'Refresh list' }));
    expect(await screen.findByText('1½ kg')).toBeVisible();
    expect(screen.getByRole('checkbox', { name: 'Broccoli' })).toBeChecked();
    expect(screen.getByText('Approximate')).toBeVisible();
    await user.click(screen.getByRole('button', { name: 'Copy list' }));
    expect(
      await screen.findByText('Copied. Paste into your notes or message.'),
    ).toBeVisible();
    expect(copyShopping).toHaveBeenCalledWith('2026-12-28', 2);
  });
  it('adds a manual unknown item using the keyboard and shows exclusions and retained orphan adjustments', async () => {
    const user = userEvent.setup();
    show();
    await screen.findByRole('checkbox', { name: 'Broccoli' });
    screen.getByRole('button', { name: 'Add manual item' }).focus();
    await user.keyboard('{Enter}');
    const form = screen.getByRole('form', { name: 'Manual shopping item' });
    await user.type(within(form).getByLabelText('Item name'), 'Soap');
    await user.type(within(form).getByLabelText('Shopping amount'), 'unknown');
    const manual = {
      ...row(),
      id: 'soap',
      key: null,
      ingredientId: null,
      label: 'Soap',
      section: 'Other',
      display: 'Unknown amount',
      numerator: null,
      denominator: null,
      unitId: null,
      adjustment: { amount: 'unknown', unitId: null },
      approximate: false,
    };
    const saved = fixture();
    saved.items.push(manual);
    vi.mocked(changeShopping).mockResolvedValueOnce(saved);
    await user.click(
      within(form).getByRole('button', { name: 'Save shopping item' }),
    );
    expect(changeShopping).toHaveBeenLastCalledWith(
      expect.objectContaining({
        operation: expect.objectContaining({
          kind: 'manual',
          itemId: expect.any(String),
          label: 'Soap',
          amount: 'unknown',
          unitId: null,
          sectionId: null,
        }),
      }),
    );
    expect(await screen.findByText('Manual item')).toBeVisible();
    const orphan = fixture();
    orphan.items[0]!.excluded = true;
    orphan.items[0]!.orphaned = true;
    orphan.items[0]!.adjustment = { amount: '2', unitId: 'g' };
    vi.mocked(changeShopping).mockResolvedValueOnce(orphan);
    await user.click(screen.getByRole('button', { name: 'Exclude Broccoli' }));
    expect(await screen.findByText('Excluded from copy')).toBeVisible();
    expect(screen.getByText(/Source no longer present/)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Copy list' })).toBeDisabled();
  });
  it('blocks stale copying until refresh and retains failed input for correction or exact retry', async () => {
    const user = userEvent.setup();
    const stale = fixture();
    stale.stale = true;
    vi.mocked(getShopping).mockResolvedValue(stale);
    show();
    expect(
      await screen.findByText(/Shopping list needs refresh/),
    ).toBeVisible();
    expect(screen.getByRole('button', { name: 'Copy list' })).toBeDisabled();
    vi.mocked(changeShopping).mockRejectedValueOnce(
      new NativeError({
        code: 'storage_unavailable',
        message: 'Local data unavailable.',
      }),
    );
    await user.click(screen.getByRole('button', { name: 'Refresh list' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Local data unavailable.',
    );
    const action = vi.mocked(changeShopping).mock.calls[0]![0];
    vi.mocked(changeShopping).mockResolvedValueOnce(fixture());
    await user.click(screen.getByRole('button', { name: 'Retry action' }));
    expect(changeShopping).toHaveBeenLastCalledWith(action);
    expect(screen.getByRole('button', { name: 'Copy list' })).toBeEnabled();
    await user.click(
      screen.getByRole('button', { name: 'Edit amount for Broccoli' }),
    );
    const amount = screen.getByLabelText('Shopping amount');
    await user.clear(amount);
    await user.type(amount, '-1');
    vi.mocked(changeShopping).mockRejectedValueOnce(
      new NativeError({
        code: 'validation',
        message: 'Shopping amounts cannot be negative.',
      }),
    );
    await user.click(
      screen.getByRole('button', { name: 'Save shopping item' }),
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Shopping amounts cannot be negative.',
    );
    expect(amount).toHaveValue('-1');
    expect(amount).toBeEnabled();
    await user.clear(amount);
    await user.type(amount, '2');
    vi.mocked(changeShopping).mockResolvedValueOnce(fixture());
    await user.click(
      screen.getByRole('button', { name: 'Save shopping item' }),
    );
    expect(changeShopping).toHaveBeenLastCalledWith(
      expect.objectContaining({
        operation: {
          kind: 'adjust',
          itemId: 'broccoli',
          amount: '2',
          unitId: 'g',
        },
      }),
    );
  });
  it('offers helpful empty guidance and reports clipboard failure without changing list data', async () => {
    const user = userEvent.setup();
    const empty = fixture();
    empty.generated = false;
    empty.items = [];
    vi.mocked(getShopping).mockResolvedValueOnce(empty);
    show();
    expect(await screen.findByText('A list for your next shop')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Copy list' })).toBeDisabled();
    expect(changeShopping).not.toHaveBeenCalled();
    vi.mocked(changeShopping).mockResolvedValueOnce(fixture());
    await user.click(screen.getByRole('button', { name: 'Generate list' }));
    await screen.findByRole('checkbox', { name: 'Broccoli' });
    vi.mocked(copyShopping).mockRejectedValueOnce(new Error('Clipboard busy.'));
    await user.click(screen.getByRole('button', { name: 'Copy list' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Clipboard busy.',
    );
    expect(
      screen.getByRole('checkbox', { name: 'Broccoli' }),
    ).not.toBeChecked();
    expect(changeShopping).toHaveBeenCalledTimes(1);
  });
});
