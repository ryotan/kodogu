import {useCallback, useEffect} from 'react';

import {useQueryErrorResetBoundary} from '@tanstack/react-query';
import {useRouter} from '@tanstack/react-router';

import {reportError} from '#reportError';

// Resets with `router.invalidate()`: the router's own `reset` only clears the error boundary, and a match whose
// loader failed throws the same error again.
export function useRouteErrorReset(): () => void {
  const router = useRouter();
  const {reset: resetQueries} = useQueryErrorResetBoundary();

  // Lets failed queries fetch again when the routes render next.
  useEffect(() => {
    resetQueries();
  }, [resetQueries]);

  return useCallback(() => {
    router.invalidate().catch((e: unknown) => {
      reportError('Failed to reload the routes', e);
    });
  }, [router]);
}
