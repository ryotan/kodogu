import {useMemo} from 'react';

import type {FrameView} from './FrameList';
import {FrameList} from './FrameList';
import {parseComponentStack} from './parseComponentStack';

type ComponentStackProps = {
  stack: string;
};

// Dev-only. Renders the React component stack as a frame list. The label, separator and collapsing
// around it are owned by ErrorDebugSection so every section reads uniformly. Reached only through
// ErrorDebugSection, which is gated behind __DEV_TOOLS__ + lazy().
export function ComponentStack({stack}: ComponentStackProps) {
  const frames = useMemo(() => parseComponentStack(stack), [stack]);

  const visibleFrames: FrameView[] = frames.map(f => ({
    name: f.component,
    ...(f.location !== undefined && {location: f.location}),
  }));

  return <FrameList emptyMessage='No frames could be parsed' frames={visibleFrames} />;
}
