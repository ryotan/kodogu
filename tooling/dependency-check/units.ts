import {readdirSync, readFileSync} from 'node:fs';
import path from 'node:path';

import type {Crate, Package} from './manifests.ts';
import {ReadError, readMetadata, readPackages, toUnit} from './manifests.ts';
import type {Place, PlaceName} from './places.ts';
import {allowed, placeOf} from './places.ts';

// Checks the build units of a tree against the table of dependency directions, for crates and packages alike.
// Usage: bun units.ts <root>. Prints one line for each finding, sorted, and exits with 1; or `no finding` and 0.
// A tree that cannot be read exits with 2, so that it is not taken for a finding.

type Edge = {
  readonly from: string;
  readonly fromPlace: Place;
  readonly to: string;
  readonly toPlace: Place;
  // A development dependency: `devDependencies` of a package, a `dev` dependency of a crate.
  readonly development: boolean;
};

function crateEdges(root: string, crate: Crate): Edge[] {
  const fromPlace = placeOf(crate.unit, 'crate');
  return crate.dependencies.flatMap(dependency => {
    if (dependency.path === undefined) {
      return [];
    }
    const to = toUnit(root, dependency.path);
    return [{from: crate.unit, fromPlace, to, toPlace: placeOf(to, 'crate'), development: dependency.kind === 'dev'}];
  });
}

function packageEdges(packages: readonly Package[], current: Package): Edge[] {
  const fromPlace = placeOf(current.unit, 'package');
  return current.dependencies.flatMap(dependency => {
    const target = packages.find(candidate => candidate.name === dependency.name);
    if (target === undefined) {
      return [];
    }
    const toPlace = placeOf(target.unit, 'package');
    return [
      {from: current.unit, fromPlace, to: target.unit, toPlace, development: dependency.field === 'devDependencies'},
    ];
  });
}

function edgeFinding(edge: Edge): string | undefined {
  const {from, fromPlace, to, toPlace} = edge;
  if (toPlace.kind === 'unplaced') {
    // Reported as a finding of that unit itself.
    return undefined;
  }
  const prefix = `${from} -> ${to}`;
  if (toPlace.name === 'settings') {
    return edge.development ? undefined : `${prefix}: settings may be a development dependency only`;
  }
  if (fromPlace.kind === 'unplaced') {
    return `${prefix}: a unit with no place may not depend on ${toPlace.name}`;
  }
  if (fromPlace.tool !== undefined && toPlace.tool !== undefined && fromPlace.tool !== toPlace.tool) {
    return `${prefix}: a tool may not depend on another tool`;
  }
  return isAllowed(fromPlace.name, toPlace.name)
    ? undefined
    : `${prefix}: ${fromPlace.name} may not depend on ${toPlace.name}`;
}

function isAllowed(from: PlaceName, to: PlaceName): boolean {
  return from === to || allowed[from].includes(to);
}

function placeFindings(unit: string, place: Place): string[] {
  return place.kind === 'unplaced' ? [`${unit}: ${place.reason}`] : [];
}

const tauriMessage = 'only the desktop app and the desktop parts of tools may depend on Tauri';

function mayUseTauri(unit: string): boolean {
  return unit.startsWith('apps/desktop/') || /^tools\/[^/]+\/desktop(\/|$)/.test(unit);
}

function crateTauriFindings(crate: Crate): string[] {
  if (mayUseTauri(crate.unit)) {
    return [];
  }
  const names = crate.dependencies.map(dependency => dependency.name);
  return names
    .filter(name => name === 'tauri' || name.startsWith('tauri-'))
    .map(name => `${crate.unit} -> ${name}: ${tauriMessage}`);
}

function packageTauriFindings(current: Package): string[] {
  const place = placeOf(current.unit, 'package');
  if (mayUseTauri(current.unit) || (place.kind === 'placed' && place.name === 'settings')) {
    return [];
  }
  const names = current.dependencies.map(dependency => dependency.name);
  return names
    .filter(name => name.startsWith('@tauri-apps/'))
    .map(name => `${current.unit} -> ${name}: ${tauriMessage}`);
}

