import type { ReactNode } from 'react';

export function EmptyState({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <section
      className="rounded-xl border border-stone-200 bg-white p-8"
      aria-label={title}
    >
      <h2 className="mb-2 text-xl font-semibold text-stone-900">{title}</h2>
      <p className="max-w-xl leading-relaxed text-stone-600">{children}</p>
    </section>
  );
}
