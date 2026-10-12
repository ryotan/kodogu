import {QueryClient} from '@tanstack/react-query';

export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        // Tauri IPC is local — browser online/offline events are irrelevant.
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
      },
    },
  });
}
