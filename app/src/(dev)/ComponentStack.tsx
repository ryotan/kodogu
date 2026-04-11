// app/src/(dev)/ComponentStack.tsx
import {useState} from 'react';

import {Code, Flex} from '@radix-ui/themes';

interface ComponentStackProps {
  stack: string;
}

export default function ComponentStack({stack}: ComponentStackProps) {
  const [expanded, setExpanded] = useState(false);
  return (
    <Flex direction="column" gap="1">
      <button type="button" onClick={() => setExpanded(v => !v)}>
        Component stack
      </button>
      {expanded && (
        // Code asChild preserves whitespace (pre) and applies monospace styling
        <Code size="1" asChild>
          <pre>{stack}</pre>
        </Code>
      )}
    </Flex>
  );
}
