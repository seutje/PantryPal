import {
  fireEvent,
  render,
  screen,
  within,
  waitFor,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import * as api from '../../lib/ipc';
import { catalogFixture, onion, preferences } from '../../test/catalog-fixture';
import type { Catalog } from '../../lib/models';
import { IngredientsPage } from './IngredientsPage';

vi.mock('../../lib/ipc', () => ({
  nativeAvailable: () => true,
  getCatalog: vi.fn(),
  getMeasurementPreferences: vi.fn(),
  ingredientCandidates: vi.fn(),
  createIngredient: vi.fn(),
  updateIngredient: vi.fn(),
  retireIngredient: vi.fn(),
  listConversionRules: vi.fn(),
  saveConversionRule: vi.fn(),
  retireConversionRule: vi.fn(),
  createVocabulary: vi.fn(),
  updateVocabulary: vi.fn(),
  retireVocabulary: vi.fn(),
  convertMeasurement: vi.fn(),
}));
let saved: Catalog;
beforeEach(() => {
  vi.clearAllMocks();
  saved = structuredClone(catalogFixture);
  vi.mocked(api.getCatalog).mockImplementation(async () =>
    structuredClone(saved),
  );
  vi.mocked(api.getMeasurementPreferences).mockResolvedValue(preferences);
  vi.mocked(api.ingredientCandidates).mockResolvedValue([]);
  vi.mocked(api.listConversionRules).mockResolvedValue([]);
});
function open() {
  return render(
    <AppProviders>
      <IngredientsPage />
    </AppProviders>,
  );
}
describe('ingredient editing workflows', () => {
  it('opens a modal and returns focus to Add ingredient on Escape without saving', async () => {
    const user = userEvent.setup();
    open();
    const add = await screen.findByRole('button', { name: 'Add ingredient' });
    await user.click(add);
    const dialog = screen.getByRole('dialog', { name: 'Add ingredient' });
    expect(dialog).toHaveAttribute('open');
    expect(within(dialog).getByLabelText('Ingredient name')).toHaveFocus();
    fireEvent(dialog, new Event('cancel', { cancelable: true }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(add).toHaveFocus();
    expect(api.createIngredient).not.toHaveBeenCalled();
  });
  it('creates, searches by alias, edits and retires with explicit confirmation', async () => {
    const user = userEvent.setup();
    vi.mocked(api.createIngredient).mockImplementation(async (input) => {
      const ingredient = {
        ...input,
        id: 'herb',
        retired: false,
        createdAt: '',
        updatedAt: '',
      };
      saved.ingredients.push(ingredient);
      return ingredient;
    });
    vi.mocked(api.updateIngredient).mockImplementation(async (id, input) => {
      const updated = {
        ...input,
        id,
        retired: false,
        createdAt: '',
        updatedAt: '',
      };
      saved.ingredients = saved.ingredients.map((i) =>
        i.id === id ? updated : i,
      );
      return updated;
    });
    vi.mocked(api.retireIngredient).mockImplementation(async (id) => {
      saved.ingredients = saved.ingredients.filter((i) => i.id !== id);
    });
    open();
    await screen.findByRole('button', { name: 'Edit Onion' });
    screen.getByRole('button', { name: 'Add ingredient' }).focus();
    await user.keyboard('{Enter}');
    expect(screen.getByLabelText('Ingredient name')).toHaveFocus();
    await user.type(screen.getByLabelText('Ingredient name'), 'Household herb');
    await user.type(
      screen.getByLabelText('Aliases (one per line)'),
      'Our leaves',
    );
    await user.selectOptions(
      screen.getByLabelText('Category'),
      'category-vegetable',
    );
    await user.selectOptions(
      screen.getByLabelText('Shopping section'),
      'section-produce',
    );
    await user.selectOptions(
      screen.getByLabelText('Preferred shopping unit'),
      'g',
    );
    await user.click(screen.getByLabelText('Year-round'));
    await user.type(screen.getByLabelText('Ingredient notes'), 'Dry gently');
    await user.click(screen.getByRole('button', { name: 'Save ingredient' }));
    await screen.findByRole('button', { name: 'Edit Household herb' });
    expect(api.createIngredient).toHaveBeenCalledWith({
      name: 'Household herb',
      aliases: ['Our leaves'],
      categoryId: 'category-vegetable',
      sectionId: 'section-produce',
      preferredUnitId: 'g',
      seasonMonths: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
      notes: 'Dry gently',
    });
    expect(
      screen.getByRole('button', { name: 'Add ingredient' }),
    ).toHaveFocus();
    await user.type(
      screen.getByLabelText('Search ingredients or aliases'),
      'Our leaves',
    );
    expect(
      screen.queryByRole('button', { name: 'Edit Onion' }),
    ).not.toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: 'Edit Household herb' }),
    );
    await user.clear(screen.getByLabelText('Ingredient notes'));
    await user.type(
      screen.getByLabelText('Ingredient notes'),
      'Store airtight',
    );
    await user.click(screen.getByRole('button', { name: 'Save ingredient' }));
    await waitFor(() =>
      expect(
        screen.queryByRole('region', { name: 'Edit ingredient' }),
      ).not.toBeInTheDocument(),
    );
    await user.click(
      screen.getByRole('button', { name: 'Edit Household herb' }),
    );
    expect(screen.getByLabelText('Ingredient notes')).toHaveValue(
      'Store airtight',
    );
    await user.click(
      screen.getByText('Retire ingredient', { selector: 'summary' }),
    );
    expect(
      screen.getByRole('button', { name: 'Retire Household herb' }),
    ).toBeDisabled();
    await user.click(
      screen.getByLabelText('Confirm retirement of Household herb'),
    );
    await user.click(
      screen.getByRole('button', { name: 'Retire Household herb' }),
    );
    await waitFor(() =>
      expect(
        screen.queryByRole('button', { name: 'Edit Household herb' }),
      ).not.toBeInTheDocument(),
    );
    expect(api.retireIngredient).toHaveBeenCalledWith('herb', true);
  });
  it('shows canonical duplicate candidates and leaves inputs intact on a failed save', async () => {
    const user = userEvent.setup();
    vi.mocked(api.ingredientCandidates).mockResolvedValue([onion]);
    open();
    await screen.findByRole('button', { name: 'Edit Onion' });
    await user.click(screen.getByRole('button', { name: 'Add ingredient' }));
    await user.type(screen.getByLabelText('Ingredient name'), 'Onions');
    await user.click(screen.getByRole('button', { name: 'Save ingredient' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'already belongs',
    );
    expect(api.createIngredient).not.toHaveBeenCalled();
    await user.click(
      screen.getByRole('button', { name: 'Open existing Onion' }),
    );
    expect(screen.getByLabelText('Ingredient name')).toHaveValue('Onion');
    vi.mocked(api.ingredientCandidates).mockResolvedValue([]);
    vi.mocked(api.updateIngredient).mockRejectedValue(
      new Error('This name is already reserved.'),
    );
    await user.clear(screen.getByLabelText('Ingredient notes'));
    await user.type(
      screen.getByLabelText('Ingredient notes'),
      'Keep this draft',
    );
    await user.click(screen.getByRole('button', { name: 'Save ingredient' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'already reserved',
    );
    expect(screen.getByLabelText('Ingredient notes')).toHaveValue(
      'Keep this draft',
    );
  });
  it('creates and updates independent categories and section order', async () => {
    const user = userEvent.setup();
    vi.mocked(api.createVocabulary).mockImplementation(
      async (kind, name, sortOrder) => {
        const v = { id: 'custom', name, sortOrder, retired: false };
        (kind === 'category' ? saved.categories : saved.sections).push(v);
        return v;
      },
    );
    vi.mocked(api.updateVocabulary).mockImplementation(
      async (kind, id, name, sortOrder) => {
        const v = { id, name, sortOrder, retired: false };
        if (kind === 'section')
          saved.sections = saved.sections.map((s) => (s.id === id ? v : s));
        return v;
      },
    );
    open();
    await screen.findByRole('button', { name: 'Edit Onion' });
    await user.click(
      screen.getByText('Manage categories', { selector: 'summary' }),
    );
    await user.click(screen.getByRole('button', { name: 'Add category' }));
    await user.type(
      screen.getByLabelText('Category name'),
      'Household favourites',
    );
    await user.click(screen.getByRole('button', { name: 'Save category' }));
    await screen.findByRole('button', { name: 'Edit Household favourites' });
    expect(api.createVocabulary).toHaveBeenCalledWith(
      'category',
      'Household favourites',
      0,
    );
    await user.click(
      screen.getByText('Manage shopping sections', { selector: 'summary' }),
    );
    await user.click(screen.getByRole('button', { name: 'Edit Produce' }));
    await user.clear(screen.getByLabelText('Shopping section order'));
    await user.type(screen.getByLabelText('Shopping section order'), '2');
    await user.click(screen.getByRole('button', { name: 'Save section' }));
    await waitFor(() =>
      expect(api.updateVocabulary).toHaveBeenCalledWith(
        'section',
        'section-produce',
        'Produce',
        2,
      ),
    );
    expect(screen.getByRole('cell', { name: 'Produce' })).toBeInTheDocument();
  });
  it('saves and retires an approximate ingredient rule with its notes', async () => {
    const user = userEvent.setup();
    const rule = {
      id: 'rule-one',
      ingredientId: onion.id,
      fromUnitId: 'piece',
      fromNumerator: 1,
      fromDenominator: 1,
      toUnitId: 'g',
      toNumerator: 150,
      toDenominator: 1,
      approximate: true,
      notes: 'Medium onion',
      originalFromText: '1',
      originalToText: '150',
      retired: false,
    };
    vi.mocked(api.saveConversionRule).mockImplementation(async () => {
      vi.mocked(api.listConversionRules).mockResolvedValue([rule]);
      return rule;
    });
    vi.mocked(api.retireConversionRule).mockImplementation(async () => {
      vi.mocked(api.listConversionRules).mockResolvedValue([]);
    });
    open();
    await user.click(await screen.findByRole('button', { name: 'Edit Onion' }));
    await user.click(
      screen.getByText('Ingredient conversion rules', { selector: 'summary' }),
    );
    await user.click(
      await screen.findByRole('button', { name: 'Add conversion rule' }),
    );
    await user.type(screen.getByLabelText('Rule equivalent amount'), '150');
    await user.type(
      screen.getByLabelText('Rule notes / package size'),
      'Medium onion',
    );
    await user.click(
      screen.getByRole('button', { name: 'Save conversion rule' }),
    );
    expect(
      await screen.findByText(/1 pc ≈ 150 g — Medium onion/),
    ).toBeInTheDocument();
    expect(api.saveConversionRule).toHaveBeenCalledWith(null, {
      ingredientId: onion.id,
      fromUnitId: 'piece',
      fromAmount: '1',
      toUnitId: 'g',
      toAmount: '150',
      approximate: true,
      notes: 'Medium onion',
    });
    await user.click(
      screen.getByRole('button', { name: 'Edit rule rule-one' }),
    );
    expect(
      screen.getByRole('button', { name: 'Retire conversion rule' }),
    ).toBeDisabled();
    await user.click(screen.getByLabelText('Confirm retirement of this rule'));
    await user.click(
      screen.getByRole('button', { name: 'Retire conversion rule' }),
    );
    await waitFor(() =>
      expect(screen.queryByText(/1 pc ≈ 150 g/)).not.toBeInTheDocument(),
    );
    expect(api.retireConversionRule).toHaveBeenCalledWith('rule-one', true);
    expect(
      within(
        screen.getByRole('region', { name: 'Edit ingredient' }),
      ).getByLabelText('Ingredient name'),
    ).toHaveValue('Onion');
  });
});
