import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { MealEditor } from './MealEditor';
import { catalogFixture, onion } from '../../test/catalog-fixture';
import * as api from '../../lib/ipc';
import type { Ingredient, MealDraft } from '../../lib/models';

vi.mock('../../lib/ipc', () => ({
  searchIngredients: vi.fn(),
  ingredientCandidates: vi.fn(),
  createIngredient: vi.fn(),
  saveMealDraft: vi.fn(),
}));

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.searchIngredients).mockResolvedValue([]);
  vi.mocked(api.ingredientCandidates).mockResolvedValue([]);
});

function open(rawText = '') {
  const draft: MealDraft = {
    title: 'Soup',
    description: '',
    servings: '4',
    prepMinutes: null,
    cookMinutes: null,
    instructions: '',
    notes: '',
    sourceUrl: null,
    sourceSite: null,
    tags: [],
    customFields: [],
    ingredients: [
      {
        id: 'line',
        ingredientId: null,
        amount: '1/2',
        unitId: 'g',
        rawText,
        sourceConvention: null,
        note: 'chopped',
        optional: true,
      },
    ],
  };
  const submit = vi.fn().mockRejectedValue(new Error('Test save'));
  render(
    <AppProviders>
      <MealEditor
        catalog={catalogFixture}
        fields={[]}
        initialDraft={draft}
        submitDraft={submit}
        onSaved={vi.fn()}
        onCancel={vi.fn()}
      />
    </AppProviders>,
  );
  return {
    user: userEvent.setup(),
    input: screen.getByRole('combobox', { name: 'Ingredient' }),
    submit,
  };
}

it('requires choosing a typo suggestion, supports keyboard selection and preserves source quantities', async () => {
  vi.mocked(api.searchIngredients).mockResolvedValue([onion]);
  const { user, input, submit } = open('1/2 g onoin, chopped');
  await user.type(input, 'onoin');
  await screen.findByRole('option', { name: 'Onion' });
  await user.keyboard('{Enter}');
  expect(input).toHaveValue('onoin');
  expect(api.createIngredient).not.toHaveBeenCalled();
  expect(submit).not.toHaveBeenCalled();
  await user.keyboard('{ArrowDown}{Enter}');
  expect(input).toHaveValue('Onion');
  expect(input).toHaveFocus();
  await user.click(screen.getByRole('button', { name: 'Save meal' }));
  await waitFor(() =>
    expect(submit).toHaveBeenCalledWith(
      expect.objectContaining({
        ingredients: [
          expect.objectContaining({
            ingredientId: onion.id,
            rawText: '1/2 g onoin, chopped',
            amount: '1/2',
            unitId: 'g',
            note: 'chopped',
            optional: true,
          }),
        ],
      }),
    ),
  );
});

it('adds a missing ingredient on Enter, blocks meal saving during creation and immediately selects it', async () => {
  let finish!: (ingredient: Ingredient) => void;
  vi.mocked(api.createIngredient).mockImplementation(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  const { user, input, submit } = open();
  await user.type(input, 'Sumac');
  await screen.findByRole('option', {
    name: 'Add “Sumac” as a new ingredient',
  });
  await user.keyboard('{Enter}');
  await waitFor(() => expect(api.createIngredient).toHaveBeenCalledTimes(1));
  expect(screen.getByRole('button', { name: 'Save meal' })).toBeDisabled();
  finish({ ...onion, name: 'Sumac', aliases: [], id: 'sumac' });
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Save meal' })).toBeEnabled(),
  );
  expect(input).toHaveValue('Sumac');
  await user.click(screen.getByRole('button', { name: 'Save meal' }));
  expect(submit).toHaveBeenCalledWith(
    expect.objectContaining({
      ingredients: [
        expect.objectContaining({ ingredientId: 'sumac', rawText: 'Sumac' }),
      ],
    }),
  );
});

it('reuses an exact alias on Enter without creating a duplicate', async () => {
  vi.mocked(api.searchIngredients).mockResolvedValue([onion]);
  const { user, input } = open();
  await user.type(input, 'Onions');
  await screen.findByRole('option', { name: 'Onion' });
  expect(
    screen.queryByRole('option', { name: /as a new ingredient/ }),
  ).not.toBeInTheDocument();
  await user.keyboard('{Enter}');
  expect(input).toHaveValue('Onion');
  expect(api.createIngredient).not.toHaveBeenCalled();
});

it('allows explicitly creating a different ingredient despite possible matches', async () => {
  vi.mocked(api.searchIngredients).mockResolvedValue([onion]);
  vi.mocked(api.createIngredient).mockResolvedValue({
    ...onion,
    name: 'Onion powder',
    id: 'powder',
  });
  const { user, input, submit } = open();
  await user.type(input, 'Onion powder');
  await user.click(
    await screen.findByRole('option', { name: /Add “Onion powder”/ }),
  );
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Save meal' })).toBeEnabled(),
  );
  expect(api.createIngredient).toHaveBeenCalledWith(
    expect.objectContaining({ name: 'Onion powder', aliases: [] }),
  );
  await user.click(screen.getByRole('button', { name: 'Save meal' }));
  expect(submit).toHaveBeenCalledWith(
    expect.objectContaining({
      ingredients: [expect.objectContaining({ ingredientId: 'powder' })],
    }),
  );
});

it('rechecks exact identities before creation to avoid a duplicate from stale suggestions', async () => {
  vi.mocked(api.ingredientCandidates).mockResolvedValue([onion]);
  const { user, input } = open();
  await user.type(input, 'Onions');
  await user.click(await screen.findByRole('option', { name: /Add “Onions”/ }));
  await waitFor(() => expect(input).toHaveValue('Onion'));
  expect(api.createIngredient).not.toHaveBeenCalled();
});

it('retains the wording after a failed creation and permits retry', async () => {
  vi.mocked(api.createIngredient)
    .mockRejectedValueOnce(new Error('Disk full'))
    .mockResolvedValueOnce({ ...onion, name: 'Sumac', id: 'sumac' });
  const { user, input } = open();
  await user.type(input, 'Sumac');
  await user.click(await screen.findByRole('option', { name: /Add “Sumac”/ }));
  expect(await screen.findByRole('alert')).toHaveTextContent('Disk full');
  expect(input).toHaveValue('Sumac');
  await user.click(input);
  await user.click(await screen.findByRole('option', { name: /Add “Sumac”/ }));
  await waitFor(() => expect(api.createIngredient).toHaveBeenCalledTimes(2));
  await waitFor(() => expect(input).toHaveAttribute('aria-expanded', 'false'));
});

it('prevents creating from a failed search and rechecks reserved retired identities before writing', async () => {
  vi.mocked(api.searchIngredients).mockRejectedValueOnce(
    new Error('Search unavailable'),
  );
  const { user, input } = open();
  await user.type(input, 'X');
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'Search unavailable',
  );
  await user.keyboard('{Enter}');
  expect(api.createIngredient).not.toHaveBeenCalled();
  vi.mocked(api.ingredientCandidates).mockResolvedValue([
    { ...onion, retired: true },
  ]);
  await user.type(input, 'yz');
  await user.click(
    await screen.findByRole('option', { name: /as a new ingredient/ }),
  );
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'retired ingredient',
  );
  expect(api.createIngredient).not.toHaveBeenCalled();
});
