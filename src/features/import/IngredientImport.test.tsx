import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { ImportPage } from './ImportPage';
import * as api from '../../lib/ipc';
import { catalogFixture, onion } from '../../test/catalog-fixture';
import type { ImportedRecipeDraft, ImportInput, Meal } from '../../lib/models';

vi.mock('../../lib/ipc', () => ({
  inspectRecipeUrl: vi.fn(),
  openRecipeBrowser: vi.fn(),
  previewRecipeImport: vi.fn(),
  confirmRecipeImport: vi.fn(),
  resetRecipeBrowser: vi.fn(),
  captureRecipeBrowser: vi.fn(),
  ingredientCandidates: vi.fn(),
  createIngredient: vi.fn(),
}));
const source: ImportInput = {
  sourceUrl: 'https://www.budgetbytes.com/chicken-noodle-soup/',
  format: 'website',
  payload: 'original visible capture',
  permitted: true,
};
const draft: ImportedRecipeDraft = {
  meal: {
    title: 'Scraped recipe',
    description: '',
    servings: '4',
    prepMinutes: null,
    cookMinutes: null,
    instructions: 'Original instructions',
    notes: '',
    sourceUrl: source.sourceUrl,
    sourceSite: 'www.budgetbytes.com',
    tags: [],
    customFields: [],
    ingredients: [
      {
        id: 'milk-line',
        ingredientId: null,
        amount: '1/2',
        unitId: null,
        rawText: '1/2 cup milk',
        sourceConvention: null,
        note: 'Review ingredient: cup milk',
        optional: false,
      },
      {
        id: 'other-line',
        ingredientId: null,
        amount: '',
        unitId: null,
        rawText: 'salt to taste',
        sourceConvention: null,
        note: '',
        optional: false,
      },
    ],
  },
  warnings: [],
  originalYield: '4',
  attribution: 'Fixture author',
  duplicateTitles: [],
};
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.inspectRecipeUrl).mockResolvedValue('Authorized adapter');
  vi.mocked(api.captureRecipeBrowser).mockResolvedValue({
    source,
    draft: structuredClone(draft),
  });
  vi.mocked(api.ingredientCandidates).mockResolvedValue([]);
});
async function review() {
  const user = userEvent.setup();
  render(
    <AppProviders>
      <ImportPage
        catalog={catalogFixture}
        fields={[]}
        onSaved={vi.fn()}
        onCancel={vi.fn()}
      />
    </AppProviders>,
  );
  await user.type(
    screen.getByLabelText('Recipe URL (HTTPS)'),
    source.sourceUrl,
  );
  await user.click(screen.getByRole('button', { name: 'Check website' }));
  await user.click(
    screen.getByLabelText(
      'I am permitted to import this supplied recipe content.',
    ),
  );
  await user.click(
    screen.getByRole('button', { name: 'Read recipe from browser' }),
  );
  await screen.findByLabelText('Base servings');
  return user;
}
it('creates and selects a missing ingredient during review, retaining failed/cancelled drafts and import provenance', async () => {
  const user = await review();
  await user.clear(screen.getByLabelText('Title'));
  await user.type(screen.getByLabelText('Title'), 'Keep reviewed title');
  const line = within(screen.getByRole('group', { name: 'Ingredient 1' }));
  await user.click(
    line.getByText('Find or add ingredient', { selector: 'summary' }),
  );
  const add = line.getByRole('button', { name: 'Add ingredient' });
  await user.click(add);
  let modal = screen.getByRole('dialog', { name: 'Add ingredient' });
  expect(modal).toHaveAttribute('open');
  await user.type(
    within(modal).getByLabelText('Ingredient name'),
    'Discarded name',
  );
  const discard = vi
    .spyOn(window, 'confirm')
    .mockReturnValueOnce(false)
    .mockReturnValueOnce(true);
  fireEvent(modal, new Event('cancel', { cancelable: true }));
  expect(modal).toBeInTheDocument();
  expect(within(modal).getByLabelText('Ingredient name')).toHaveValue(
    'Discarded name',
  );
  fireEvent(modal, new Event('cancel', { cancelable: true }));
  discard.mockRestore();
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(add).toHaveFocus();
  expect(line.getByLabelText('Ingredient')).toHaveValue('');
  expect(api.createIngredient).not.toHaveBeenCalled();
  await user.click(add);
  modal = screen.getByRole('dialog', { name: 'Add ingredient' });
  await user.type(within(modal).getByLabelText('Ingredient name'), 'Milk');
  vi.mocked(api.createIngredient)
    .mockRejectedValueOnce(new Error('Save failed'))
    .mockImplementationOnce(async (input) => ({
      ...input,
      id: 'new-milk',
      retired: false,
      createdAt: '',
      updatedAt: '',
    }));
  await user.click(
    within(modal).getByRole('button', { name: 'Save ingredient' }),
  );
  await within(modal).findByText('Save failed');
  expect(within(modal).getByLabelText('Ingredient name')).toHaveValue('Milk');
  expect(api.confirmRecipeImport).not.toHaveBeenCalled();
  await user.click(
    within(modal).getByRole('button', { name: 'Save ingredient' }),
  );
  await waitFor(() =>
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument(),
  );
  expect(add).toHaveFocus();
  expect(line.getByLabelText('Ingredient')).toHaveValue('Milk');
  expect(line.getByLabelText('Original wording')).toHaveValue('1/2 cup milk');
  expect(line.getByLabelText('Amount')).toHaveValue('1/2');
  expect(line.getByLabelText('Unit')).toHaveValue('');
  expect(screen.getByLabelText('Title')).toHaveValue('Keep reviewed title');
  expect(screen.getByLabelText('Instructions')).toHaveValue(
    'Original instructions',
  );
  expect(api.confirmRecipeImport).not.toHaveBeenCalled();
  await user.click(
    screen.getByLabelText(
      'I reviewed servings, quantities, regional units and ingredient matches.',
    ),
  );
  vi.mocked(api.confirmRecipeImport).mockResolvedValue({
    title: 'Saved',
  } as Meal);
  await user.click(screen.getByRole('button', { name: 'Save meal' }));
  await waitFor(() => expect(api.confirmRecipeImport).toHaveBeenCalled());
  const request = vi.mocked(api.confirmRecipeImport).mock.calls[0]![0];
  expect(request.source).toEqual(source);
  expect(request.reviewed.title).toBe('Keep reviewed title');
  expect(request.reviewed.ingredients[0]).toEqual({
    ...draft.meal.ingredients[0],
    ingredientId: 'new-milk',
  });
  expect(request.reviewed.ingredients[1]).toEqual(draft.meal.ingredients[1]);
});
it('requires an explicit choice of an existing duplicate and changes only the chosen recipe line', async () => {
  const user = await review();
  const line = within(screen.getByRole('group', { name: 'Ingredient 2' }));
  await user.click(line.getByRole('button', { name: 'Add ingredient' }));
  const modal = within(screen.getByRole('dialog', { name: 'Add ingredient' }));
  await user.type(modal.getByLabelText('Ingredient name'), 'Onions');
  vi.mocked(api.ingredientCandidates).mockResolvedValue([onion]);
  await user.click(modal.getByRole('button', { name: 'Save ingredient' }));
  await modal.findByRole('button', { name: 'Use existing Onion' });
  expect(line.getByLabelText('Ingredient')).toHaveValue('');
  expect(api.createIngredient).not.toHaveBeenCalled();
  await user.click(modal.getByRole('button', { name: 'Use existing Onion' }));
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(line.getByLabelText('Ingredient')).toHaveValue(onion.name);
  expect(
    within(screen.getByRole('group', { name: 'Ingredient 1' })).getByLabelText(
      'Ingredient',
    ),
  ).toHaveValue('');
  expect(api.confirmRecipeImport).not.toHaveBeenCalled();
});
