import {lazy, Suspense, useCallback} from 'react';

import {useQueryErrorResetBoundary} from '@tanstack/react-query';

import {Button, Flex, Separator} from '@radix-ui/themes';

import {ErrorDisplay} from '#shared/core/errors/ErrorDisplay';

import {PopoverLayout} from './PopoverLayout';

// Dev-only component stack display — excluded from production bundle
const ComponentStack = import.meta.env.DEV
  ? lazy(() => import('#(dev)/ComponentStack').then(m => ({default: m.ComponentStack})))
  : null;

// Compatible with TanStack Router's ErrorComponentProps.
// When used via react-error-boundary's fallbackRender, pass only error + reset (no info).
// Requires QueryErrorResetBoundary ancestor — provided by PopoverErrorBoundary.
type PopoverAppErrorPageProps = {
  error: Error;
  reset: () => void;
  info?: {componentStack: string} | null;
};

export const PopoverAppErrorPage = ({error, reset, info}: PopoverAppErrorPageProps) => {
  const {reset: resetQueries} = useQueryErrorResetBoundary();

  const resetQueriesAndErrorBoundary = useCallback(() => {
    resetQueries();
    reset();
  }, [resetQueries, reset]);

  return (
    <PopoverLayout>
      <Flex direction='column' gap='3' p='3'>
        <ErrorDisplay error={error} />
        {ComponentStack && info?.componentStack && (
          <Suspense fallback={null}>
            <ComponentStack stack={info.componentStack} />
          </Suspense>
        )}
        <Separator size='4' />
        <Button onClick={resetQueriesAndErrorBoundary} size='2' variant='soft'>
          Try Again
        </Button>
      </Flex>
    </PopoverLayout>
  );
};
