import {ReactQueryDevtools} from '@tanstack/react-query-devtools';
import {TanStackRouterDevtools} from '@tanstack/react-router-devtools';

const TanStackToolsDev = () => (
  <>
    <TanStackRouterDevtools />
    <ReactQueryDevtools />
  </>
);

// Dev-only. Consumers must use import.meta.env.DEV + lazy() to exclude this module from the production bundle:
//   const TanStackTools = import.meta.env.DEV
//     ? lazy(() => import('#(dev)/TanStackTools').then(m => ({default: m.TanStackTools})))
//     : () => null;
export const TanStackTools = import.meta.env.DEV ? TanStackToolsDev : () => null;
