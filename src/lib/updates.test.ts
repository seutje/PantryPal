import { beforeEach, expect, it, vi } from 'vitest';
const native = vi.hoisted(() => ({ invoke: vi.fn(), isTauri: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({
  ...native,
  Channel: class {
    onmessage = () => {};
  },
}));
beforeEach(() => {
  vi.resetModules();
  vi.resetAllMocks();
});
it('uses one startup request and sends only explicit install consent/version/channel', async () => {
  native.isTauri.mockReturnValue(true);
  native.invoke.mockResolvedValue({ enabled: true, update: null });
  const api = await import('./updates');
  expect(api.startupUpdate()).toBe(api.startupUpdate());
  await api.startupUpdate();
  expect(native.invoke).toHaveBeenCalledOnce();
  await api.installUpdate('0.2.0', vi.fn());
  expect(native.invoke).toHaveBeenLastCalledWith('install_app_update', {
    version: '0.2.0',
    confirmed: true,
    progress: expect.any(Object),
  });
});
it('does not contact an updater in browser preview', async () => {
  native.isTauri.mockReturnValue(false);
  const api = await import('./updates');
  expect(await api.checkUpdate()).toEqual({ enabled: false, update: null });
  expect(native.invoke).not.toHaveBeenCalled();
});
