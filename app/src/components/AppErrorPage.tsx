import type {ErrorComponentProps} from '@tanstack/react-router';

import {AppErrorComponent} from '#components/AppErrorComponent';
import {useAutoResize} from '#hooks/useAutoResize';

/**
 * Route-level error page for the popover window.
 * Wraps AppErrorComponent with auto-resize so the window height adjusts to the error content.
 *
 * Layout follows the same scroll-area structure as popover.tsx so that useAutoResize
 * computes chrome height correctly.
 */
export const AppErrorPage = (props: ErrorComponentProps) => {
  const {resizeCallbackRef} = useAutoResize();
  return (
    <div style={{display: 'flex', flexDirection: 'column', height: '100dvh'}}>
      <div style={{flex: 1, overflowY: 'auto', minHeight: 0}}>
        <div ref={resizeCallbackRef}>
          <AppErrorComponent {...props} />
        </div>
      </div>
    </div>
  );
};
