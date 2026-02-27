import {createFileRoute} from '@tanstack/react-router';
import './popover.css';

import {PlusIcon} from '@radix-ui/react-icons';
import {Flex, IconButton, Theme} from '@radix-ui/themes';

import {StubCardList, StubFooter, StubHeader} from '#(prototype)/popover-content';
import {useAutoResize} from '#hooks/useAutoResize';

// Shell-level design tokens (window chrome).
// Will migrate to app-wide settings infrastructure in M1.5.
const shell = {
  surface: 'rgba(28, 28, 30, 0.92)',
  border: 'rgba(255, 255, 255, 0.08)',
} as const;

// =============================================================================
// Popover shell — window management and layout skeleton
// =============================================================================

const PopoverLayout = () => {
  const {ref: contentRef} = useAutoResize();

  return (
    <Theme appearance='dark' hasBackground={false}>
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          height: '100dvh',
          background: shell.surface,
          backdropFilter: 'blur(24px) saturate(180%)',
          WebkitBackdropFilter: 'blur(24px) saturate(180%)',
          fontFamily: "-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif",
        }}
      >
        {/* Header (drag region) */}
        <Flex
          align='center'
          data-tauri-drag-region=''
          justify='between'
          style={{borderBottom: `1px solid ${shell.border}`, height: 40, flexShrink: 0}}
        >
          <StubHeader />
          <IconButton mr='3' size='1' variant='ghost'>
            <PlusIcon />
          </IconButton>
        </Flex>

        {/* Card list — scrolls when Rust-side max height cap is active */}
        <div style={{flex: 1, overflowY: 'auto', minHeight: 0}}>
          <div ref={contentRef} style={{padding: '6px 0'}}>
            <StubCardList />
          </div>
        </div>

        {/* Footer */}
        <div
          style={{
            borderTop: `1px solid ${shell.border}`,
            padding: '8px 14px',
            display: 'flex',
            justifyContent: 'flex-end',
            flexShrink: 0,
          }}
        >
          <StubFooter />
        </div>
      </div>
    </Theme>
  );
};

export const Route = createFileRoute('/popover')({
  component: PopoverLayout,
});