function cycleFindings(edges: readonly Edge[]): string[] {
  const next = new Map<string, string[]>();
  for (const edge of edges) {
    next.set(edge.from, [...(next.get(edge.from) ?? []), edge.to]);
  }
  const cycles = new Set<string>();
  const visit = (unit: string, stack: readonly string[]): void => {
    const start = stack.indexOf(unit);
    if (start >= 0) {
      cycles.add(canonicalCycle(stack.slice(start)));
      return;
    }
    for (const to of next.get(unit) ?? []) {
      visit(to, [...stack, unit]);
    }
  };
  for (const unit of next.keys()) {
    visit(unit, []);
  }
  return [...cycles].map(cycle => `${cycle}: a cycle between packages`);
}

// The cycle written from its smallest unit, so that each cycle is reported once.
function canonicalCycle(units: readonly string[]): string {
  const smallest = units.indexOf([...units].sort()[0] ?? '');
  const rotated = [...units.slice(smallest), ...units.slice(0, smallest)];
  return [...rotated, rotated[0]].join(' -> ');
}

const developmentFragments = ['/(dev)/', '/(prototype)/', '/gen/', '.test.', '.spec.', '.story.', '.fixture.'];

function exportEntries(exports: unknown): [string, unknown][] {
  if (exports === undefined || exports === null) {
    return [];
  }
  if (typeof exports === 'string' || Array.isArray(exports)) {
    return [['.', exports]];
  }
  const entries = Object.entries(exports);
  return entries.length > 0 && entries.every(([key]) => key.startsWith('.')) ? entries : [['.', exports]];
}

function targetsOf(value: unknown): string[] {
  if (typeof value === 'string') {
    return [value];
  }
  if (Array.isArray(value)) {
    return value.flatMap(targetsOf);
  }
  return value !== null && typeof value === 'object' ? Object.values(value).flatMap(targetsOf) : [];
}

// The key that Node resolves a subpath through: an exact key, else the pattern with the longest prefix, then the
// longest key.
function resolvingKey(keys: readonly string[], subpath: string): string | undefined {
  if (keys.includes(subpath)) {
    return subpath;
  }
  const matching = keys.filter(key => {
    const star = key.indexOf('*');
    return (
      star >= 0 &&
      subpath.startsWith(key.slice(0, star)) &&
      subpath.endsWith(key.slice(star + 1)) &&
      subpath.length >= key.length
    );
  });
  return matching.sort((a, b) => b.indexOf('*') - a.indexOf('*') || b.length - a.length)[0];
}

function filesOf(directory: string, base = directory): string[] {
  return readdirSync(directory, {withFileTypes: true}).flatMap(entry => {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      return entry.name === 'node_modules' ? [] : filesOf(full, base);
    }
    return [`./${toUnit(base, full)}`];
  });
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function exposedFiles(current: Package): [string, string][] {
  const entries = exportEntries(current.exports);
  const keys = entries.map(([key]) => key);
  const files = entries.some(([key]) => key.includes('*')) ? filesOf(current.directory) : [];
  return entries.flatMap(([key, value]) => {
    const targets = targetsOf(value);
    if (!key.includes('*')) {
      return targets.map((target): [string, string] => [key, target]);
    }
    // A target without `*` under a pattern key: every subpath that this key resolves gets that one file, and the key
    // resolves at least one, whatever the other keys are. Take as the capture a character that no key contains,
    // repeated more times than the longest key is long. No exact key equals that subpath. A key that Node prefers
    // has a longer prefix, or the same prefix and a longer suffix; either would have to contain that character.
    const fixed = targets.filter(target => !target.includes('*'));
    return [
      ...fixed.map((target): [string, string] => [key, target]),
      ...targets
        .filter(target => target.includes('*'))
        .flatMap(target => {
          // Node puts the string that the key's `*` matched in place of every `*` of the target. The reference to it
          // is named, so that a digit after a wildcard is not read as part of the reference.
          const [first = '', ...rest] = target.split('*').map(escapeRegExp);
          const pattern = new RegExp(
            `^${first}${rest.map((piece, index) => `${index === 0 ? '(?<s>.*)' : '\\k<s>'}${piece}`).join('')}$`,
          );
          return files.flatMap((file): [string, string][] => {
            const captured = pattern.exec(file)?.[1];
            return captured !== undefined && resolvingKey(keys, key.replace('*', captured)) === key
              ? [[key, file]]
              : [];
          });
        }),
    ];
  });
}

