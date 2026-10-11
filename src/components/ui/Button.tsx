import type { ComponentProps } from 'react';

// Native buttons supply keyboard activation, focus and disabled semantics.
export function Button({
  className = '',
  type = 'button',
  ...props
}: ComponentProps<'button'>) {
  return (
    <button
      type={type}
      className={`rounded-lg border border-green-800 bg-green-800 px-4 py-2 text-sm font-semibold text-white hover:bg-green-900 disabled:cursor-not-allowed disabled:opacity-50 ${className}`}
      {...props}
    />
  );
}
