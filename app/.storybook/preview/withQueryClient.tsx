import type {ReactNode} from 'react';

import {QueryClient, QueryClientProvider, QueryErrorResetBoundary} from '@tanstack/react-query';

import type {Decorator} from '@storybook/react-vite';

import {defaultAppConfig} from '#commands/appConfig.fixture';
import {appConfigQuery} from '#hooks/useAppConfig';
import {useStable} from '#hooks/useStable';

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

// Each story gets a freshly minted QueryClient — pre-seeded with the default AppConfig fixture
// so route layouts reading it via useSuspenseQuery render synchronously, but isolated from every
// other story's cache by virtue of being scoped to this component's mount lifecycle.
// A future change will let stories override the seed via `parameters.query` and may swap this
// component's identity through a `key` prop to force re-mounting on parameter changes.
function ScopedQueryClientProvider({children}: {children: ReactNode}) {
  const queryClient = useStable(() => {
    // useStable wraps useState's lazy initializer (see its own docstring), which is exactly what
    // this rule wants — it just can't see through the custom hook to verify that.
    // eslint-disable-next-line @tanstack/query/stable-query-client
    const qc = new QueryClient();
    qc.setQueryData(appConfigQuery.queryKey, defaultAppConfig);
    return qc;
  });
  return (
    <QueryClientProvider client={queryClient}>
      <QueryErrorResetBoundary>{children}</QueryErrorResetBoundary>
    </QueryClientProvider>
  );
}

export const withQueryClient: Decorator = (Story, {parameters}) => {
  if (parameters.query?.skip === true) {
    return <Story />;
  }
  return (
    <ScopedQueryClientProvider>
      <Story />
    </ScopedQueryClientProvider>
  );
};
