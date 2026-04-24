import type {ReactNode} from 'react';

import {Theme} from '@radix-ui/themes';

import {useAutoResize} from '#hooks/useAutoResize';

type PopoverLayoutProps = {
  children: ReactNode;
};

export const PopoverLayout = ({children}: PopoverLayoutProps) => {
  const {resizeCallbackRef} = useAutoResize();
  return (
    <Theme appearance='dark' hasBackground={false}>
      <div className='popover-shell'>
        <div className='card-list-scroll-area'>
          <div className='card-list-content' ref={resizeCallbackRef}>
            {children}
          </div>
        </div>
      </div>
    </Theme>
  );
};
