import {ReactQueryDevtools} from '@tanstack/react-query-devtools';
import {TanStackRouterDevtools} from '@tanstack/react-router-devtools';

export const TanStackTools = () => {
  if (import.meta.env.DEV) {
    return null;
  }

  return (
    <>
      <TanStackRouterDevtools />
      <ReactQueryDevtools />
    </>
  );
};
