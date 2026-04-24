import {useState} from 'react';

import {Badge, Box, Button, Card, Code, Flex, Separator, Text} from '@radix-ui/themes';

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

const StackTraceInspectorDev = () => {
  const [captured, setCaptured] = useState<Error | null>(null);

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
    <Box bottom='0' left='0' maxHeight='50dvh' position='fixed' right='0'>
      <Card size='1'>
        <Flex direction='column' gap='2'>
          {/* Header — always visible */}
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

          {/* Scrollable content */}
          {captured && (
            <Box maxHeight='40dvh' overflowY='auto'>
              <Flex direction='column' gap='2'>
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

                <Separator size='4' />

                {/* Raw stack */}
                <Text size='1' weight='bold'>
                  Raw error.stack
                </Text>
                <Card size='1' variant='surface'>
                  <Code asChild size='1' style={{whiteSpace: 'pre-wrap', wordBreak: 'break-all'}}>
                    <pre>{captured.stack}</pre>
                  </Code>
                </Card>
              </Flex>
            </Box>
          )}
        </Flex>
      </Card>
    </Box>
  );
};

// Dev-only. Consumers must use import.meta.env.DEV + lazy() to exclude this module from the production bundle:
//   const StackTraceInspector = import.meta.env.DEV
//     ? lazy(() => import('#(dev)/StackTraceInspector').then(m => ({default: m.StackTraceInspector})))
//     : () => null;
export const StackTraceInspector = import.meta.env.DEV ? StackTraceInspectorDev : () => null;
