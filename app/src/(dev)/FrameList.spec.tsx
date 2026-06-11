import {describe, expect, it} from 'vitest';
import {page} from 'vitest/browser';
import {render} from 'vitest-browser-react';

import {Theme} from '@radix-ui/themes';

import type {FrameView} from './FrameList';
import {FrameList} from './FrameList';

const wrap = (frames: FrameView[], emptyMessage = 'No frames') =>
  render(
    <Theme>
      <FrameList emptyMessage={emptyMessage} frames={frames} />
    </Theme>,
  );

describe('FrameList', () => {
  it('renders the empty message when there are no frames', async () => {
    await wrap([], 'No frames available');
    await expect.element(page.getByText('No frames available')).toBeVisible();
  });

  it('renders a frame name', async () => {
    await wrap([{name: 'MyComponent'}]);
    await expect.element(page.getByText('MyComponent')).toBeVisible();
  });

  it('renders the optional location below the name', async () => {
    await wrap([{name: 'MyComponent', location: 'src/foo.tsx:42:18'}]);
    await expect.element(page.getByText('MyComponent')).toBeVisible();
    await expect.element(page.getByText('src/foo.tsx:42:18')).toBeVisible();
  });

  it('renders the optional badge', async () => {
    await wrap([{badge: {text: 'app', color: 'blue'}, name: 'MyComponent'}]);
    await expect.element(page.getByText('app')).toBeVisible();
    await expect.element(page.getByText('MyComponent')).toBeVisible();
  });

  it('renders multiple frames in order', async () => {
    await wrap([{name: 'First'}, {name: 'Second'}, {name: 'Third'}]);
    await expect.element(page.getByText('First')).toBeVisible();
    await expect.element(page.getByText('Second')).toBeVisible();
    await expect.element(page.getByText('Third')).toBeVisible();
  });
});
