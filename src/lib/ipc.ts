import { invoke, isTauri } from '@tauri-apps/api/core';

export interface AppInfo {
  name: string;
  version: string;
}

export interface AppError {
  code: string;
  message: string;
}

export class NativeError extends Error {
  readonly code: string;

  constructor(error: AppError) {
    super(error.message);
    this.name = 'NativeError';
    this.code = error.code;
  }
}

function normalizeError(error: unknown): NativeError {
  if (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error &&
    typeof error.code === 'string' &&
    typeof error.message === 'string'
  ) {
    return new NativeError({ code: error.code, message: error.message });
  }
  return new NativeError({
    code: 'native_unavailable',
    message: 'Could not connect to PantryPal. Please restart the app.',
  });
}

export const nativeAvailable = isTauri;

// Expose named, typed operations, never a generic UI-controlled command name.
export async function getAppInfo(): Promise<AppInfo> {
  try {
    return await invoke<AppInfo>('get_app_info');
  } catch (error: unknown) {
    throw normalizeError(error);
  }
}
