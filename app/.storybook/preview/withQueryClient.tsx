import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import type {Decorator} from '@storybook/react-vite';

import {defaultAppConfig} from '#commands/appConfig.fixture';
import {appConfigQuery} from '#hooks/useAppConfig';

declare module '@storybook/react-vite' {
  // biome-ignore lint/style/useConsistentTypeDefinitions: TypeScript module augmentation requires `interface`; `type` cannot extend an existing interface from another module.
  interface Parameters {
    /**
     * Story configuration consumed by the {@link withQueryClient} decorator.
     * Has no effect when the decorator is not applied to the story.
     */
    query?: {
      /** Render the story without wrapping it in QueryClientProvider/QueryErrorResetBoundary. */
      skip?: boolean;
    };
  }
}

// Module-scoped QueryClient: pre-seeded with the default AppConfig fixture so route layouts that
// read it via useSuspenseQuery render synchronously in stories. Per-story overrides (KDG-78) will
// extend this decorator to read seeds from `parameters.query` rather than the singleton cache.
const queryClient = new QueryClient();
queryClient.setQueryData(appConfigQuery.queryKey, defaultAppConfig);

export const withQueryClient: Decorator = (Story, {parameters}) => {
  if (parameters.query?.skip === true) {
    return <Story />;
  }
  return (
    <QueryClientProvider client={queryClient}>
      <QueryErrorResetBoundary>
        <Story />
      </QueryErrorResetBoundary>
    </QueryClientProvider>
  );
};
