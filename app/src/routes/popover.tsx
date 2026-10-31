import type {ReactElement} from 'react';

import {createFileRoute} from '@tanstack/react-router';
import './popover.css';

import {PlusIcon} from '@radix-ui/react-icons';
import {Flex, IconButton, Theme} from '@radix-ui/themes';

import {StubCardList, StubHeader} from '#(prototype)/popover-content';
import {useAutoResize} from '#hooks/useAutoResize';

// =============================================================================
// Popover shell — window management and layout skeleton
// =============================================================================

function PopoverLayout(): ReactElement {
  const {resizeCallbackRef} = useAutoResize();

  return (
    <Theme appearance='dark' hasBackground={false}>
      <div className='popover-shell'>
        {/* Header (drag region) */}
        <Flex align='center' className='popover-header' data-tauri-drag-region='' justify='between'>
          <StubHeader />
          <IconButton mr='3' size='1' variant='ghost'>
            <PlusIcon />
          </IconButton>
        </Flex>

        <div className='card-list-scroll-area'>
          <div className='card-list-content' ref={resizeCallbackRef}>
            <StubCardList />
          </div>
        </div>
      </div>
    </Theme>
  );
}

export const Route = createFileRoute('/popover')({
  component: PopoverLayout,
});
