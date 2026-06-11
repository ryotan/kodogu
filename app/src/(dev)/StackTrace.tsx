import {useMemo, useState} from 'react';

import {Button, Flex} from '@radix-ui/themes';

import StackTraceParser from 'error-stack-parser';

import {formatOriginLocation, isExternalLocation} from '#shared/core/errors/stackTrace';

import type {FrameView} from './FrameList';
import {FrameList} from './FrameList';

type StackTraceProps = {
  error: Error;
};

// Dev-only. Consumers must use __DEV_TOOLS__ + lazy() to exclude this module from non-dev bundles:
//   const StackTrace = __DEV_TOOLS__
//     ? lazy(() => import('#(dev)/StackTrace').then(m => ({default: m.StackTrace})))
//     : null;
export const StackTrace = ({error}: StackTraceProps) => {
  const [expanded, setExpanded] = useState(false);
  const [showExternal, setShowExternal] = useState(false);

  const allFrames = useMemo(() => {
    try {
      return StackTraceParser.parse(error);
    } catch {
      return [];
    }
  }, [error]);

  if (allFrames.length === 0) {
    return null;
  }

  const visibleFrames: FrameView[] = (
    showExternal ? allFrames : allFrames.filter(f => !isExternalLocation(f.fileName ?? ''))
  ).map(frame => {
    const fileName = frame.fileName ?? '(unknown)';
    const isExternal = isExternalLocation(fileName);
    return {
      badge: {text: isExternal ? 'ext' : 'app', color: isExternal ? 'gray' : 'blue'},
      name: frame.functionName ?? '<anonymous>',
      location: formatOriginLocation(`${fileName}:${frame.lineNumber ?? '?'}:0`),
    };
  });

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
      {expanded && <FrameList emptyMessage='No internal frames found' frames={visibleFrames} />}
    </Flex>
  );
};
