// app/src/(dev)/StackTrace.tsx
import {useMemo, useState} from 'react';

import {Badge, Button, Card, Code, Flex, Text} from '@radix-ui/themes';

import StackTraceParser from 'error-stack-parser';

import {formatOriginLocation, isExternalLocation} from '#shared/core/errors/stackTrace';

type StackTraceProps = {
  error: Error;
};

// Dev-only. Consumers must use import.meta.env.DEV + lazy() to exclude this module from the production bundle:
//   const StackTrace = import.meta.env.DEV
//     ? lazy(() => import('#(dev)/StackTrace').then(m => ({default: m.StackTrace})))
//     : null;
export const StackTrace = ({error}: StackTraceProps) => {
  const [expanded, setExpanded] = useState(false);
  const [showExternal, setShowExternal] = useState(false);

  const frames = useMemo(() => {
    try {
      return StackTraceParser.parse(error);
    } catch {
      return [];
    }
  }, [error]);

  if (frames.length === 0) {
    return null;
  }

  const visibleFrames = showExternal ? frames : frames.filter(f => !isExternalLocation(f.fileName ?? ''));

  return (
    <Flex direction='column' gap='2'>
      <Flex align='center' gap='2'>
        <Button onClick={() => setExpanded(v => !v)} size='1' variant='outline'>
          {expanded ? 'Hide stack trace' : 'Show stack trace'}
        </Button>
        {expanded && (
          <Button onClick={() => setShowExternal(v => !v)} size='1' variant='ghost'>
            {showExternal ? 'Internal only' : 'Show all'}
          </Button>
        )}
      </Flex>

      {expanded && (
        <Card size='1'>
          <Flex direction='column' gap='1'>
            {visibleFrames.map((frame, i) => {
              const location = frame.fileName ?? '(unknown)';
              const isExternal = isExternalLocation(location);
              const shortLocation = formatOriginLocation(`${location}:${frame.lineNumber ?? '?'}:0`);
              return (
                <Flex align='baseline' gap='2' key={i}>
                  <Badge color={isExternal ? 'gray' : 'blue'} size='1' variant='soft'>
                    {isExternal ? 'ext' : 'app'}
                  </Badge>
                  <Flex direction='column' minWidth='0'>
                    <Code size='1' truncate>
                      {frame.functionName ?? '<anonymous>'}
                    </Code>
                    <Text color='gray' size='1' truncate>
                      {shortLocation}
                    </Text>
                  </Flex>
                </Flex>
              );
            })}
            {visibleFrames.length === 0 && (
              <Text color='gray' size='1'>
                No internal frames found
              </Text>
            )}
          </Flex>
        </Card>
      )}
    </Flex>
  );
};
