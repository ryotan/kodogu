/** @jsxRuntime classic */
// The Storybook manager bundle uses the classic JSX runtime, so `React` must be in scope. The
// pragma above tells tsc the same; biome doesn't read the pragma, hence the ignore.
// biome-ignore lint/correctness/noUnusedImports: required in scope by the classic JSX runtime.
import React from 'react';

import {BrowserIcon} from '@storybook/icons';
import {Select} from 'storybook/internal/components';
import {addons, types, useGlobals} from 'storybook/manager-api';

import {SURFACE_GLOBAL_KEY, SURFACE_OPTIONS} from './preview/surfaces';

const TOOL_ID = 'kodogu/surface';

// Custom toolbar for the `surface` global, modeled on Storybook's own Backgrounds tool. The
// `Select` component's `onReset` sets the global to `undefined`, which truly clears it — the
// selection returns to none and the value drops out of the URL — instead of leaving an empty-string
// sentinel selected (which a declarative `globalTypes.toolbar` item list can only do).
function SurfaceTool() {
  const [globals, updateGlobals] = useGlobals();
  const selected = globals[SURFACE_GLOBAL_KEY] as string | undefined;

  return (
    <Select
      ariaLabel='Surface'
      defaultOptions={selected}
      icon={<BrowserIcon />}
      onReset={() => updateGlobals({[SURFACE_GLOBAL_KEY]: undefined})}
      onSelect={value => updateGlobals({[SURFACE_GLOBAL_KEY]: value})}
      options={SURFACE_OPTIONS.map(option => ({value: option.value, title: option.title}))}
      resetLabel='Reset surface'
      tooltip='Production surface'
    />
  );
}

addons.register(TOOL_ID, () => {
  addons.add(TOOL_ID, {
    type: types.TOOL,
    title: 'Surface',
    // Show on stories and docs, like the built-in toolbar tools.
    match: ({tabId, viewMode}) => !tabId && (viewMode === 'story' || viewMode === 'docs'),
    render: SurfaceTool,
  });
});
