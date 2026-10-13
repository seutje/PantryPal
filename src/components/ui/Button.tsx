import type { ComponentProps } from 'react';

// Native buttons supply keyboard activation, focus and disabled semantics.
export function Button({
  className = '',
  type = 'button',
  variant = type === 'submit' ? 'primary' : 'secondary',
  ...props
}: ComponentProps<'button'> & {
  variant?: 'primary' | 'secondary' | 'quiet' | 'danger';
}) {
  return (
    <button
      type={type}
      className={`pp-button pp-button-${variant} ${className}`}
      {...props}
    />
  );
}
