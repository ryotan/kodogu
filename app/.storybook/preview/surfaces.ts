// Shared surface model — consumed by both the preview decorator (`withSurface.tsx`) and the
// manager-side toolbar (`.storybook/manager.tsx`). Neither side owns the other, so the type lives
// here rather than in whichever file happened to need it first.
export type Appearance = 'inherit' | 'light' | 'dark';
// Not exported: nothing outside this module needs the kind in isolation — consumers get it via
// `Surface.kind`.
type SurfaceKind = 'app' | 'popover';

// The wire form — what the toolbar option list, story `parameters.surface.value`, and the `surface`
// global actually hold. A plain string so it stays URL/JSON-friendly.
export type SurfaceLiteral = `${SurfaceKind}-${Appearance}`;

// The structured form — what `withSurface.tsx` actually consumes (`.kind` to pick a component,
// `.appearance` to pass through). Decoding a `SurfaceLiteral` into this shape is this module's own
// business (see `parseSurfaceLiteral` below), never a concern for its callers.
export type Surface = {kind: SurfaceKind; appearance: Appearance};

// Neither `SurfaceKind` nor `Appearance` values contain `-`, so this split is unambiguous.
function parseSurfaceLiteral(literal: SurfaceLiteral): Surface {
  const [kind, appearance] = literal.split('-') as [SurfaceKind, Appearance];
  return {kind, appearance};
}

// The selectable surfaces, shared by the preview's `globalTypes` declaration and the manager-side
// custom toolbar. The toolbar adds its own "Reset" entry that clears the global (sets it to
// `undefined`), which a declarative `globalTypes.toolbar.items` list can't express — a list item
// can only set a value, never unset the global.
export const SURFACE_OPTIONS: {value: SurfaceLiteral; title: string}[] = [
  {value: 'app-inherit', title: 'App (inherit)'},
  {value: 'app-light', title: 'App (light)'},
  {value: 'app-dark', title: 'App (dark)'},
  {value: 'popover-inherit', title: 'Popover (inherit)'},
  {value: 'popover-light', title: 'Popover (light)'},
  {value: 'popover-dark', title: 'Popover (dark)'},
];

export const SURFACE_GLOBAL_KEY = 'surface';

const VALID_SURFACE_LITERALS = new Set<string>(SURFACE_OPTIONS.map(option => option.value));

// `globals[SURFACE_GLOBAL_KEY]` is untyped (Storybook's `Globals` is an index signature by design)
// and reflects state a user can hand-edit via the URL, so a raw value needs checking against
// `SURFACE_OPTIONS` — the only source of truth for what counts as valid — before it's trusted.
export function isSurfaceLiteral(value: unknown): value is SurfaceLiteral {
  return typeof value === 'string' && VALID_SURFACE_LITERALS.has(value);
}

export function resolveSurface(globals: Record<string, unknown>, parameterValue: SurfaceLiteral | undefined): Surface {
  const toolbarValue = globals[SURFACE_GLOBAL_KEY];
  const toolbarLiteral = isSurfaceLiteral(toolbarValue) ? toolbarValue : undefined;
  return parseSurfaceLiteral(toolbarLiteral ?? parameterValue ?? 'app-inherit');
}
