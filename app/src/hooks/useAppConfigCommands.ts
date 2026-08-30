import {useCallback} from 'react';

import {useMutation, useQueryClient} from '@tanstack/react-query';

import type {Draft} from 'mutative';
import {create} from 'mutative';

import {setConfig} from '#commands/appConfig';
import type {AppConfig} from '#config';
import {ApplicationError} from '#shared/core/errors/ApplicationError';

import {appConfigQuery} from './useAppConfig';

type ConfigRecipe = (draft: Draft<AppConfig>) => void;

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
      // Cancel any in-flight queries so they don't overwrite our optimistic data.
      // With staleTime: Infinity no background refetch is normally in progress,
      // but this guards against manual invalidation or future staleTime changes.
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
  });

  // Read from the cache at call time (not render time) so rapid consecutive updates chain correctly.
  const update = useCallback(
    (recipe: ConfigRecipe) => {
      // Root route's beforeLoad guarantees AppConfig is cached before any route renders,
      // so the cache is always populated at this point.
      const current = queryClient.getQueryData(appConfigQuery.queryKey);
      if (!current) {
        throw new ApplicationError('update() called before AppConfig was loaded — this should be unreachable');
      }
      mutate(create(current, recipe));
    },
    [queryClient, mutate],
  );

  return {update, isPending};
}
