import type {ReactElement, ReactNode} from 'react';
import {useCallback, useState} from 'react';

import type {QueryClient} from '@tanstack/react-query';
import {useMutation, useQueryClient} from '@tanstack/react-query';
import {batch, Store, useStore} from '@tanstack/react-store';

import {create} from 'mutative';

import {setConfig} from '#commands/appConfig';
import {ConfigurationCommandError} from '#commands/ConfigurationCommandError';
import type {AppConfig} from '#config';
import {reportError} from '#reportError';

import {appConfigQuery} from './useAppConfig';
import type {ConfigurationCommands, ConfigurationSubmission, UpdateBlock} from './useAppConfigCommands';
import {AppConfigCommandsContext} from './useAppConfigCommands';

type FailedWrite = Extract<ConfigurationSubmission, {status: 'failed'}>;

type WriteBatch = {blockedBy?: UpdateBlock; rejected?: FailedWrite};

type SettingsChanges = {
  readonly [Group in keyof AppConfig]: {
    readonly [Section in keyof AppConfig[Group]]: Partial<AppConfig[Group][Section]>;
  };
};

type SettingsWrite = {
  readonly configuration: AppConfig;
  readonly changes: SettingsChanges;
  readonly batch: WriteBatch;
};

type WriteResult =
  | {readonly status: 'saved'; readonly configuration: AppConfig}
  | FailedWrite
  | {readonly status: 'not_sent'; readonly blockedBy: UpdateBlock};

type SubmissionState =
  | Exclude<ConfigurationSubmission, {status: 'pending'}>
  | {
      readonly status: 'pending';
      readonly write: SettingsWrite;
    };

export function AppConfigCommandsProvider({children}: {readonly children: ReactNode}): ReactElement {
  const queryClient = useQueryClient();
  // eslint-disable-next-line react/hook-use-state -- The provider keeps one Store for its lifetime.
  const [submissions] = useState(() => new Store<SubmissionState>({status: 'idle'}));
  const current = useStore(submissions);
  const queryKey = appConfigQuery.queryKey;

  const {mutate} = useMutation<WriteResult, Error, SettingsWrite>({
    mutationFn: async write => {
      const confirmed = write.batch.blockedBy ?? confirmedSettings(queryClient);
      if (confirmed.status !== 'available') {
        return {status: 'not_sent', blockedBy: confirmed};
      }
      const configuration = applyChanges(confirmed.configuration, write.changes);
      try {
        await setConfig(configuration);
        return {status: 'saved', configuration};
      } catch (error) {
        if (!(error instanceof ConfigurationCommandError)) {
          throw error;
        }
        return {status: 'failed', configuration, error};
      }
    },
    scope: {id: 'app-config'},
    retry: false,
    networkMode: 'always',
    onSuccess: async (result, write) => {
      if (result.status === 'not_sent') {
        stopWrites(submissions, write, result.blockedBy);
        return;
      }
      if (result.status === 'saved') {
        await queryClient.cancelQueries({queryKey, exact: true});
        queryClient.setQueryData(queryKey, result.configuration);
        settle(submissions, write, write.batch.rejected ?? {status: 'idle'});
        return;
      }
      const {error} = result;
      if (error.unavailable) {
        const blockedBy: UpdateBlock = {status: 'unavailable', error};
        write.batch.blockedBy = blockedBy;
        await queryClient.cancelQueries({queryKey, exact: true});
        queryClient.getQueryCache().find({queryKey, exact: true})?.setState({
          status: 'error',
          error,
          isInvalidated: true,
          errorUpdatedAt: Date.now(),
        });
        stopWrites(submissions, write, blockedBy, result);
      } else {
        write.batch.rejected = result;
        settle(submissions, write, result);
      }
      reportError('configuration_write_failed', error);
    },
  });

  const update = useCallback<ConfigurationCommands['update']>(
    recipe => {
      const previous = submissions.state;
      const pending = previous.status === 'pending' ? previous.write : undefined;
      const confirmed = pending?.batch.blockedBy ?? confirmedSettings(queryClient);
      if (confirmed.status !== 'available') {
        return confirmed;
      }
      const previousConfiguration = pending?.configuration ?? confirmed.configuration;
      const configuration = create(previousConfiguration, recipe);
      const write = {
        configuration,
        changes: changesBetween(previousConfiguration, configuration),
        batch: pending?.batch ?? {},
      };
      batch(() => {
        submissions.setState(() => ({status: 'pending', write}));
        mutate(write);
      });
      return {status: 'submitted'};
    },
    [mutate, queryClient, submissions],
  );

  const submission: ConfigurationSubmission =
    current.status === 'pending' ? {status: 'pending', configuration: current.write.configuration} : current;
  return <AppConfigCommandsContext value={{update, submission}}>{children}</AppConfigCommandsContext>;
}

function changedFields<T extends object>(before: T, after: T): Partial<T> {
  const changes: {-readonly [Field in keyof T]?: T[Field]} = {};
  for (const field in after) {
    if (Object.getOwnPropertyDescriptor(after, field) && !Object.is(before[field], after[field])) {
      changes[field] = after[field];
    }
  }
  return changes;
}

function changesBetween(before: AppConfig, after: AppConfig): SettingsChanges {
  return {
    popover: {
      behavior: changedFields(before.popover.behavior, after.popover.behavior),
      layout: changedFields(before.popover.layout, after.popover.layout),
      appearance: changedFields(before.popover.appearance, after.popover.appearance),
    },
  };
}

function applyChanges(confirmed: AppConfig, changes: SettingsChanges): AppConfig {
  return {
    popover: {
      behavior: {...confirmed.popover.behavior, ...changes.popover.behavior},
      layout: {...confirmed.popover.layout, ...changes.popover.layout},
      appearance: {...confirmed.popover.appearance, ...changes.popover.appearance},
    },
  };
}

function confirmedSettings(queryClient: QueryClient): {status: 'available'; configuration: AppConfig} | UpdateBlock {
  const confirmed = queryClient.getQueryState(appConfigQuery.queryKey);
  if (confirmed?.error) {
    return {status: 'unavailable', error: confirmed.error};
  }
  return confirmed?.data === undefined ? {status: 'not_loaded'} : {status: 'available', configuration: confirmed.data};
}

function settle(store: Store<SubmissionState>, write: SettingsWrite, result: SubmissionState): void {
  store.setState(current => (current.status === 'pending' && current.write === write ? result : current));
}

function stopWrites(
  store: Store<SubmissionState>,
  write: SettingsWrite,
  blockedBy: UpdateBlock,
  failure?: FailedWrite,
): void {
  write.batch.blockedBy ??= blockedBy;
  store.setState(current => {
    if (current.status !== 'pending' || current.write.batch !== write.batch) {
      return current;
    }
    return current.write === write && failure
      ? failure
      : {status: 'not_sent', configuration: current.write.configuration, blockedBy};
  });
}
