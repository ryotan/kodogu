import {useMemo, useState} from 'react';

import {Button, Flex} from '@radix-ui/themes';

import type {FrameView} from './FrameList';
import {FrameList} from './FrameList';
import {parseComponentStack} from './parseComponentStack';

type ComponentStackProps = {
  stack: string;
};

// Dev-only. Consumers must use __DEV_TOOLS__ + lazy() to exclude this module from non-dev bundles:
//   const ComponentStack = __DEV_TOOLS__
//     ? lazy(() => import('#(dev)/ComponentStack').then(m => ({default: m.ComponentStack})))
//     : null;
export const ComponentStack = ({stack}: ComponentStackProps) => {
  const [expanded, setExpanded] = useState(false);
  const frames = useMemo(() => parseComponentStack(stack), [stack]);

  if (frames.length === 0) {
    return null;
  }

  const visibleFrames: FrameView[] = frames.map(f => ({
    name: f.component,
    ...(f.location !== undefined && {location: f.location}),
  }));

  return (
    <Flex direction='column' gap='2'>
      <Button onClick={() => setExpanded(v => !v)} size='1' variant='outline'>
        {expanded ? 'Hide component stack' : 'Show component stack'}
      </Button>
      {expanded && <FrameList emptyMessage='No frames could be parsed' frames={visibleFrames} />}
    </Flex>
  );
};
