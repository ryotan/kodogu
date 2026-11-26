// The places of the table of dependency directions, and what each may depend on.

export type ToolPart = 'types' | 'domain' | 'resident' | 'desktop' | 'cli';

export type PlaceName =
  | 'kernel'
  | 'resident-foundation'
  | 'resident-startup'
  | 'desktop-routing'
  | 'desktop-foundation'
  | 'desktop-startup'
  | 'desktop-function'
  | 'cli-foundation'
  | 'cli-startup'
  | `tool-${ToolPart}`
  | 'settings';

export type Place =
  | {readonly kind: 'placed'; readonly name: PlaceName; readonly tool: string | undefined}
  | {readonly kind: 'unplaced'; readonly reason: string};

const placed = (name: PlaceName, tool?: string): Place => ({kind: 'placed', name, tool});
const unplaced = (reason: string): Place => ({kind: 'unplaced', reason});

// The startup directory of the desktop app.
const desktopStartup = 'app-root';
// A directory of a tool other than these holds its types; that directory has no name yet.
const toolParts: Readonly<Record<string, PlaceName>> = {
  domain: 'tool-domain',
  resident: 'tool-resident',
  desktop: 'tool-desktop',
  cli: 'tool-cli',
};
const noPlace = unplaced('no place in the table');

function appPlace(application: string, part: string): Place {
  switch (application) {
    case 'resident':
      return placed(part === 'foundation' ? 'resident-foundation' : 'resident-startup');
    case 'desktop': {
      const named: Record<string, PlaceName> = {
        routing: 'desktop-routing',
        foundation: 'desktop-foundation',
        [desktopStartup]: 'desktop-startup',
      };
      return placed(named[part] ?? 'desktop-function');
    }
    case 'cli':
      return placed(part === 'foundation' ? 'cli-foundation' : 'cli-startup');
    default:
      return noPlace;
  }
}

// The place of a build unit from its directory relative to the root; `src-tauri` belongs to its parent.
export function placeOf(unit: string, kind: 'crate' | 'package'): Place {
  const parts = unit.split('/').filter(part => part !== 'src-tauri');
  const [top, second, third] = parts;
  if (top === 'kernel' && parts.length === 2) {
    return kind === 'crate' ? placed('kernel') : unplaced('the kernel holds Rust only');
  }
  if (top === 'apps' && parts.length === 3 && second !== undefined && third !== undefined) {
    return appPlace(second, third);
  }
  if (top === 'tools' && parts.length === 3 && second !== undefined && third !== undefined) {
    return placed(toolParts[third] ?? 'tool-types', second);
  }
  if (top === 'tooling' && parts.length === 2) {
    return placed('settings');
  }
  return noPlace;
}

// What each place may depend on besides itself. Places of a tool are allowed within the same tool only, and
// `settings` is allowed from every place as a development dependency only; both are checked apart from this table.
export const allowed: Readonly<Record<PlaceName, readonly PlaceName[]>> = {
  kernel: ['kernel'],
  'resident-foundation': ['kernel'],
  'resident-startup': ['resident-foundation', 'tool-resident', 'kernel'],
  'desktop-routing': ['kernel'],
  'desktop-function': ['kernel', 'desktop-routing', 'desktop-function', 'resident-foundation'],
  'desktop-foundation': ['kernel', 'desktop-routing', 'desktop-function', 'resident-foundation'],
  'desktop-startup': [
    'desktop-routing',
    'desktop-function',
    'desktop-foundation',
    'tool-desktop',
    'resident-foundation',
    'kernel',
  ],
  'cli-foundation': ['kernel', 'resident-foundation'],
  'cli-startup': ['cli-foundation', 'tool-cli', 'resident-foundation', 'kernel'],
  'tool-types': ['kernel'],
  'tool-domain': ['tool-types', 'kernel'],
  'tool-resident': ['tool-domain', 'tool-types', 'kernel', 'resident-foundation'],
  'tool-desktop': [
    'tool-types',
    'kernel',
    'desktop-routing',
    'desktop-function',
    'desktop-foundation',
    'resident-foundation',
  ],
  'tool-cli': ['tool-types', 'kernel', 'cli-foundation', 'resident-foundation'],
  settings: [],
};
