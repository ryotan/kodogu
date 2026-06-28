import {lazy, Suspense} from 'react';

import {Box, Button, Flex} from '@radix-ui/themes';

import './errors.css';

import type {ComponentErrorInfo} from './ComponentErrorInfo';
import {ErrorDisplay} from './ErrorDisplay';

// Dev-only — Vite replaces `__DEV_TOOLS__` literally at build time, so the
// import is eliminated from non-dev bundles together with `error-stack-parser`
// and the rest of the dev-stack rendering chain.
const ErrorDebugSection = __DEV_TOOLS__
  ? lazy(() => import('#(dev)/ErrorDebugSection').then(m => ({default: m.ErrorDebugSection})))
  : null;

type ErrorPageProps = {
  error: Error;
  /** React ErrorBoundary's `errorInfo` (carries `componentStack`). */
  errorInfo?: ComponentErrorInfo;
  /** Provided → render the retry button; omitted → no retry affordance. */
  reset?: () => void;
};

/**
 * Framing-agnostic page body for an error: the user-facing {@link ErrorDisplay} + optional retry
 * button + dev-only debug section, as two stacked sections split by a hairline divider. It imposes
 * no surface, card, or width — the caller frames it, e.g.
 * `<Card><Inset side='all'><ErrorPage /></Inset></Card>` (Inset lets the divider bleed to the card
 * edge, respecting the radius).
 *
 * Compatible with TanStack Router's `defaultErrorComponent` shape and
 * react-error-boundary's `fallbackRender` (which pass `error` + a reset).
 */
export const ErrorPage = ({error, errorInfo, reset}: ErrorPageProps) => (
  <>
    <Box px='5' py='5'>
      {/* Two-level Flex: outer gap separates the message group from the */}
      {/* retry button; inner gap is set inside ErrorDisplay. */}
      <Flex align='center' direction='column' gap='5'>
        <ErrorDisplay error={error} />
        {reset && (
          <Button color='gray' onClick={reset} size='2' variant='surface'>
            もう一度試す
          </Button>
        )}
      </Flex>
    </Box>
    {ErrorDebugSection && (
      <Box className='error-page-debug-divider'>
        <Suspense fallback={null}>
          <ErrorDebugSection error={error} errorInfo={errorInfo} />
        </Suspense>
      </Box>
    )}
  </>
);
