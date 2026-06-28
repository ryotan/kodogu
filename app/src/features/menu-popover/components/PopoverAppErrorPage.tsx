import {useCallback} from 'react';

import {useQueryErrorResetBoundary} from '@tanstack/react-query';

import {Card, Inset} from '@radix-ui/themes';

import type {ComponentErrorInfo} from '#shared/core/errors/ComponentErrorInfo';
import {ErrorPage} from '#shared/core/errors/ErrorPage';

import {PopoverLayout} from './PopoverLayout';

// Compatible with TanStack Router's ErrorComponentProps (`error`, `reset`) and
// react-error-boundary's `fallbackRender` (`error`, `resetErrorBoundary`).
// When used via `fallbackRender`, no `errorInfo` is supplied — `componentStack`
// is unavailable in that mode (use `FallbackComponent` if you need it).
type PopoverAppErrorPageProps = {
  error: Error;
  reset: () => void;
  errorInfo?: ComponentErrorInfo;
};

export const PopoverAppErrorPage = ({error, reset, errorInfo}: PopoverAppErrorPageProps) => {
  const {reset: resetQueries} = useQueryErrorResetBoundary();

  const onRetry = useCallback(() => {
    resetQueries();
    reset();
  }, [resetQueries, reset]);

  return (
    <PopoverLayout>
      <Card size='3'>
        {/* Inset side='all' drops the Card padding so each ErrorPage section owns its spacing and
            the debug divider bleeds to the card edge, respecting the radius. */}
        <Inset side='all'>
          <ErrorPage error={error} errorInfo={errorInfo} reset={onRetry} />
        </Inset>
      </Card>
    </PopoverLayout>
  );
};
