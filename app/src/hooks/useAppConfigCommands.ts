import {createContext, useContext} from 'react';

import type {Draft} from 'mutative';

import type {ConfigurationCommandError} from '#commands/ConfigurationCommandError';
import type {AppConfig} from '#config';

export type UpdateBlock = {readonly status: 'not_loaded'} | {readonly status: 'unavailable'; readonly error: Error};

export type ConfigurationSubmission =
  | {readonly status: 'idle'}
  | {readonly status: 'pending'; readonly configuration: AppConfig}
  | {readonly status: 'failed'; readonly configuration: AppConfig; readonly error: ConfigurationCommandError}
  | {readonly status: 'not_sent'; readonly configuration: AppConfig; readonly blockedBy: UpdateBlock};

export type UpdateAdmission = {readonly status: 'submitted'} | UpdateBlock;

export type ConfigurationCommands = {
  readonly update: (recipe: (draft: Draft<AppConfig>) => void) => UpdateAdmission;
  readonly submission: ConfigurationSubmission;
};

export const AppConfigCommandsContext = createContext<ConfigurationCommands | undefined>(undefined);

export function useAppConfigCommands(): ConfigurationCommands {
  const commands = useContext(AppConfigCommandsContext);
  if (commands === undefined) {
    throw new Error('AppConfigCommandsProvider is missing');
  }
  return commands;
}
