import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';
import {
  getAppInfo,
  NativeError,
  createIngredient,
  retireIngredient,
  updateMeal,
} from './ipc';
import demo from '../../tests/fixtures/demo-meal.json';

afterEach(clearMocks);

describe('native API', () => {
  it('sends named ingredient payloads and explicit retirement confirmation', async () => {
    const input = {
      name: 'Parsley',
      categoryId: null,
      sectionId: null,
      preferredUnitId: null,
      aliases: [],
      seasonMonths: [],
      notes: '',
    };
    const calls: { command: string; args: unknown }[] = [];
    mockIPC((command, args) => {
      calls.push({ command, args });
      return command === 'create_ingredient'
        ? {
            id: 'created',
            ...input,
            retired: false,
            createdAt: '',
            updatedAt: '',
          }
        : null;
    });
    expect((await createIngredient(input)).id).toBe('created');
    await retireIngredient('created', true);
    expect(calls).toEqual([
      { command: 'create_ingredient', args: { input } },
      {
        command: 'retire_ingredient',
        args: { id: 'created', confirmed: true },
      },
    ]);
  });
  it('includes the expected meal revision in updates', async () => {
    mockIPC((command, args) => {
      expect(command).toBe('update_meal');
      expect(args).toMatchObject({
        id: 'meal',
        expectedRevision: 2,
        input: { title: demo.title },
      });
      return { id: 'meal', revision: 3, ...demo };
    });
    expect((await updateMeal('meal', 2, demo)).revision).toBe(3);
  });
  it('uses the expected narrow command', async () => {
    mockIPC((command) => {
      expect(command).toBe('get_app_info');
      return { name: 'PantryPal', version: '0.1.0' };
    });
    await expect(getAppInfo()).resolves.toEqual({
      name: 'PantryPal',
      version: '0.1.0',
    });
  });

  it('preserves structured backend errors', async () => {
    mockIPC(() => {
      throw { code: 'unavailable', message: 'Try again.' };
    });
    await expect(getAppInfo()).rejects.toMatchObject({
      code: 'unavailable',
      message: 'Try again.',
    });
  });

  it('does not expose raw unexpected errors to the user', async () => {
    mockIPC(() => {
      throw new Error('private internal detail');
    });
    await expect(getAppInfo()).rejects.toBeInstanceOf(NativeError);
    await expect(getAppInfo()).rejects.toMatchObject({
      code: 'native_unavailable',
      message: 'Could not connect to PantryPal. Please restart the app.',
    });
  });
});
