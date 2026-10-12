import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(cleanup);

// JSDOM has no native dialog implementation. Production WebView2 provides the
// modal top layer and focus trap; emulate opening/closing for workflow tests.
HTMLDialogElement.prototype.showModal = function () {
  this.setAttribute('open', '');
  this.querySelector<HTMLElement>('input, button, select, textarea')?.focus();
};
HTMLDialogElement.prototype.close = function () {
  this.removeAttribute('open');
};
