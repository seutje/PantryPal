import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';
import { getAppInfo, NativeError } from './ipc';

afterEach(clearMocks);

describe('native API', () => {
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
