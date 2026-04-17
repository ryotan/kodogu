import {describe, expect, it} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {StackTraceInspector} from './StackTraceInspector';

describe('StackTraceInspector', () => {
  it('renders the inspector panel in development mode', async () => {
    await render(<StackTraceInspector />);
    await expect.element(page.getByText('Stack Trace Inspector')).toBeVisible();
  });
});
