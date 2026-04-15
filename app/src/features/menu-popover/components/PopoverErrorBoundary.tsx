import type {ReactNode} from 'react';

import {QueryErrorResetBoundary} from '@tanstack/react-query';

import type {FallbackProps} from 'react-error-boundary';
import {ErrorBoundary} from 'react-error-boundary';

import {PopoverAppErrorPage} from './PopoverAppErrorPage';

// Module-scope function — stable reference, no useCallback needed
const fallbackRender = ({error, resetErrorBoundary}: FallbackProps) => (
  <PopoverAppErrorPage error={error instanceof Error ? error : new Error(String(error))} reset={resetErrorBoundary} />
);

export const PopoverErrorBoundary = ({children}: {children: ReactNode}) => (
  <QueryErrorResetBoundary>
    <ErrorBoundary fallbackRender={fallbackRender}>{children}</ErrorBoundary>
  </QueryErrorResetBoundary>
);
