import {useState} from 'react';

import {Badge, Box, Button, Code, Flex, Separator, Text} from '@radix-ui/themes';

import {formatOriginLocation, parseFrame} from '#shared/core/errors/stackTrace';

// ---------------------------------------------------------------------------
// Helpers — call stack depth for realistic frames
// ---------------------------------------------------------------------------

function level3(): never {
  throw new Error('Stack trace inspector test error');
}

function level2(): never {
  return level3();
}

function level1(): never {
  return level2();
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const StackTraceInspector = () => {
  const [captured, setCaptured] = useState<Error | null>(null);

  if (!import.meta.env.DEV) return null;

  const throwAndCapture = () => {
    try {
      level1();
    } catch (e) {
      if (e instanceof Error) setCaptured(e);
    }
  };

  const frames = captured
    ? (captured.stack?.split('\n').slice(1) ?? [])
        .map(line => ({line, frame: parseFrame(line)}))
    : [];

  return (
    <Box
      p="3"
      style={{
        position: 'fixed',
        bottom: 0,
        left: 0,
        right: 0,
        background: 'var(--color-panel-solid)',
        borderTop: '1px solid var(--gray-6)',
        maxHeight: '50dvh',
        overflowY: 'auto',
        zIndex: 9999,
      }}
    >
      <Flex direction="column" gap="2">
        <Flex align="center" gap="2">
          <Text size="1" weight="bold">Stack Trace Inspector</Text>
          <Button size="1" variant="soft" onClick={throwAndCapture}>
            Throw Test Error
          </Button>
          {captured && (
            <Button size="1" variant="ghost" color="red" onClick={() => setCaptured(null)}>
              Clear
            </Button>
          )}
        </Flex>

        {captured && (
          <>
            <Separator size="4" />

            {/* Raw stack */}
            <Text size="1" weight="bold">Raw error.stack</Text>
            <Box
              p="2"
              style={{
                background: 'var(--gray-2)',
                borderRadius: 'var(--radius-2)',
                fontFamily: 'monospace',
                fontSize: '11px',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-all',
              }}
            >
              {captured.stack}
            </Box>

            <Separator size="4" />

            {/* Parsed frames */}
            <Text size="1" weight="bold">Parsed frames (isExternal判定)</Text>
            {frames.map(({line, frame}, i) => (
              <Flex key={i} gap="2" align="start">
                {frame ? (
                  <>
                    <Badge size="1" color={frame.isExternal ? 'gray' : 'green'}>
                      {frame.isExternal ? 'external' : 'internal'}
                    </Badge>
                    <Flex direction="column" gap="0">
                      <Code size="1">{frame.name}</Code>
                      <Code size="1" color="gray">{formatOriginLocation(frame.location)}</Code>
                      <Code size="1" color={frame.location.includes('node_modules') ? 'orange' : frame.location.includes('.vite/deps') ? 'orange' : 'gray'}>
                        {frame.location}
                      </Code>
                    </Flex>
                  </>
                ) : (
                  <>
                    <Badge size="1" color="red">parse failed</Badge>
                    <Code size="1" color="gray">{line}</Code>
                  </>
                )}
              </Flex>
            ))}
          </>
        )}
      </Flex>
    </Box>
  );
};
