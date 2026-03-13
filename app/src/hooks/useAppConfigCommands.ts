import {useCallback} from 'react';

import {useMutation, useQueryClient} from '@tanstack/react-query';

import {create} from 'mutative';

import type {AppConfig} from '#config';
import {setConfig} from '#config';
import {QueryKeys} from '#QueryKeys';

type ConfigRecipe = (draft: AppConfig) => void;

// Stable reference — avoids recreating useCallback/useMutation closures every render.
const queryKey = QueryKeys.appConfig();

/**
 * Returns mutation helpers for updating and persisting AppConfig.
 *
 * Usage:
 *   const {update} = useAppConfigCommands();
 *   update(draft => { draft.popover.appearance.surfaceColor = '#fff'; });
 */
export function useAppConfigCommands() {
  const queryClient = useQueryClient();

  const {mutate, isPending} = useMutation({
    mutationFn: setConfig,
    onMutate: async next => {
      await queryClient.cancelQueries({queryKey});
      const previous = queryClient.getQueryData<AppConfig>(queryKey);
      queryClient.setQueryData(queryKey, next);
      return {previous};
    },
    // Rollback to the previous value immediately, then re-fetch the authoritative state.
    onError: (_err, _next, context) => {
      if (context?.previous) {
        queryClient.setQueryData(queryKey, context.previous);
      }
      queryClient.invalidateQueries({queryKey});
    },
  });

  // Read from the cache at call time (not render time) so rapid consecutive updates chain correctly.
  const update = useCallback(
    (recipe: ConfigRecipe) => {
      // AppBootstrap guarantees AppConfig is loaded before the app renders,
      // so the cache is always populated at this point.
      const current = queryClient.getQueryData<AppConfig>(queryKey);
      if (!current) {
        throw new Error('update() called before AppConfig was loaded — this should be unreachable');
      }
      mutate(create(current, recipe));
    },
    [queryClient, mutate],
  );

  return {update, isPending};
}
