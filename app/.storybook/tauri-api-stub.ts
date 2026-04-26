// Stub for @tauri-apps/api/core in the Storybook environment.
// Tauri IPC is not available in the browser, so we route command names to in-memory
// handlers. The default handlers serve the shared AppConfig fixture; individual stories can
// override a command's response via setTauriMockHandler.

import {defaultAppConfig} from '../src/commands/appConfig.fixture';

// Each handler returns an unknown payload; invoke<T> casts to the caller-supplied T at the
// stub boundary just like the real `@tauri-apps/api/core` invoke<T> does at runtime.
type Handler = (args?: unknown) => Promise<unknown>;

// Map<string, Handler> instead of Record<string, Handler> so that lookups return Handler|undefined
// without depending on noUncheckedIndexedAccess.
const handlers = new Map<string, Handler>([
  ['get_config', () => Promise.resolve(defaultAppConfig)],
  ['set_config', () => Promise.resolve()],
]);

export function invoke<T>(cmd: string, args?: unknown): Promise<T> {
  const handler = handlers.get(cmd);
  if (!handler) {
    // Surface unregistered commands instead of returning undefined silently — otherwise a
    // mistyped command name leaves the consumer waiting on a never-resolving query value.
    console.warn(
      `[tauri-api-stub] No handler registered for command "${cmd}". Returning undefined. ` +
        `Register one via setTauriMockHandler() in your story.`,
    );
    return Promise.resolve(undefined as T);
  }
  return handler(args) as Promise<T>;
}

export function setTauriMockHandler(cmd: string, handler: Handler): void {
  handlers.set(cmd, handler);
}
