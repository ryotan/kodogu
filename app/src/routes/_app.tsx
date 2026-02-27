import {createFileRoute, Outlet} from '@tanstack/react-router';

import {Theme} from '@radix-ui/themes';

import {RadixThemePanel} from '#(dev)/RadixThemePanel';

export const Route = createFileRoute('/_app')({
  component: () => (
    <Theme panelBackground='solid'>
      <Outlet />
      <RadixThemePanel />
    </Theme>
  ),
});
