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

  if (!import.meta.env.DEV) {
    return null;
  }

  const throwAndCapture = () => {
    try {
      level1();
    } catch (e) {
      if (e instanceof Error) {
        setCaptured(e);
      }
    }
  };

  const frames = captured
    ? (captured.stack?.split('\n').slice(1) ?? []).map(line => ({line, frame: parseFrame(line)}))
    : [];

  return (
    <Box
      p='3'
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
      <Flex direction='column' gap='2'>
        <Flex align='center' gap='2'>
          <Text size='1' weight='bold'>
            Stack Trace Inspector
          </Text>
          <Button onClick={throwAndCapture} size='1' variant='soft'>
            Throw Test Error
          </Button>
          {captured && (
            <Button color='red' onClick={() => setCaptured(null)} size='1' variant='ghost'>
              Clear
            </Button>
          )}
        </Flex>

        {captured && (
          <>
            <Separator size='4' />

            {/* Raw stack */}
            <Text size='1' weight='bold'>
              Raw error.stack
            </Text>
            <Box
              p='2'
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

            <Separator size='4' />

            {/* Parsed frames */}
            <Text size='1' weight='bold'>
              Parsed frames (isExternal判定)
            </Text>
            {frames.map(({line, frame}, i) => (
              <Flex align='start' gap='2' key={i}>
                {frame ? (
                  <>
                    <Badge color={frame.isExternal ? 'gray' : 'green'} size='1'>
                      {frame.isExternal ? 'external' : 'internal'}
                    </Badge>
                    <Flex direction='column' gap='0'>
                      <Code size='1'>{frame.name}</Code>
                      <Code color='gray' size='1'>
                        {formatOriginLocation(frame.location)}
                      </Code>
                      <Code
                        color={
                          frame.location.includes('node_modules') || frame.location.includes('.vite/deps')
                            ? 'orange'
                            : 'gray'
                        }
                        size='1'
                      >
                        {frame.location}
                      </Code>
                    </Flex>
                  </>
                ) : (
                  <>
                    <Badge color='red' size='1'>
                      parse failed
                    </Badge>
                    <Code color='gray' size='1'>
                      {line}
                    </Code>
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
