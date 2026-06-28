import {useMemo, useState} from 'react';

import {Box, Button, Flex, Separator, Text} from '@radix-ui/themes';

import type {ComponentErrorInfo} from '#shared/core/errors/ComponentErrorInfo';
import {Disclosure} from '#ui/Disclosure/Disclosure';

import './ErrorDebugSection.css';

import {buildErrorTree} from './buildErrorTree';
import {ComponentStack} from './ComponentStack';
import {ErrorTreeNode} from './ErrorTreeNode';

type Props = {
  error: Error;
  /** React ErrorBoundary's `errorInfo` (carries `componentStack`). */
  errorInfo?: ComponentErrorInfo;
};

// Dev-only. Consumers gate it behind `__DEV_TOOLS__` + `lazy()` so that the module (and the
// libraries it transitively pulls in) is dead-code eliminated from non-dev bundles:
//   const ErrorDebugSection = __DEV_TOOLS__
//     ? lazy(() => import('#(dev)/ErrorDebugSection').then(m => ({default: m.ErrorDebugSection})))
//     : null;
//
// 何で / どこで:
//   Debug info
//   ├─ Error            the caught error as a cause tree (name: message, frames, nested causes)
//   └─ Component stack   where in the React tree the error surfaced (one, shared)
export const ErrorDebugSection = ({error, errorInfo}: Props) => {
  // External-frame visibility is owned here so a single switch governs every stack in the tree.
  const [showExternal, setShowExternal] = useState(false);
  const tree = useMemo(() => buildErrorTree(error), [error]);

  return (
    <Disclosure.Root>
      <Disclosure.Trigger>
        <Flex align='center' gap='3' px='4' py='3'>
          <Box flexGrow='1'>
            <Caption>Debug info</Caption>
          </Box>
          <Disclosure.Chevron />
        </Flex>
      </Disclosure.Trigger>
      <Disclosure.Content>
        <Flex direction='column' gap='3' pb='3' px='4'>
          <Flex align='center' justify='between'>
            <Caption>Error</Caption>
            <Button color='gray' onClick={() => setShowExternal(v => !v)} size='1' variant='ghost'>
              {showExternal ? 'Internal only' : 'Show all'}
            </Button>
          </Flex>
          <ErrorTreeNode node={tree} showExternal={showExternal} />
          {errorInfo?.componentStack && (
            <>
              <Separator size='4' />
              {/* The component stack is one shared, secondary detail, so ErrorDebugSection owns its
                  labelled disclosure (unlike per-node stacks, which each own their own toggle). */}
              <Disclosure.Root>
                <Disclosure.Trigger>
                  <Flex align='center' gap='3' py='1'>
                    <Box flexGrow='1'>
                      <Caption>Component stack</Caption>
                    </Box>
                    <Disclosure.Chevron />
                  </Flex>
                </Disclosure.Trigger>
                <Disclosure.Content>
                  <Box pt='2'>
                    <ComponentStack stack={errorInfo.componentStack} />
                  </Box>
                </Disclosure.Content>
              </Disclosure.Root>
            </>
          )}
        </Flex>
      </Disclosure.Content>
    </Disclosure.Root>
  );
};

const Caption = ({children}: {children: string}) => (
  <Text as='div' className='error-debug-section__caption' color='gray' size='1' weight='medium'>
    {children}
  </Text>
);
