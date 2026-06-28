import {useMemo} from 'react';

import {Box} from '@radix-ui/themes';

import StackTraceParser from 'error-stack-parser';

import type {FrameView} from './FrameList';
import {FrameList} from './FrameList';
import {formatOriginLocation, isExternalLocation} from './frameLocation';

type StackTraceProps = {
  error: Error;
  /** Whether external (node_modules/.vite) frames are shown. Owned by the debug section so one
   *  switch controls every stack in the tree. */
  showExternal: boolean;
};

// Dev-only. Renders an error's parsed stack frames; the collapsing is owned by ErrorTreeNode (the
// node's `name: message` header is the toggle), so this is just the content. Reached only through
// ErrorDebugSection, which is gated behind __DEV_TOOLS__ + lazy().
export const StackTrace = ({error, showExternal}: StackTraceProps) => {
  const allFrames = useMemo(() => {
    try {
      return StackTraceParser.parse(error);
    } catch {
      return [];
    }
  }, [error]);

  const visibleFrames: FrameView[] = (
    showExternal ? allFrames : allFrames.filter(f => !isExternalLocation(f.fileName ?? ''))
  ).map(frame => {
    const fileName = frame.fileName ?? '(unknown)';
    const isExternal = isExternalLocation(fileName);
    return {
      badge: {text: isExternal ? 'ext' : 'app', color: isExternal ? 'gray' : 'blue'},
      name: frame.functionName ?? '<anonymous>',
      location: formatOriginLocation(`${fileName}:${frame.lineNumber ?? '?'}:0`),
      isExternal,
    };
  });

  return (
    <Box pb='1'>
      <FrameList emptyMessage='No internal frames found' frames={visibleFrames} />
    </Box>
  );
};
