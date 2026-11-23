import type {ReactNode} from 'react';

import {ThemePanel} from '@radix-ui/themes';

export function RadixThemePanel(): ReactNode {
  if (!import.meta.env.DEV) {
    return null;
  }

  return <ThemePanel defaultOpen={false} />;
}
