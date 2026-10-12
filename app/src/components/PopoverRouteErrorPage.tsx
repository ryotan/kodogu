import type {ErrorComponentProps} from '@tanstack/react-router';

import {AppErrorPage} from '#components/AppErrorPage';
import {useRouteErrorReset} from '#hooks/useRouteErrorReset';

export const PopoverRouteErrorPage = ({error}: ErrorComponentProps) => {
  const reset = useRouteErrorReset();
  return <AppErrorPage error={error} reset={reset} />;
};
