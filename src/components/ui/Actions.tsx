import { useRef, type ReactNode } from 'react';

/** Native disclosure: ordinary tab order, no unsupported ARIA menu semantics. */
export function Actions({
  label = 'More actions',
  accessibleLabel,
  children,
}: {
  label?: string;
  accessibleLabel?: string;
  children: ReactNode;
}) {
  const ref = useRef<HTMLDetailsElement>(null);
  return (
    <details
      ref={ref}
      className="pp-actions"
      onKeyDown={(event) => {
        if (event.key === 'Escape' && ref.current?.open) {
          event.preventDefault();
          ref.current.open = false;
          ref.current.querySelector('summary')?.focus();
        }
      }}
    >
      <summary
        aria-label={accessibleLabel}
        onKeyDown={(event) => {
          if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            if (ref.current) ref.current.open = !ref.current.open;
          }
        }}
      >
        {label}
      </summary>
      <div
        className="pp-actions-content"
        onClick={(event) => {
          if (
            (event.target as HTMLElement).closest('button:not(:disabled)') &&
            ref.current
          ) {
            ref.current.open = false;
            ref.current.querySelector('summary')?.focus();
          }
        }}
      >
        {children}
      </div>
    </details>
  );
}
