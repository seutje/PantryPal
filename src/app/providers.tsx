import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useEffect, useState, type ReactNode } from 'react';
import { DraftContext } from '../lib/drafts';

export function AppProviders({ children }: { children: ReactNode }) {
  const [drafts] = useState(
    () => new Map<string, { dirty: boolean; pending: boolean }>(),
  );
  const [draftContext] = useState(() => ({
    register: (id: string, dirty: boolean, pending: boolean) => {
      drafts.set(id, { dirty, pending });
    },
    remove: (id: string) => {
      drafts.delete(id);
    },
    canLeave: () => {
      if ([...drafts.values()].some((draft) => draft.pending)) return false;
      return (
        ![...drafts.values()].some((draft) => draft.dirty) ||
        window.confirm('Discard your unsaved changes and leave this screen?')
      );
    },
  }));
  useEffect(() => {
    const warn = (event: BeforeUnloadEvent) => {
      if ([...drafts.values()].some((draft) => draft.dirty || draft.pending)) {
        event.preventDefault();
        event.returnValue = '';
      }
    };
    window.addEventListener('beforeunload', warn);
    return () => window.removeEventListener('beforeunload', warn);
  }, [drafts]);
  const [client] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: { retry: false, refetchOnWindowFocus: false },
        },
      }),
  );
  return (
    <DraftContext.Provider value={draftContext}>
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    </DraftContext.Provider>
  );
}
