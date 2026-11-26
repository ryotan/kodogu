import {spawnSync} from 'node:child_process';
import {cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import path from 'node:path';

import type {CargoDependency, Crate} from './manifests.ts';
import {ReadError, readMetadata} from './manifests.ts';
import {placeOf} from './places.ts';
import {readRoot} from './rust-root.ts';

// Checks the modules of each crate of a tool's domain: the domain uses nothing of the use cases.
// Usage: bun modules.ts <root>. Prints one line for each finding, sorted, and exits with 1; or `no finding` and 0.
// The output of `cargo check` for a failing layer goes to standard error. A tree that cannot be read exits with 2.
//
// Each layer is compiled alone, in a temporary crate whose root declares that layer's module and the modules it may
// use, and nothing else. Whatever the layer names outside them fails to compile: a path, a re-export at the crate
// root, a macro that expands to such a path. So the forms that a reading of the source would miss fail closed.

const layers: readonly {readonly module: string; readonly uses: readonly string[]}[] = [
  {module: 'domain', uses: []},
  {module: 'use_cases', uses: ['domain']},
];
const maxFeatures = 4;

// The root of a domain crate holds the two modules, `use` items, attributes and comments, and nothing else. An
// attribute that gives a module a file of its own choosing, directly or under `cfg_attr`, is rejected: the check
// copies the files of the modules from their usual places, and would otherwise check a file that the crate does not
// use. So is an attribute that Rust does not build in: it may be a macro, which can replace a module with other code.
function rootFindings(crate: Crate, rootFile: string): string[] {
  const {attributes, items} = readRoot(readFileSync(rootFile, 'utf8'));
  const declared = items.flatMap(item => (item.kind === 'module' ? [item.name] : []));
  const unexpected = items.filter(item =>
    item.kind === 'module' ? !layers.some(layer => layer.module === item.name) : item.kind !== 'use',
  );
  const missing = layers.filter(layer => !declared.includes(layer.module));
  return [
    ...attributes
      .filter(attribute => attribute.givesPath)
      .map(attribute => `${crate.unit}: the root of a domain crate gives a module a path attribute ${attribute.text}`),
    ...attributes
      .filter(attribute => !attribute.builtIn)
      .map(
        attribute =>
          `${crate.unit}: the root of a domain crate has an attribute that is not built into Rust ${attribute.text}`,
      ),
    ...unexpected.map(item => `${crate.unit}: the root of a domain crate declares ${item.text}`),
    ...missing.map(layer => `${crate.unit}: the root of a domain crate does not declare ${layer.module}`),
  ];
}

function quote(text: string): string {
  return JSON.stringify(text);
}

function dependencyLine(dependency: CargoDependency): string {
  const fields = [
    `package = ${quote(dependency.name)}`,
    dependency.path === undefined ? `version = ${quote(dependency.req)}` : `path = ${quote(dependency.path)}`,
    ...(dependency.features.length > 0 ? [`features = ${JSON.stringify(dependency.features)}`] : []),
    ...(dependency.usesDefaultFeatures ? [] : ['default-features = false']),
    ...(dependency.optional ? ['optional = true'] : []),
  ];
  return `${quote(dependency.rename ?? dependency.name)} = { ${fields.join(', ')} }`;
}

function sectionOf(dependency: CargoDependency): string {
  const table = dependency.kind === 'dev' ? 'dev-dependencies' : 'dependencies';
  return dependency.target === undefined ? table : `target.'${dependency.target}'.${table}`;
}

// The manifest of a temporary crate: the dependencies, the per-target sections and the features of the crate.
function manifest(crate: Crate, name: string): string {
  const sections = new Map<string, string[]>();
  for (const dependency of crate.dependencies.filter(d => d.kind !== 'build')) {
    sections.set(sectionOf(dependency), [...(sections.get(sectionOf(dependency)) ?? []), dependencyLine(dependency)]);
  }
  const features = Object.entries(crate.features).map(
    ([feature, values]) => `${quote(feature)} = ${JSON.stringify(values)}`,
  );
  return [
    `[package]\nname = ${quote(name)}\nversion = "0.0.0"\nedition = ${quote(crate.edition)}\n\n[workspace]\n`,
    ...[...sections].map(([table, lines]) => `[${table}]\n${lines.join('\n')}\n`),
    ...(features.length > 0 ? [`[features]\n${features.join('\n')}\n`] : []),
  ].join('\n');
}

// Every subset of the declared features: every set a build can enable is the closure of one of them.
function featureSubsets(names: readonly string[]): string[][] {
  return Array.from({length: 2 ** names.length}, (_, mask) => names.filter((_, bit) => (mask >> bit) & 1));
}

function copyModule(sourceDirectory: string, targetDirectory: string, module: string): void {
  for (const source of [path.join(sourceDirectory, `${module}.rs`), path.join(sourceDirectory, module)]) {
    if (existsSync(source)) {
      cpSync(source, path.join(targetDirectory, path.basename(source)), {recursive: true});
    }
  }
}

// A temporary manifest that Cargo cannot read, or that declares other features than the crate, is a failure of this
// check, not a finding about the crate.
function assertSameFeatures(crate: Crate, directory: string): void {
  const {crates} = readMetadata(directory);
  const generated = Object.keys(crates[0]?.features ?? {}).sort();
  const declared = Object.keys(crate.features).sort();
  if (JSON.stringify(generated) !== JSON.stringify(declared)) {
    throw new ReadError(`${crate.unit}: the temporary manifest declares the features ${generated.join(', ')}`);
  }
}

function checkLayer(crate: Crate, sourceDirectory: string, work: string, layer: (typeof layers)[number]): boolean {
  const directory = path.join(work, crate.name, `layer-${layer.module}`);
  rmSync(directory, {recursive: true, force: true});
  mkdirSync(path.join(directory, 'src'), {recursive: true});
  const modules = [...layer.uses, layer.module];
  for (const module of modules) {
    copyModule(sourceDirectory, path.join(directory, 'src'), module);
  }
  writeFileSync(path.join(directory, 'src', 'lib.rs'), modules.map(module => `pub mod ${module};\n`).join(''));
  writeFileSync(path.join(directory, 'Cargo.toml'), manifest(crate, `layer-${layer.module.replace(/_/g, '-')}`));
  assertSameFeatures(crate, directory);
  return featureSubsets(Object.keys(crate.features).sort()).every(subset => {
    const flags = ['--no-default-features', ...(subset.length > 0 ? ['--features', subset.join(',')] : [])];
    const result = spawnSync('cargo', ['check', '--quiet', '--all-targets', '--message-format', 'short', ...flags], {
      cwd: directory,
      encoding: 'utf8',
      // A layer compiles with all the dependencies of the crate, and Cargo warns of those it does not use; the check
      // asks only whether the layer compiles, so a setting that denies warnings, as CI has, does not apply.
      env: {...process.env, CARGO_TARGET_DIR: path.join(work, 'target'), CARGO_BUILD_WARNINGS: 'allow'},
    });
    if (result.error) {
      throw new ReadError(`cargo check: ${result.error.message}`);
    }
    if (result.status !== 0) {
      console.error(`${crate.unit}, ${layer.module}, features: ${subset.join(', ') || 'none'}\n${result.stderr}`);
    }
    return result.status === 0;
  });
}

function crateFindings(crate: Crate, work: string): string[] {
  if (crate.hasBuildScript) {
    return [`${crate.unit}: a domain crate may not have a build script, whose cfg values the check cannot reproduce`];
  }
  if (Object.keys(crate.features).length > maxFeatures) {
    return [`${crate.unit}: a domain crate may not have more than ${maxFeatures} features, one build for each subset`];
  }
  if (crate.libPath === undefined) {
    return [`${crate.unit}: a domain crate has no library target`];
  }
  const root = rootFindings(crate, crate.libPath);
  if (root.length > 0) {
    return root;
  }
  // A layer is compiled with the layers it may use, so only the first layer that fails is reported.
  const failing = layers.find(layer => !checkLayer(crate, path.dirname(crate.libPath ?? ''), work, layer));
  return failing === undefined ? [] : [`${crate.unit}: module ${failing.module} uses what it may not`];
}

export function findings(root: string): {checked: number; found: string[]} {
  const metadata = readMetadata(root);
  const domains = metadata.crates.filter(crate => {
    const place = placeOf(crate.unit, 'crate');
    return place.kind === 'placed' && place.name === 'tool-domain';
  });
  const work = path.join(metadata.targetDirectory, 'dependency-check');
  return {checked: domains.length, found: domains.flatMap(crate => crateFindings(crate, work)).sort()};
}

function main(): number {
  const [root] = process.argv.slice(2);
  if (root === undefined) {
    console.error('usage: bun modules.ts <root>');
    return 2;
  }
  try {
    const {checked, found} = findings(path.resolve(root));
    if (checked === 0) {
      console.error('no crate of a tool domain to check');
    }
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
