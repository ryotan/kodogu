import type {ErrorComponentProps} from '@tanstack/react-router';

import {AppErrorComponent} from '#components/AppErrorComponent';
import {useAutoResize} from '#hooks/useAutoResize';

/**
 * Route-level error page for the popover window.
 * Wraps AppErrorComponent with auto-resize so the window height adjusts to the error content.
 */
export const AppErrorPage = (props: ErrorComponentProps) => {
  const {resizeCallbackRef} = useAutoResize();
  return (
    <div style={{height: '100dvh'}}>
      <div ref={resizeCallbackRef}>
        <AppErrorComponent {...props} />
      </div>
    </div>
  );
};
