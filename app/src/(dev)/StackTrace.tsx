// app/src/(dev)/StackTrace.tsx
import {useMemo, useState} from 'react';

import {Code, Flex, Text} from '@radix-ui/themes';

import StackTraceParser from 'error-stack-parser';

import {isExternalLocation} from '#shared/core/errors/stackTrace';

type StackTraceProps = {
  error: Error;
};

// Dev-only. Consumers must use import.meta.env.DEV + lazy() to exclude this module from the production bundle:
//   const StackTrace = import.meta.env.DEV
//     ? lazy(() => import('#(dev)/StackTrace').then(m => ({default: m.StackTrace})))
//     : null;
export const StackTrace = ({error}: StackTraceProps) => {
  const [expanded, setExpanded] = useState(false);

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

  return (
    <Flex direction='column' gap='1'>
      <button onClick={() => setExpanded(v => !v)} type='button'>
        Stack trace
      </button>
      {expanded &&
        frames.map((frame, i) => {
          const isExternal = isExternalLocation(frame.fileName ?? '');
          return (
            <Text color={isExternal ? 'gray' : undefined} key={i} size='1'>
              <Code>{frame.functionName ?? '<anonymous>'}</Code> @{' '}
              <Code>
                {frame.fileName ?? '(unknown)'}:{frame.lineNumber ?? '?'}
              </Code>
            </Text>
          );
        })}
    </Flex>
  );
};
