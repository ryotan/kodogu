import {useCallback} from 'react';

import {useMutation, useQueryClient} from '@tanstack/react-query';

import {create} from 'mutative';

import type {AppConfig} from '#config';
import {setConfig} from '#config';
import {useAppConfig} from '#hooks/useAppConfig';

type ConfigRecipe = (draft: AppConfig) => void;

/**
 * Returns mutation helpers for updating and persisting AppConfig.
 *
 * Usage:
 *   const {update} = useAppConfigCommands();
 *   update(draft => { draft.popover.appearance.surfaceColor = '#fff'; });
 */
export function useAppConfigCommands() {
  const queryClient = useQueryClient();
  const config = useAppConfig();

  // mutationFn is a pure server call — variables are the fully computed next state.
  // Destructure to get stable references for the useCallback dependency array.
  const {mutate, isPending} = useMutation({
    mutationFn: setConfig,
    onSuccess: (_, next) => queryClient.setQueryData(['app-config'], next),
  });

  const update = useCallback((recipe: ConfigRecipe) => mutate(create(config, recipe)), [config, mutate]);

  return {update, isPending};
}
