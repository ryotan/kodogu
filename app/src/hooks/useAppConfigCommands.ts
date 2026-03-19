import {useCallback} from 'react';

import {useMutation, useQueryClient} from '@tanstack/react-query';

import {create} from 'mutative';

import {setConfig} from '#commands/appConfig';
import type {AppConfig} from '#config';

import {appConfigQuery} from './useAppConfig';

type ConfigRecipe = (draft: AppConfig) => void;

/**
 * Returns mutation helpers for updating and persisting AppConfig.
 *
 * Usage:
 *   const {update} = useAppConfigCommands();
 *   update(draft => { draft.popover.appearance.surfaceColor = CssColor('#fff'); });
 */
export function useAppConfigCommands() {
  const queryClient = useQueryClient();

  const {mutate, isPending} = useMutation({
    mutationFn: setConfig,
    scope: {id: 'app-config'},
    onMutate: async next => {
      // Currently a no-op (staleTime: Infinity prevents background refetches),
      // but guards against future staleTime changes.
      await queryClient.cancelQueries({queryKey: appConfigQuery.queryKey});
      const previous = queryClient.getQueryData(appConfigQuery.queryKey);
      queryClient.setQueryData(appConfigQuery.queryKey, next);
      return {previous};
    },
    onError: (err, _next, context) => {
      console.error('Failed to persist AppConfig', err);
      if (context?.previous !== undefined) {
        queryClient.setQueryData(appConfigQuery.queryKey, context.previous);
      }
    },
    onSettled: () => {
      return queryClient.invalidateQueries({queryKey: appConfigQuery.queryKey});
    },
  });

  // Read from the cache at call time (not render time) so rapid consecutive updates chain correctly.
  const update = useCallback(
    (recipe: ConfigRecipe) => {
      // AppBootstrap guarantees AppConfig is loaded before the app renders,
      // so the cache is always populated at this point.
      const current = queryClient.getQueryData(appConfigQuery.queryKey);
      if (!current) {
        throw new Error('update() called before AppConfig was loaded — this should be unreachable');
      }
      mutate(create(current, recipe));
    },
    [queryClient, mutate],
  );

  return {update, isPending};
}
