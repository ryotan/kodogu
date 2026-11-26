import {spawnSync} from 'node:child_process';
import {readdirSync, readFileSync} from 'node:fs';
import path from 'node:path';

// Reading the build units of a tree: the crates from `cargo metadata`, the packages from the `package.json` files.

// A failure to read the tree, as opposed to a finding about it. The checks exit with status 2 on it.
export class ReadError extends Error {
  override readonly name = 'ReadError';
}

export type DependencyKind = 'normal' | 'dev' | 'build';

export type CargoDependency = {
  readonly name: string;
  readonly rename: string | undefined;
  readonly req: string;
  readonly kind: DependencyKind;
  readonly target: string | undefined;
  readonly path: string | undefined;
  readonly optional: boolean;
  readonly usesDefaultFeatures: boolean;
  readonly features: readonly string[];
};

export type Crate = {
  readonly kind: 'crate';
  readonly name: string;
  // The directory of the manifest, relative to the root, with `/`.
  readonly unit: string;
  readonly directory: string;
  readonly edition: string;
  readonly libPath: string | undefined;
  readonly hasBuildScript: boolean;
  readonly features: Readonly<Record<string, readonly string[]>>;
  readonly dependencies: readonly CargoDependency[];
};

export type PackageField = 'dependencies' | 'devDependencies' | 'peerDependencies' | 'optionalDependencies';

export const packageFields: readonly PackageField[] = [
  'dependencies',
  'devDependencies',
  'peerDependencies',
  'optionalDependencies',
];

export type PackageDependency = {
  // The name the dependency points to, after an alias (`npm:<name>@…`, `workspace:<name>@…`) is resolved.
  readonly name: string;
  readonly field: PackageField;
};

export type Package = {
  readonly kind: 'package';
  readonly name: string;
  readonly unit: string;
  readonly directory: string;
  readonly exports: unknown;
  readonly dependencies: readonly PackageDependency[];
};

export function toUnit(root: string, directory: string): string {
  return path.relative(root, directory).split(path.sep).join('/');
}

function run(command: string, args: readonly string[], cwd: string): string {
  const result = spawnSync(command, args, {cwd, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024});
  if (result.error) {
    throw new ReadError(`${command} ${args.join(' ')}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new ReadError(`${command} ${args.join(' ')} failed:\n${result.stderr}`);
  }
  return result.stdout;
}

type MetadataDependency = {
  name: string;
  rename: string | null;
  req: string;
  kind: 'dev' | 'build' | null;
  target: string | null;
  path?: string;
  optional: boolean;
  uses_default_features: boolean;
  features: string[];
};

type MetadataPackage = {
  name: string;
  manifest_path: string;
  edition: string;
  features: Record<string, string[]>;
  dependencies: MetadataDependency[];
  targets: {kind: string[]; src_path: string}[];
};

export type Metadata = {
  readonly crates: readonly Crate[];
  readonly targetDirectory: string;
};

function toDependency(dependency: MetadataDependency): CargoDependency {
  return {
    name: dependency.name,
    rename: dependency.rename ?? undefined,
    req: dependency.req,
    kind: dependency.kind ?? 'normal',
    target: dependency.target ?? undefined,
    path: dependency.path,
    optional: dependency.optional,
    usesDefaultFeatures: dependency.uses_default_features,
    features: dependency.features,
  };
}

function toCrate(root: string, metadataPackage: MetadataPackage): Crate {
  const directory = path.dirname(metadataPackage.manifest_path);
  const lib = metadataPackage.targets.find(target => target.kind.includes('lib'));
  return {
    kind: 'crate',
    name: metadataPackage.name,
    unit: toUnit(root, directory),
    directory,
    edition: metadataPackage.edition,
    libPath: lib?.src_path,
    hasBuildScript: metadataPackage.targets.some(target => target.kind.includes('custom-build')),
    features: metadataPackage.features,
    dependencies: metadataPackage.dependencies.map(toDependency),
  };
}

// A workspace member without a manifest makes `cargo metadata` fail, which is reported as a read error.
export function readMetadata(root: string): Metadata {
  const output = run('cargo', ['metadata', '--no-deps', '--format-version', '1'], root);
  const metadata = JSON.parse(output) as {packages: MetadataPackage[]; target_directory: string};
  return {
    crates: metadata.packages.map(metadataPackage => toCrate(root, metadataPackage)),
    targetDirectory: metadata.target_directory,
  };
}

const skippedDirectories = new Set(['node_modules', 'fixtures', 'target', '.git']);

function findManifests(directory: string): string[] {
  return readdirSync(directory, {withFileTypes: true}).flatMap(entry => {
    if (entry.isDirectory()) {
      return skippedDirectories.has(entry.name) ? [] : findManifests(path.join(directory, entry.name));
    }
    return entry.name === 'package.json' ? [path.join(directory, entry.name)] : [];
  });
}

// An alias: `npm:<name>` with an optional `@<range>`, or `workspace:<name>@<range>`. `workspace:*` and the like are
// not.
const aliasPatterns = [/^npm:((?:@[^/@]+\/)?[^@]+)(?:@.*)?$/, /^workspace:((?:@[^/@]+\/)?[^@]+)@.*$/];

function resolveAlias(name: string, specifier: unknown): string {
  if (typeof specifier !== 'string') {
    return name;
  }
  return aliasPatterns.map(pattern => pattern.exec(specifier)?.[1]).find(target => target !== undefined) ?? name;
}

function readJson(file: string): Record<string, unknown> {
  try {
    return JSON.parse(readFileSync(file, 'utf8')) as Record<string, unknown>;
  } catch (error) {
    throw new ReadError(`${file}: ${error instanceof Error ? error.message : String(error)}`);
  }
}

function toPackage(root: string, manifest: string): Package {
  const data = readJson(manifest);
  const directory = path.dirname(manifest);
  if (typeof data.name !== 'string') {
    throw new ReadError(`${manifest}: no name`);
  }
  return {
    kind: 'package',
    name: data.name,
    unit: toUnit(root, directory),
    directory,
    exports: data.exports,
    dependencies: packageFields.flatMap(field => {
      const entries = Object.entries((data[field] ?? {}) as Record<string, unknown>);
      return entries.map(([name, specifier]) => ({name: resolveAlias(name, specifier), field}));
    }),
  };
}

// Every package under the root, except the root itself and the directories `node_modules/` and `fixtures/`.
export function readPackages(root: string): Package[] {
  return findManifests(root)
    .filter(manifest => path.dirname(manifest) !== path.resolve(root))
    .map(manifest => toPackage(root, manifest));
}
