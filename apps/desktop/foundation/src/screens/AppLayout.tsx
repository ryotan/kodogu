import type {ReactElement} from 'react';

import {Outlet} from '@tanstack/react-router';

import {Theme} from '@radix-ui/themes';

import {RadixThemePanel} from '#(dev)/RadixThemePanel';

export function AppLayout(): ReactElement {
  return (
    <Theme panelBackground='solid'>
      <Outlet />
      <RadixThemePanel />
    </Theme>
  );
}
