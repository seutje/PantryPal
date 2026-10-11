import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { ImportPage } from './ImportPage';
import * as api from '../../lib/ipc';
import { catalogFixture } from '../../test/catalog-fixture';
import type { ImportedRecipeDraft, Meal } from '../../lib/models';
import sites from '../../../tests/fixtures/import-sites.json';
vi.mock('../../lib/ipc', () => ({
  inspectRecipeUrl: vi.fn(),
  openRecipeBrowser: vi.fn(),
  previewRecipeImport: vi.fn(),
  confirmRecipeImport: vi.fn(),
  resetRecipeBrowser: vi.fn(),
}));
const draft: ImportedRecipeDraft = {
  meal: {
    title: 'Owned fixture soup',
    description: '',
    servings: '',
    prepMinutes: null,
    cookMinutes: null,
    instructions: '<script>inert</script>',
    notes: '',
    sourceUrl: 'https://example.com/owned-recipe',
    sourceSite: 'example.com',
    tags: [],
    customFields: [],
    ingredients: [
      {
        id: 'line',
        ingredientId: null,
        amount: '1/2',
        unitId: null,
        rawText: '1/2 cup milk',
        sourceConvention: null,
        note: '',
        optional: false,
      },
    ],
  },
  warnings: ['Review regional units and exact ingredient matches.'],
  originalYield: '4 servings',
  attribution: 'Original test author',
  duplicateTitles: ['Earlier import'],
};
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.inspectRecipeUrl).mockResolvedValue(
    'No automated site adapter. Use permitted manual text.',
  );
  vi.mocked(api.openRecipeBrowser).mockResolvedValue();
  vi.mocked(api.previewRecipeImport).mockResolvedValue(structuredClone(draft));
  vi.mocked(api.resetRecipeBrowser).mockResolvedValue();
});
function open() {
  const onSaved = vi.fn();
  const onCancel = vi.fn();
  render(
    <AppProviders>
      <ImportPage
        catalog={catalogFixture}
        fields={[]}
        onSaved={onSaved}
        onCancel={onCancel}
      />
    </AppProviders>,
  );
  return { onSaved, onCancel };
}
it('reviews a supplied draft, resolves cups and identity, and retries an identical save safely', async () => {
  const user = userEvent.setup();
  const { onSaved } = open();
  await user.type(
    screen.getByLabelText('Recipe URL (HTTPS)'),
    draft.meal.sourceUrl!,
  );
  await user.click(screen.getByRole('button', { name: 'Check website' }));
  await user.click(
    await screen.findByRole('button', { name: 'Open recipe website' }),
  );
  await waitFor(() =>
    expect(api.openRecipeBrowser).toHaveBeenCalledWith(draft.meal.sourceUrl),
  );
  expect(screen.getByRole('button', { name: 'Preview recipe' })).toBeDisabled();
  await user.selectOptions(screen.getByLabelText('Import method'), 'json_ld');
  await user.type(
    screen.getByLabelText('Recipe JSON-LD'),
    'original supplied JSON',
  );
  await user.click(
    screen.getByLabelText(
      'I am permitted to import this supplied recipe content.',
    ),
  );
  await user.click(screen.getByRole('button', { name: 'Preview recipe' }));
  await screen.findByText(/Already saved from this source/);
  expect(api.confirmRecipeImport).not.toHaveBeenCalled();
  expect(screen.getByLabelText('Instructions')).toHaveValue(
    '<script>inert</script>',
  );
  expect(document.querySelector('script')).toBeNull();
  await user.type(screen.getByLabelText('Base servings'), '4');
  const line = within(screen.getByRole('group', { name: 'Ingredient 1' }));
  await user.selectOptions(line.getByLabelText('Unit'), 'cup-us');
  await user.selectOptions(
    line.getByLabelText('Ingredient'),
    catalogFixture.ingredients[0]!.id,
  );
  await user.click(screen.getByRole('button', { name: 'Save meal' }));
  await screen.findByRole('alert');
  expect(api.confirmRecipeImport).not.toHaveBeenCalled();
  await user.click(
    screen.getByLabelText(
      'I reviewed servings, quantities, regional units and ingredient matches.',
    ),
  );
  await user.click(
    screen.getByLabelText('Save another meal if this source already exists.'),
  );
  vi.mocked(api.confirmRecipeImport)
    .mockRejectedValueOnce(new Error('Response unavailable'))
    .mockResolvedValueOnce({ title: 'Saved import' } as Meal);
  await user.click(screen.getByRole('button', { name: 'Save meal' }));
  await screen.findByText(/Response unavailable/);
  expect(line.getByLabelText('Original wording')).toHaveValue('1/2 cup milk');
  await user.click(screen.getByRole('button', { name: 'Save meal' }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  const first = vi.mocked(api.confirmRecipeImport).mock.calls[0]![0];
  const second = vi.mocked(api.confirmRecipeImport).mock.calls[1]![0];
  expect(second).toEqual(first);
  expect(first.reviewed.ingredients[0]).toMatchObject({
    unitId: 'cup-us',
    sourceConvention: 'us',
    rawText: '1/2 cup milk',
  });
  expect(first.allowDuplicate).toBe(true);
});
it('retains rejected input, supports manual entry and confirmed cancellation/session reset', async () => {
  const user = userEvent.setup();
  const { onCancel } = open();
  await user.type(screen.getByLabelText('Recipe URL (HTTPS)'), sites[0]!.url);
  vi.mocked(api.inspectRecipeUrl).mockRejectedValueOnce(
    new Error('Website unavailable'),
  );
  await user.click(screen.getByRole('button', { name: 'Check website' }));
  await screen.findByText(/Website unavailable/);
  expect(screen.getByLabelText('Recipe URL (HTTPS)')).toHaveValue(
    sites[0]!.url,
  );
  await user.type(screen.getByLabelText('Recipe title'), 'My personal recipe');
  await user.type(screen.getByLabelText('Recipe servings'), '1 1/2');
  await user.type(
    screen.getByLabelText('Recipe ingredients (one per line)'),
    'salt to taste',
  );
  await user.click(
    screen.getByLabelText(
      'I am permitted to import this supplied recipe content.',
    ),
  );
  vi.mocked(api.previewRecipeImport).mockRejectedValueOnce(
    new Error('Check recipe data'),
  );
  await user.click(screen.getByRole('button', { name: 'Preview recipe' }));
  await screen.findByText(/Check recipe data/);
  expect(
    JSON.parse(vi.mocked(api.previewRecipeImport).mock.calls[0]![0].payload),
  ).toMatchObject({
    title: 'My personal recipe',
    servings: '1 1/2',
    ingredients: ['salt to taste'],
  });
  await user.click(
    screen.getByRole('button', { name: 'Clear website sessions' }),
  );
  expect(api.resetRecipeBrowser).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole('button', { name: 'Confirm clear sessions' }),
  );
  await screen.findByText('Website sessions cleared.');
  await user.click(screen.getByRole('button', { name: 'Cancel import' }));
  expect(onCancel).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Discard import' }));
  expect(onCancel).toHaveBeenCalled();
  expect(api.confirmRecipeImport).not.toHaveBeenCalled();
});
