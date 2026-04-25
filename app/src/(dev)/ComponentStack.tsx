// app/src/(dev)/ComponentStack.tsx
import {useState} from 'react';

import {Code, Flex} from '@radix-ui/themes';

type ComponentStackProps = {
  stack: string;
};

// Dev-only. Consumers must use __DEV_TOOLS__ + lazy() to exclude this module from non-dev bundles:
//   const ComponentStack = __DEV_TOOLS__
//     ? lazy(() => import('#(dev)/ComponentStack').then(m => ({default: m.ComponentStack})))
//     : null;
export const ComponentStack = ({stack}: ComponentStackProps) => {
  const [expanded, setExpanded] = useState(false);
  return (
    <Flex direction='column' gap='1'>
      <button onClick={() => setExpanded(v => !v)} type='button'>
        Component stack
      </button>
      {expanded && (
        // Code asChild preserves whitespace (pre) and applies monospace styling
        <Code asChild size='1'>
          <pre>{stack}</pre>
        </Code>
      )}
    </Flex>
  );
};
