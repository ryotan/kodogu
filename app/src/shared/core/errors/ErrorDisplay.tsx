// app/src/shared/core/errors/ErrorDisplay.tsx
import {lazy, Suspense, useMemo} from 'react';

import {Card, Code, Flex, Heading, Text} from '@radix-ui/themes';

import {formatOriginLocation, parseStackTrace} from './stackTrace';

// (dev)/StackTrace uses error-stack-parser (devDependency) — lazy-load only when dev tools are
// enabled. Vite replaces __DEV_TOOLS__ literally at build time, so the import is eliminated when
// the flag is false (production builds, Storybook, regular browser tests).
const StackTrace = __DEV_TOOLS__ ? lazy(() => import('#(dev)/StackTrace').then(m => ({default: m.StackTrace}))) : null;

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
      {depth === 0 ? (
        <Heading as='h2' size='3'>
          {error.message}
        </Heading>
      ) : (
        <Text size='2' weight='bold'>
          {error.message}
        </Text>
      )}

      {originFrame && (
        <Code color='gray' size='1' variant='ghost'>
          {error.name} at {formatOriginLocation(originFrame.location)}
        </Code>
      )}

      {StackTrace && (
        <Suspense fallback={null}>
          <StackTrace error={error} />
        </Suspense>
      )}

      {/* Cause: AggregateError — list parallel errors */}
      {error.cause instanceof AggregateError && depth < MAX_CAUSE_DEPTH && (
        <Card size='1'>
          <Flex direction='column' gap='2'>
            {error.cause.message && (
              <Text size='1' weight='bold'>
                {error.cause.message}
              </Text>
            )}
            {error.cause.errors
              .filter((e): e is Error => e instanceof Error)
              .map((sub, i) => (
                <ErrorDisplay depth={depth + 1} error={sub} key={i} />
              ))}
          </Flex>
        </Card>
      )}

      {/* Cause: single Error — recurse into cause chain */}
      {error.cause instanceof Error && !(error.cause instanceof AggregateError) && depth < MAX_CAUSE_DEPTH && (
        <Card size='1'>
          <ErrorDisplay depth={depth + 1} error={error.cause} />
        </Card>
      )}
    </Flex>
  );
};
