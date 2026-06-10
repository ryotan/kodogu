import {ReactQueryDevtools} from '@tanstack/react-query-devtools';
import {TanStackRouterDevtools} from '@tanstack/react-router-devtools';

// Dev-only. Consumers must use __DEV_TOOLS__ + lazy() to exclude this module from non-dev bundles:
//   const TanStackTools = __DEV_TOOLS__
//     ? lazy(() => import('#(dev)/TanStackTools').then(m => ({default: m.TanStackTools})))
//     : () => null;
export const TanStackTools = () => (
  <>
    <TanStackRouterDevtools />
    <ReactQueryDevtools />
  </>
);
