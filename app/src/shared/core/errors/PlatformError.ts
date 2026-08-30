import {RootError} from './RootError';

/**
 * Errors that originate from the platform or infrastructure layer — Tauri IPC failures,
 * filesystem errors, network problems, OS-level rejections, missing browser APIs, and other
 * environmental misbehaviour the app itself cannot prevent.
 *
 * Throw `PlatformError` (or extend it) when the cause is _not_ expressible in app-domain
 * terms and the failure is rooted in the runtime: "IPC handler crashed", "couldn't write
 * config to disk", "this Tauri plugin isn't installed". The app asked correctly; the
 * environment couldn't deliver.
 *
 * For failures rooted in application logic itself, reach for {@link ApplicationError}.
 */
export class PlatformError extends RootError {}
