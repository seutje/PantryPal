import { render, screen } from '@testing-library/react';
import { StrictMode } from 'react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { AppProviders } from './providers';

vi.mock('../lib/ipc', () => ({
  nativeAvailable: () => true,
  getAppInfo: vi.fn(async () => ({ name: 'PantryPal', version: '0.1.0' })),
}));

describe('navigation shell', () => {
  it('renders the initial section and connects through the typed native API', async () => {
    render(
      <StrictMode>
        <AppProviders>
          <App />
        </AppProviders>
      </StrictMode>,
    );
    expect(
      screen.getByRole('heading', { name: 'Week', level: 1 }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Week', level: 1 }),
    ).not.toHaveFocus();
    expect(await screen.findByText('Desktop · v0.1.0')).toBeInTheDocument();
    expect(
      screen.getByRole('navigation', { name: 'Main navigation' }),
    ).toBeInTheDocument();
  });

  it('lets the keyboard reach every section and focuses the new heading', async () => {
    const user = userEvent.setup();
    render(
      <AppProviders>
        <App />
      </AppProviders>,
    );
    await user.tab();
    expect(screen.getByRole('link', { name: 'Skip to content' })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole('button', { name: 'Week' })).toHaveFocus();
    for (const name of ['Meals', 'Shopping', 'Ingredients', 'Settings']) {
      screen.getByRole('button', { name }).focus();
      await user.keyboard('{Enter}');
      expect(screen.getByRole('heading', { name, level: 1 })).toHaveFocus();
      expect(screen.getByRole('button', { name })).toHaveAttribute(
        'aria-current',
        'page',
      );
      if (name === 'Meals')
        expect(
          screen.getByRole('button', { name: 'Import recipe' }),
        ).toBeDisabled();
    }
  });
});
