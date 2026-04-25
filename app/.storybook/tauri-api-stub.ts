// Stub for @tauri-apps/api/core in the Storybook environment.
// Tauri IPC is not available in the browser, so we route command names to in-memory
// handlers. The default handlers serve the shared AppConfig fixture; individual stories can
// override a command's response via setTauriMockHandler.

import {defaultAppConfig} from '../src/commands/appConfig.fixture';

// biome-ignore lint/suspicious/noExplicitAny: handlers receive arbitrary command args
type Handler = (args?: any) => Promise<any>;

// Map<string, Handler> instead of Record<string, Handler> so that lookups return Handler|undefined
// without depending on noUncheckedIndexedAccess.
const handlers = new Map<string, Handler>([
  ['get_config', () => Promise.resolve(defaultAppConfig)],
  ['set_config', () => Promise.resolve()],
]);

export function invoke<T>(cmd: string, args?: unknown): Promise<T> {
  const handler = handlers.get(cmd);
  if (!handler) {
    return Promise.resolve(undefined as T);
  }
  return handler(args) as Promise<T>;
}

export function setTauriMockHandler(cmd: string, handler: Handler): void {
  handlers.set(cmd, handler);
}
