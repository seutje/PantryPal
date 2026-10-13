import { StrictMode } from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';
import { UpdateNotice } from './UpdateNotice';
import * as updates from '../../lib/updates';
vi.mock('../../lib/updates', () => ({
  startupUpdate: vi.fn(),
  checkUpdate: vi.fn(),
  installUpdate: vi.fn(),
}));
beforeEach(() => {
  vi.resetAllMocks();
});
it('survives StrictMode cleanup, focuses Not now and postpones without downloading', async () => {
  const user = userEvent.setup();
  let complete!: (value: updates.UpdateCheck) => void;
  const promise = new Promise<updates.UpdateCheck>((resolve) => {
    complete = resolve;
  });
  vi.mocked(updates.startupUpdate).mockReturnValue(promise);
  const opener = document.createElement('button');
  document.body.append(opener);
  opener.focus();
  render(
    <StrictMode>
      <UpdateNotice />
    </StrictMode>,
  );
  complete({
    enabled: true,
    update: { version: '0.2.0', notes: '<script>inert notes</script>' },
  });
  expect(
    await screen.findByRole('dialog', { name: 'PantryPal 0.2.0 update' }),
  ).toBeVisible();
  expect(screen.getByText('<script>inert notes</script>')).toBeVisible();
  expect(screen.getByRole('button', { name: 'Not now' })).toHaveFocus();
  await user.keyboard('{Escape}');
  fireEvent(
    screen.getByRole('dialog'),
    new Event('cancel', { cancelable: true }),
  );
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(updates.installUpdate).not.toHaveBeenCalled();
  expect(opener).toHaveFocus();
  opener.remove();
});
it('ignores an offline startup failure and reports deliberate check errors', async () => {
  vi.mocked(updates.startupUpdate).mockRejectedValue(
    new Error('private transport detail'),
  );
  const { unmount } = render(<UpdateNotice />);
  await waitFor(() => expect(updates.startupUpdate).toHaveBeenCalled());
  expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  unmount();
  vi.mocked(updates.checkUpdate).mockRejectedValue(new Error('offline'));
  const user = userEvent.setup();
  render(<UpdateNotice manual />);
  await user.click(screen.getByRole('button', { name: 'Check for updates' }));
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'keep using PantryPal offline',
  );
  vi.mocked(updates.checkUpdate).mockResolvedValue({
    enabled: true,
    update: null,
  });
  await user.click(screen.getByRole('button', { name: 'Check for updates' }));
  expect(await screen.findByRole('status')).toHaveTextContent('up to date');
});
it('requires deliberate installation, announces progress, blocks cancellation during handoff and retains retry', async () => {
  const user = userEvent.setup();
  vi.mocked(updates.startupUpdate).mockResolvedValue({
    enabled: true,
    update: { version: '0.2.0', notes: '' },
  });
  let fail!: (reason: Error) => void;
  vi.mocked(updates.installUpdate).mockImplementation(
    (_version, onProgress) => {
      onProgress({
        phase: 'downloading',
        downloaded: 50,
        total: 100,
        recoveryPath: null,
      });
      return new Promise((_resolve, reject) => {
        fail = reject;
      });
    },
  );
  render(<UpdateNotice />);
  await screen.findByRole('dialog');
  expect(updates.installUpdate).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole('button', { name: 'Download and install' }),
  );
  expect(screen.getByText('50%')).toBeVisible();
  expect(screen.getByRole('button', { name: 'Not now' })).toBeDisabled();
  await user.keyboard('{Escape}');
  fireEvent(
    screen.getByRole('dialog'),
    new Event('cancel', { cancelable: true }),
  );
  expect(screen.getByRole('dialog')).toBeVisible();
  fail(new Error('invalid signature'));
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'household data has not been replaced',
  );
  expect(
    screen.getByRole('button', { name: 'Download and install' }),
  ).toBeEnabled();
});
