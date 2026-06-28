import type {Surface} from './withTheme';

// The selectable surfaces, shared by the preview's `globalTypes` declaration and the manager-side
// custom toolbar. The toolbar adds its own "Reset" entry that clears the global (sets it to
// `undefined`), which a declarative `globalTypes.toolbar.items` list can't express — a list item
// can only set a value, never unset the global.
export const SURFACE_OPTIONS: {value: Surface; title: string}[] = [
  {value: 'app-inherit', title: 'App (inherit)'},
  {value: 'app-light', title: 'App (light)'},
  {value: 'app-dark', title: 'App (dark)'},
  {value: 'popover-inherit', title: 'Popover (inherit)'},
  {value: 'popover-light', title: 'Popover (light)'},
  {value: 'popover-dark', title: 'Popover (dark)'},
];

export const SURFACE_GLOBAL_KEY = 'themeSurface';