function exportFindings(current: Package): string[] {
  const place = placeOf(current.unit, 'package');
  const isFoundation = place.kind === 'placed' && place.name === 'desktop-foundation';
  return exposedFiles(current)
    .filter(
      ([key, file]) =>
        !(isFoundation && key === './storybook/preview') && developmentFragments.some(f => file.includes(f)),
    )
    .map(([key, file]) => `${current.unit}: export ${key} exposes ${file}`);
}

const osCondition = /\b(target_os|target_family|windows|unix)\b/;

// The conditions of `cfg(`, `cfg!(` and `cfg_attr(` in a source text, each with its line. A condition built by a
// macro is not seen; the check of the modules compiles the domain on each operating system of CI.
function cfgConditions(source: string): {line: number; condition: string}[] {
  return [...source.matchAll(/\b(?:cfg!?|cfg_attr)\(/g)].map(match => {
    const start = (match.index ?? 0) + match[0].length;
    let depth = 1;
    let end = start;
    while (end < source.length && depth > 0) {
      depth += source[end] === '(' ? 1 : source[end] === ')' ? -1 : 0;
      end += 1;
    }
    return {line: source.slice(0, match.index).split('\n').length, condition: source.slice(start, end - 1)};
  });
}

function rustFiles(directory: string): string[] {
  return readdirSync(directory, {withFileTypes: true}).flatMap(entry => {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      return entry.name === 'target' ? [] : rustFiles(full);
    }
    return entry.name.endsWith('.rs') ? [full] : [];
  });
}

const osMessage = "a tool's domain and types may not hold code for one operating system";

function operatingSystemFindings(root: string, crate: Crate): string[] {
  const place = placeOf(crate.unit, 'crate');
  if (place.kind !== 'placed' || (place.name !== 'tool-domain' && place.name !== 'tool-types')) {
    return [];
  }
  const sections = crate.dependencies
    .filter(dependency => dependency.target !== undefined)
    .map(dependency => `${crate.unit} -> ${dependency.name}: a section for ${dependency.target ?? ''}; ${osMessage}`);
  const conditions = rustFiles(crate.directory).flatMap(file =>
    cfgConditions(readFileSync(file, 'utf8'))
      .filter(({condition}) => osCondition.test(condition))
      .map(
        ({line}) =>
          `${crate.unit}: ${toUnit(root, file)}:${line} has a cfg condition on an operating system; ${osMessage}`,
      ),
  );
  return [...sections, ...conditions];
}

export function findings(root: string): string[] {
  const {crates} = readMetadata(root);
  const packages = readPackages(root);
  const crateEdgeList = crates.flatMap(crate => crateEdges(root, crate));
  const packageEdgeList = packages.flatMap(current => packageEdges(packages, current));
  const all = [
    ...crates.flatMap(crate => placeFindings(crate.unit, placeOf(crate.unit, 'crate'))),
    ...packages.flatMap(current => placeFindings(current.unit, placeOf(current.unit, 'package'))),
    ...[...crateEdgeList, ...packageEdgeList].flatMap(edge => edgeFinding(edge) ?? []),
    ...crates.flatMap(crateTauriFindings),
    ...packages.flatMap(packageTauriFindings),
    ...cycleFindings(packageEdgeList),
    ...packages.flatMap(exportFindings),
    ...crates.flatMap(crate => operatingSystemFindings(root, crate)),
  ];
  return [...new Set(all)].sort();
}

function main(): number {
  const [root] = process.argv.slice(2);
  if (root === undefined) {
    console.error('usage: bun units.ts <root>');
    return 2;
  }
  try {
    const found = findings(path.resolve(root));
    console.log(found.length > 0 ? found.join('\n') : 'no finding');
    return found.length > 0 ? 1 : 0;
  } catch (error) {
    if (error instanceof ReadError) {
      console.error(error.message);
      return 2;
    }
    throw error;
  }
}

if (import.meta.main) {
  process.exitCode = main();
}
