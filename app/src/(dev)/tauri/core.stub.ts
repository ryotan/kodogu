// Stub for @tauri-apps/api/core in browser-only environments (Storybook, vitest browser tests).
// Tauri IPC is not available in the browser, so we route command names to in-memory handlers.
// The default handlers serve the shared AppConfig fixture; individual stories can override a
// command's response via setTauriStubHandler.
//
// This module lives in `(dev)/` so it is excluded from production bundles by convention. Its only
// consumer is `.storybook/main.ts`, which aliases `@tauri-apps/api/core` to this file at build
// time — app code never imports it directly.

import {defaultAppConfig} from '#commands/appConfig.fixture';

// Each handler returns an unknown payload; invoke<T> casts to the caller-supplied T at the
// stub boundary just like the real `@tauri-apps/api/core` invoke<T> does at runtime.
type Handler = (args?: unknown) => Promise<unknown>;

// Map<string, Handler> instead of Record<string, Handler> so that lookups return Handler|undefined
// without depending on noUncheckedIndexedAccess.
// Default handlers cover the Tauri commands that real app code calls during render. Returning
// no-op (or fixture data) keeps stories and component specs renderable without per-test setup.
// Tests that assert specific Tauri behavior override these via `vi.mock(...)` or by calling
// `setTauriStubHandler(...)`.
const handlers = new Map<string, Handler>([
  ['get_config', () => Promise.resolve(defaultAppConfig)],
  ['set_config', () => Promise.resolve()],
  ['resize_popover', () => Promise.resolve()],
]);

export function invoke<T>(cmd: string, args?: unknown): Promise<T> {
  const handler = handlers.get(cmd);
  if (!handler) {
    // Surface unregistered commands instead of returning undefined silently — otherwise a
    // mistyped command name leaves the consumer waiting on a never-resolving query value.
    console.warn(
      `[tauri-core-stub] No handler registered for command "${cmd}". Returning undefined. ` +
        `Register one via setTauriStubHandler() in your story.`,
    );
    return Promise.resolve(undefined as T);
  }
  return handler(args) as Promise<T>;
}

export function setTauriStubHandler(cmd: string, handler: Handler): void {
  handlers.set(cmd, handler);
}
