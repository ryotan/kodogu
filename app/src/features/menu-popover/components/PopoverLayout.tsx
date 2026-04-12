import type {ReactNode} from 'react';

import {Box, Flex, Theme} from '@radix-ui/themes';

import {useAutoResize} from '#hooks/useAutoResize';

interface PopoverLayoutProps {
  children: ReactNode;
}

export const PopoverLayout = ({children}: PopoverLayoutProps) => {
  const {resizeCallbackRef} = useAutoResize();
  return (
    <Theme appearance="dark" hasBackground={false}>
      <Flex direction="column" height="100dvh">
        <Box flexGrow="1" overflowY="auto" minHeight="0">
          <div ref={resizeCallbackRef}>{children}</div>
        </Box>
      </Flex>
    </Theme>
  );
};
