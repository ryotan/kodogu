// app/src/shared/core/errors/ErrorDisplay.tsx
import {lazy, Suspense, useMemo} from 'react';

import {Flex, Text} from '@radix-ui/themes';

import {formatOriginLocation, parseStackTrace} from './stackTrace';

// (dev)/StackTrace uses error-stack-parser (devDependency) — lazy-load only in dev.
// Vite replaces import.meta.env.DEV with false at build time, eliminating this import.
const StackTrace = import.meta.env.DEV
  ? lazy(() => import('#(dev)/StackTrace').then(m => ({default: m.StackTrace})))
  : null;

const MAX_CAUSE_DEPTH = 10;

type ErrorDisplayProps = {
  error: Error;
  // No onReset — retry is the caller's responsibility (PopoverAppErrorPage)
  depth?: number;
};

export const ErrorDisplay = ({error, depth = 0}: ErrorDisplayProps) => {
  const {originFrame} = useMemo(() => {
    const frames = parseStackTrace(error.stack ?? '');
    return {originFrame: frames.find(f => !f.isExternal)};
  }, [error.stack]);

  return (
    <Flex direction='column' gap='2'>
      <Text weight='bold'>{error.name}</Text>
      <Text>{error.message}</Text>

      {originFrame && <Text size='1'>at {formatOriginLocation(originFrame.location)}</Text>}

      {StackTrace && (
        <Suspense fallback={null}>
          <StackTrace error={error} />
        </Suspense>
      )}

      {/* Cause: AggregateError — list parallel errors */}
      {error.cause instanceof AggregateError && (
        <Flex direction='column' gap='1'>
          {error.cause.message && <Text size='1'>{error.cause.message}</Text>}
          {depth < MAX_CAUSE_DEPTH &&
            error.cause.errors
              .filter((e): e is Error => e instanceof Error)
              .map((sub, i) => <ErrorDisplay depth={depth + 1} error={sub} key={i} />)}
        </Flex>
      )}

      {/* Cause: single Error — recurse into cause chain */}
      {error.cause instanceof Error && !(error.cause instanceof AggregateError) && depth < MAX_CAUSE_DEPTH && (
        <ErrorDisplay depth={depth + 1} error={error.cause} />
      )}
    </Flex>
  );
};
