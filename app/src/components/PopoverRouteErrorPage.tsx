import type {ReactElement} from 'react';

import type {ErrorComponentProps} from '@tanstack/react-router';

import {AppErrorPage} from '#components/AppErrorPage';
import {useRouteErrorReset} from '#hooks/useRouteErrorReset';

export function PopoverRouteErrorPage({error}: ErrorComponentProps): ReactElement {
  const reset = useRouteErrorReset();
  return <AppErrorPage error={error} reset={reset} />;
}
