import type {ReactNode} from 'react';

import {QueryErrorResetBoundary} from '@tanstack/react-query';

import type {FallbackProps} from 'react-error-boundary';
import {ErrorBoundary} from 'react-error-boundary';

import {ApplicationError} from '#shared/core/errors/ApplicationError';

import {PopoverAppErrorPage} from './PopoverAppErrorPage';

/**
 * Error fallback for {@link PopoverErrorBoundary}. Adapts react-error-boundary's `FallbackProps`
 * to {@link PopoverAppErrorPage}: normalises a non-Error throw to an `ApplicationError` and maps
 * `resetErrorBoundary` → `reset`. Exported so it can be exercised on its own (story/spec).
 */
export const PopoverErrorBoundaryFallback = ({error, resetErrorBoundary}: FallbackProps) => (
  <PopoverAppErrorPage
    error={error instanceof Error ? error : new ApplicationError(String(error))}
    reset={resetErrorBoundary}
  />
);

export const PopoverErrorBoundary = ({children}: {children: ReactNode}) => (
  <QueryErrorResetBoundary>
    <ErrorBoundary fallbackRender={PopoverErrorBoundaryFallback}>{children}</ErrorBoundary>
  </QueryErrorResetBoundary>
);
