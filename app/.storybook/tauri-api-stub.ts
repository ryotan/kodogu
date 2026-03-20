// Stub for @tauri-apps/api/core in Storybook environment.
// Tauri IPC is not available in the browser, so invoke is a no-op.
export function invoke(): Promise<void> {
  return Promise.resolve();
}
