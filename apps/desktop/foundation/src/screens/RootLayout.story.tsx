import type {ReactElement} from 'react';
import {useEffect} from 'react';

import {QueryClientProvider} from '@tanstack/react-query';
import {
  createMemoryHistory,
  createRootRouteWithContext,
  createRoute,
  createRouter,
  RouterProvider,
} from '@tanstack/react-router';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {create} from 'mutative';
import {expect, waitFor} from 'storybook/test';

import {appConfigFixture} from '#commands/appConfig.fixture';
import type {AppConfig} from '#config';
import {CssColor} from '#config';
import {createQueryClient} from '#createQueryClient';
import {appConfigQuery} from '#hooks/useAppConfig';
import {useStable} from '#hooks/useStable';

import type {RouterContext} from './__root';
import {RootLayout} from './__root';

const updatedConfig = create(appConfigFixture, draft => {
  draft.popover.appearance.surfaceColor = CssColor('rgba(1, 2, 3, 1)');
});
const tokenNames = [
  '--kodogu-surface-color',
  '--kodogu-border-color',
  '--kodogu-backdrop-filter',
  '--kodogu-font-family',
  '--kodogu-header-height',
  '--kodogu-card-list-padding',
] as const;

function RootExample(): ReactElement {
  const queryClient = useStable(() => {
    const client = createQueryClient();
    client.setQueryData(appConfigQuery.queryKey, appConfigFixture);
    return client;
  });
  const router = useStable(() => {
    const root = createRootRouteWithContext<RouterContext>()({component: RootLayout});
    const child = createRoute({getParentRoute: () => root, path: '/', component: () => <h1>Nested route</h1>});
    return createRouter({
      routeTree: root.addChildren([child]),
      history: createMemoryHistory({initialEntries: ['/']}),
      context: {queryClient},
    });
  });
  useEffect(() => () => queryClient.clear(), [queryClient]);

  return (
    <QueryClientProvider client={queryClient}>
      <button onClick={() => queryClient.setQueryData(appConfigQuery.queryKey, updatedConfig)} type='button'>
        Change theme
      </button>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}

const meta = {
  component: RootExample,
  beforeEach: () => {
    const root = document.documentElement;
    const previous = tokenNames.map(name => ({
      name,
      value: root.style.getPropertyValue(name),
      priority: root.style.getPropertyPriority(name),
    }));
    return () => {
      for (const {name, value, priority} of previous) {
        if (value) {
          root.style.setProperty(name, value, priority);
        } else {
          root.style.removeProperty(name);
        }
      }
    };
  },
} satisfies Meta<typeof RootExample>;

export default meta;
type Story = StoryObj<typeof meta>;

async function expectDesignTokens(config: AppConfig): Promise<void> {
  const {appearance, layout} = config.popover;
  await waitFor(() =>
    expect(
      Object.fromEntries(tokenNames.map(name => [name, document.documentElement.style.getPropertyValue(name)])),
    ).toEqual({
      '--kodogu-surface-color': appearance.surfaceColor,
      '--kodogu-border-color': appearance.borderColor,
      '--kodogu-backdrop-filter': appearance.backdropFilter,
      '--kodogu-font-family': appearance.fontFamily,
      '--kodogu-header-height': layout.headerHeight,
      '--kodogu-card-list-padding': layout.cardListPadding,
    }),
  );
}

export const RendersChildAndDesignTokens: Story = {
  play: async ({canvas}) => {
    await expect(await canvas.findByRole('heading', {name: 'Nested route'})).toBeVisible();
    await expectDesignTokens(appConfigFixture);
  },
};

export const UpdatesDesignTokensFromCache: Story = {
  play: async ({canvas, userEvent}) => {
    await expect(await canvas.findByRole('heading', {name: 'Nested route'})).toBeVisible();
    await expectDesignTokens(appConfigFixture);
    await userEvent.click(canvas.getByRole('button', {name: 'Change theme'}));
    await expectDesignTokens(updatedConfig);
    await expect(canvas.getByRole('heading', {name: 'Nested route'})).toBeVisible();
  },
};
