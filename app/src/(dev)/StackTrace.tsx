// app/src/(dev)/StackTrace.tsx
import {useMemo, useState} from 'react';

import StackTraceParser from 'error-stack-parser';
import {Code, Flex, Text} from '@radix-ui/themes';

import {isExternalLocation} from '#shared/core/errors/stackTrace';

interface StackTraceProps {
  error: Error;
}

export default function StackTrace({error}: StackTraceProps) {
  const [expanded, setExpanded] = useState(false);

  const frames = useMemo(() => {
    try {
      return StackTraceParser.parse(error);
    } catch {
      return [];
    }
  }, [error]);

  if (frames.length === 0) return null;

  return (
    <Flex direction="column" gap="1">
      <button type="button" onClick={() => setExpanded(v => !v)}>
        Stack trace
      </button>
      {expanded &&
        frames.map((frame, i) => {
          const isExternal = isExternalLocation(frame.fileName ?? '');
          return (
            <Text key={i} size="1" color={isExternal ? 'gray' : undefined}>
              <Code>{frame.functionName ?? '<anonymous>'}</Code>
              {' @ '}
              <Code>{frame.fileName ?? '(unknown)'}:{frame.lineNumber ?? '?'}</Code>
            </Text>
          );
        })}
    </Flex>
  );
}
