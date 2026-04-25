import {ThemePanel} from '@radix-ui/themes';

const RadixThemePanelDev = () => <ThemePanel defaultOpen={false} />;

// Dev-only. Consumers must use __DEV_TOOLS__ + lazy() to exclude this module from non-dev bundles:
//   const RadixThemePanel = __DEV_TOOLS__
//     ? lazy(() => import('#(dev)/RadixThemePanel').then(m => ({default: m.RadixThemePanel})))
//     : () => null;
export const RadixThemePanel = __DEV_TOOLS__ ? RadixThemePanelDev : () => null;
