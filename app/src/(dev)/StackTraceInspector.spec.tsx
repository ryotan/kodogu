import {afterEach, describe, expect, it, vi} from 'vitest';
import {render} from 'vitest-browser-react';

import {StackTraceInspector} from './StackTraceInspector';

describe('StackTraceInspector', () => {
  describe('in production mode (DEV = false)', () => {
    afterEach(() => {
      vi.unstubAllEnvs();
    });

    it('renders nothing', async () => {
      vi.stubEnv('DEV', false);
      const {container} = await render(<StackTraceInspector />);
      expect(container.firstChild).toBeNull();
    });
  });

  describe('in development mode (DEV = true)', () => {
    it('renders the inspector panel', async () => {
      const {page} = await import('vitest/browser');
      await render(<StackTraceInspector />);
      await expect.element(page.getByText('Stack Trace Inspector')).toBeVisible();
    });
  });
});
