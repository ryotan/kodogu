import {ThemePanel} from '@radix-ui/themes';

// Dev-only. Consumers must use __DEV_TOOLS__ + lazy() to exclude this module from non-dev bundles:
//   const RadixThemePanel = __DEV_TOOLS__
//     ? lazy(() => import('#(dev)/RadixThemePanel').then(m => ({default: m.RadixThemePanel})))
//     : () => null;
export const RadixThemePanel = () => <ThemePanel defaultOpen={false} />;
