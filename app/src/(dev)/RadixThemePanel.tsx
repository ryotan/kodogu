import {ThemePanel} from '@radix-ui/themes';

const RadixThemePanelDev = () => <ThemePanel defaultOpen={false} />;

// Dev-only. Consumers must use import.meta.env.DEV + lazy() to exclude this module from the production bundle:
//   const RadixThemePanel = import.meta.env.DEV
//     ? lazy(() => import('#(dev)/RadixThemePanel').then(m => ({default: m.RadixThemePanel})))
//     : () => null;
export const RadixThemePanel = import.meta.env.DEV ? RadixThemePanelDev : () => null;
