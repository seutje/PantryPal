import { render, screen, within, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import * as api from '../../lib/ipc';
import { catalogFixture } from '../../test/catalog-fixture';
import type { FieldDefinition, Meal, MealRecord } from '../../lib/models';
import { MealsPage } from './MealsPage';
import { MealEditor } from './MealEditor';
import demo from '../../../tests/fixtures/demo-meal.json';

vi.mock('../../lib/ipc', () => ({
  readRecipeClipboardUrl: vi.fn(async () => null),
  searchMeals: vi.fn(),
  getMealHistory: vi.fn(async () => ({
    mealId: 'meal-test',
    lastCooked: null,
    events: [],
  })),
  saveMealDraft: vi.fn(),
  createField: vi.fn(),
  updateField: vi.fn(),
  retireField: vi.fn(),
  retireMeal: vi.fn(),
}));
let fields: FieldDefinition[];
let records: MealRecord[];
const meal: Meal = {
  ...demo,
  id: 'meal-test',
  revision: 1,
  retired: false,
  createdAt: '',
  updatedAt: '',
  baseServingsText: '4',
  ingredients: demo.ingredients.map((l, i) => ({
    ...l,
    amountText: i === 0 ? '⅓' : i === 1 ? '500' : 'to taste',
  })),
};
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.readRecipeClipboardUrl).mockResolvedValue(null);
  records = [];
  fields = [
    {
      id: 'freezes',
      label: 'Freezes well',
      kind: 'boolean',
      description: '',
      options: [],
      minimum: null,
      maximum: null,
      unitLabel: null,
      sortOrder: 0,
      retired: false,
    },
    {
      id: 'text',
      label: 'Family note',
      kind: 'text',
      description: '',
      options: [],
      minimum: null,
      maximum: null,
      unitLabel: null,
      sortOrder: 1,
      retired: false,
    },
  ];
  vi.mocked(api.searchMeals).mockImplementation(async () =>
    structuredClone({ meals: records, catalog: catalogFixture, fields }),
  );
});
function open() {
  return render(
    <AppProviders>
      <MealsPage />
    </AppProviders>,
  );
}
describe('meal library workflows', () => {
  it('reads optional URL prefill only when Import recipe is deliberately activated', async () => {
    const user = userEvent.setup();
    vi.mocked(api.readRecipeClipboardUrl).mockResolvedValueOnce(
      'https://example.com/owned',
    );
    open();
    const button = await screen.findByRole('button', { name: 'Import recipe' });
    expect(api.readRecipeClipboardUrl).not.toHaveBeenCalled();
    await user.click(button);
    expect(await screen.findByLabelText('Recipe URL (HTTPS)')).toHaveValue(
      'https://example.com/owned',
    );
    expect(api.readRecipeClipboardUrl).toHaveBeenCalledTimes(1);
    expect(
      screen.getByRole('button', { name: 'Open recipe website' }),
    ).toBeInTheDocument();
  });
  it('opens a retired planned recipe with its historical ingredient labels', async () => {
    const retiredMeal = {
      ...meal,
      retired: true,
      ingredients: [
        { ...meal.ingredients[0]!, ingredientId: 'historical-onion' },
      ],
    };
    const record = {
      meal: retiredMeal,
      lastCooked: null,
      servingsDisplay: '4',
      quantityDisplays: ['⅓'],
    };
    const library = {
      meals: [record],
      fields: [],
      catalog: {
        ...catalogFixture,
        ingredients: [
          {
            ...catalogFixture.ingredients[0]!,
            id: 'historical-onion',
            name: 'Retired onion',
            retired: true,
          },
        ],
      },
    };
    render(
      <AppProviders>
        <MealsPage initialMeal={record} initialLibrary={library} />
      </AppProviders>,
    );
    expect(await screen.findByText(/⅓.*Retired onion/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Edit meal' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Retire meal' })).toBeDisabled();
  });
  it('keeps invalid date filters available for correction', async () => {
    const user = userEvent.setup();
    vi.mocked(api.searchMeals).mockImplementation(async (input) => {
      if (
        input.cookedFrom &&
        input.cookedTo &&
        input.cookedFrom > input.cookedTo
      )
        throw new Error('The cooked date range is reversed.');
      return structuredClone({
        meals: records,
        catalog: catalogFixture,
        fields,
      });
    });
    open();
    await screen.findByRole('button', { name: 'New meal' });
    await user.click(screen.getByText('Filters'));
    const { fireEvent } = await import('@testing-library/react');
    fireEvent.change(screen.getByLabelText('Last cooked from'), {
      target: { value: '2026-10-11' },
    });
    fireEvent.change(screen.getByLabelText('Last cooked through'), {
      target: { value: '2026-10-10' },
    });
    expect(await screen.findByRole('alert')).toHaveTextContent('reversed');
    expect(screen.getByLabelText('Last cooked through')).toHaveValue(
      '2026-10-10',
    );
    await user.click(screen.getByRole('button', { name: 'Clear filters' }));
    await waitFor(() =>
      expect(screen.queryByRole('alert')).not.toBeInTheDocument(),
    );
  });
  it('creates a meal with original fractions, unknowns, ordering and false, then edits after reopening', async () => {
    const user = userEvent.setup();
    vi.mocked(api.saveMealDraft).mockImplementation(
      async (id, revision, draft) => {
        const saved: Meal = {
          ...meal,
          id: id ?? meal.id,
          revision: (revision ?? 0) + 1,
          title: draft.title,
          notes: draft.notes,
          customFields: draft.customFields,
        };
        records = [
          {
            meal: saved,
            lastCooked: null,
            servingsDisplay: '4',
            quantityDisplays: ['⅓', '500', null],
          },
        ];
        return saved;
      },
    );
    const page = open();
    await user.click(await screen.findByRole('button', { name: 'New meal' }));
    expect(screen.getByLabelText('Title')).toHaveFocus();
    await user.type(screen.getByLabelText('Title'), 'Family soup');
    await user.type(
      screen.getByLabelText('Instructions'),
      '<script>Stay text</script>\nSimmer',
    );
    await user.selectOptions(screen.getByLabelText('Freezes well'), 'false');
    await user.click(
      screen.getByRole('button', { name: 'Add ingredient line' }),
    );
    let line = screen.getByRole('group', { name: 'Ingredient 1' });
    await user.type(within(line).getByLabelText('Amount'), '⅓');
    await user.selectOptions(within(line).getByLabelText('Unit'), 'cup-us');
    await user.selectOptions(
      within(line).getByLabelText('Ingredient'),
      'ingredient-onion',
    );
    await user.type(
      within(line).getByLabelText('Original wording'),
      '⅓ US cup onions',
    );
    await user.click(
      screen.getByRole('button', { name: 'Add ingredient line' }),
    );
    line = screen.getByRole('group', { name: 'Ingredient 2' });
    await user.type(within(line).getByLabelText('Amount'), 'to taste');
    await user.type(
      within(line).getByLabelText('Original wording'),
      'family seasoning to taste',
    );
    await user.click(within(line).getByLabelText('Optional ingredient'));
    await user.click(
      screen.getByRole('button', { name: 'Move ingredient 2 up' }),
    );
    await user.click(screen.getByRole('button', { name: 'Save meal' }));
    await waitFor(() =>
      expect(api.saveMealDraft).toHaveBeenCalledWith(
        null,
        null,
        expect.objectContaining({
          title: 'Family soup',
          servings: '4',
          ingredients: [
            expect.objectContaining({
              amount: 'to taste',
              ingredientId: null,
              optional: true,
            }),
            expect.objectContaining({
              amount: '⅓',
              unitId: 'cup-us',
              sourceConvention: 'us',
            }),
          ],
          customFields: [
            { fieldId: 'freezes', value: { kind: 'boolean', value: false } },
          ],
        }),
      ),
    );
    await screen.findByRole('button', { name: 'View Family soup' });
    page.unmount();
    open();
    await user.click(
      await screen.findByRole('button', { name: 'View Family soup' }),
    );
    expect(screen.getByText('No')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Edit meal' }));
    expect(screen.getAllByLabelText('Amount')[0]).toHaveValue('⅓');
    await user.type(screen.getByLabelText('Notes'), 'Edited after reopening');
    await user.click(screen.getByRole('button', { name: 'Save meal' }));
    await waitFor(() =>
      expect(api.saveMealDraft).toHaveBeenLastCalledWith(
        'meal-test',
        1,
        expect.objectContaining({
          notes: 'Edited after reopening',
          customFields: [
            { fieldId: 'freezes', value: { kind: 'boolean', value: false } },
          ],
        }),
      ),
    );
  });
  it('keeps filters keyboard accessible and sends typed false separately from unknown', async () => {
    const user = userEvent.setup();
    open();
    await screen.findByRole('button', { name: 'New meal' });
    const search = screen.getByLabelText('Search meals');
    await user.type(search, 'onions');
    expect(search).toHaveFocus();
    expect(search).toHaveValue('onions');
    const summary = screen.getByText('Filters');
    summary.focus();
    await user.keyboard('{Enter}');
    await user.selectOptions(
      screen.getByLabelText('Contains ingredient'),
      'ingredient-onion',
    );
    await user.selectOptions(
      screen.getByLabelText('Custom field filter'),
      'freezes',
    );
    await user.selectOptions(
      screen.getByLabelText('Field condition'),
      'equals',
    );
    await user.selectOptions(screen.getByLabelText('Freezes well'), 'false');
    await waitFor(() =>
      expect(api.searchMeals).toHaveBeenLastCalledWith(
        expect.objectContaining({
          text: 'onions',
          ingredientId: 'ingredient-onion',
          field: {
            fieldId: 'freezes',
            operator: 'equals',
            value: { kind: 'boolean', value: false },
          },
        }),
      ),
    );
    await user.selectOptions(
      screen.getByLabelText('Field condition'),
      'unknown',
    );
    await user.selectOptions(screen.getByLabelText('Cooked history'), 'never');
    await waitFor(() =>
      expect(api.searchMeals).toHaveBeenLastCalledWith(
        expect.objectContaining({
          cooked: 'never',
          field: { fieldId: 'freezes', operator: 'unknown', value: null },
        }),
      ),
    );
  });
  it('preserves a failed draft, retired values and source text; confirms ingredient removal', async () => {
    const user = userEvent.setup();

    vi.mocked(api.saveMealDraft).mockRejectedValue(
      new Error('This meal changed.'),
    );
    fields[0]!.retired = true;
    const saved = {
      ...meal,
      customFields: [
        {
          fieldId: 'freezes',
          value: { kind: 'boolean' as const, value: false },
        },
      ],
    };
    render(
      <AppProviders>
        <MealEditor
          meal={saved}
          catalog={catalogFixture}
          fields={fields}
          onSaved={vi.fn()}
          onCancel={vi.fn()}
        />
      </AppProviders>,
    );
    expect(screen.getByText('Freezes well (retired): No')).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: 'Remove ingredient 1' }),
    );
    expect(screen.getAllByLabelText('Amount')).toHaveLength(3);
    await user.click(screen.getByRole('button', { name: 'Keep ingredient' }));
    await user.type(screen.getByLabelText('Notes'), ' preserved');
    await user.click(screen.getByRole('button', { name: 'Save meal' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Your draft is still here',
    );
    expect(screen.getByLabelText('Notes')).toHaveValue(
      'Fixture only. preserved',
    );
    expect(api.saveMealDraft).toHaveBeenCalledWith(
      'meal-test',
      1,
      expect.objectContaining({
        ingredients: [
          expect.objectContaining({
            amount: '⅓',
            rawText: '1/3 US cup rice',
            sourceConvention: 'us',
          }),
          expect.anything(),
          expect.objectContaining({ amount: 'to taste' }),
        ],
        customFields: saved.customFields,
      }),
    );
    await user.click(
      screen.getByRole('button', { name: 'Remove ingredient 2' }),
    );
    await user.click(screen.getByRole('button', { name: 'Confirm removal' }));
    expect(screen.getAllByLabelText('Amount')).toHaveLength(2);
  });
  it('creates and safely renames and retires custom fields with confirmation', async () => {
    const user = userEvent.setup();
    vi.mocked(api.createField).mockImplementation(async (input) => {
      const f = { ...input, id: 'choice', retired: false };
      fields.push(f);
      return f;
    });
    vi.mocked(api.updateField).mockImplementation(async (id, input) => {
      const f = { ...input, id, retired: false };
      fields = fields.map((old) => (old.id === id ? f : old));
      return f;
    });
    vi.mocked(api.retireField).mockImplementation(async (id) => {
      fields = fields.map((f) => (f.id === id ? { ...f, retired: true } : f));
    });
    open();
    await user.click(
      await screen.findByRole('button', { name: 'Manage custom fields' }),
    );
    await user.click(screen.getByRole('button', { name: 'New custom field' }));
    await user.type(screen.getByLabelText('Field label'), 'Season');
    await user.selectOptions(screen.getByLabelText('Field kind'), 'select');
    await user.click(screen.getByRole('button', { name: 'Add choice' }));
    await user.type(screen.getByLabelText('Choice 1'), 'Winter');
    await user.click(screen.getByRole('button', { name: 'Save field' }));
    await user.click(
      await screen.findByRole('button', { name: 'Edit Season' }),
    );
    expect(screen.getByLabelText('Field kind')).toBeDisabled();
    await user.clear(screen.getByLabelText('Field label'));
    await user.type(screen.getByLabelText('Field label'), 'Preferred season');
    await user.click(screen.getByRole('button', { name: 'Save field' }));
    await user.click(
      await screen.findByRole('button', { name: 'Retire Preferred season' }),
    );
    expect(api.retireField).not.toHaveBeenCalled();
    await user.click(
      screen.getByRole('button', { name: 'Confirm field retirement' }),
    );
    await waitFor(() =>
      expect(api.retireField).toHaveBeenCalledWith('choice', true),
    );
    expect(
      await screen.findByText('Preferred season · select · retired'),
    ).toBeInTheDocument();
  });
});
