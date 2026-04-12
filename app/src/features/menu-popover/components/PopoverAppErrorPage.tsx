import {lazy, Suspense, useCallback} from 'react';

import {Button} from '@radix-ui/themes';
import {useQueryErrorResetBoundary} from '@tanstack/react-query';

import {ErrorDisplay} from '#shared/core/errors/ErrorDisplay';

import {PopoverLayout} from './PopoverLayout';

// Dev-only component stack display — excluded from production bundle
const ComponentStack = import.meta.env.DEV
  ? lazy(() => import('#(dev)/ComponentStack'))
  : null;

// Compatible with TanStack Router's ErrorComponentProps.
// When used via react-error-boundary's fallbackRender, pass only error + reset (no info).
interface PopoverAppErrorPageProps {
  error: Error;
  reset: () => void;
  info?: {componentStack: string} | null;
}

export const PopoverAppErrorPage = ({error, reset, info}: PopoverAppErrorPageProps) => {
  const {reset: resetQueries} = useQueryErrorResetBoundary();

  const resetQueriesAndErrorBoundary = useCallback(() => {
    resetQueries();
    reset();
  }, [resetQueries, reset]);

  return (
    <PopoverLayout>
      <ErrorDisplay error={error} />
      {ComponentStack && info?.componentStack && (
        <Suspense fallback={null}>
          <ComponentStack stack={info.componentStack} />
        </Suspense>
      )}
      <Button onClick={resetQueriesAndErrorBoundary}>Try Again</Button>
    </PopoverLayout>
  );
};
