import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import { Actions } from './Actions';
import { Button } from './Button';

it('supports keyboard disclosure, Escape and trigger focus without activating an action', async () => {
  const user = userEvent.setup();
  const apply = vi.fn();
  render(
    <Actions label="More meal actions">
      <Button onClick={apply}>Retire meal</Button>
    </Actions>,
  );
  const summary = screen.getByText('More meal actions', {
    selector: 'summary',
  });
  summary.focus();
  await user.keyboard('{Enter}');
  expect(summary.closest('details')).toHaveAttribute('open');
  await user.tab();
  expect(screen.getByRole('button', { name: 'Retire meal' })).toHaveFocus();
  await user.keyboard('{Escape}');
  expect(summary.closest('details')).not.toHaveAttribute('open');
  expect(summary).toHaveFocus();
  expect(apply).not.toHaveBeenCalled();
});
