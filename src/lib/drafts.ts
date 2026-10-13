import { createContext, useContext, useEffect, useId } from 'react';

export const DraftContext = createContext<{
  register: (id: string, dirty: boolean, pending: boolean) => void;
  remove: (id: string) => void;
  canLeave: () => boolean;
}>({
  register: () => {},
  remove: () => {},
  canLeave: () => true,
});

/** Warn before leaving an editor; native writes must finish before navigation. */
export function useDraftGuard(dirty: boolean, pending = false) {
  const context = useContext(DraftContext);
  const id = useId();
  useEffect(() => {
    context.register(id, dirty, pending);
    return () => context.remove(id);
  }, [context, id, dirty, pending]);
  return () =>
    !pending && (!dirty || window.confirm('Discard your unsaved changes?'));
}
