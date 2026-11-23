import {spawnSync} from 'node:child_process';

// The crates of the Cargo workspace that depend on `tauri`, directly or through another crate of the workspace.
// tauri-build 2.7.4 omits the Common Controls v6 manifest from the test executables of such crates on Windows, so the
// tests there run without them.
//
// Usage, from the root of the repository:
//   bun tooling/ci/tauri-crates.ts coverage [args...]  runs `mise run cargo:coverage` with `--exclude` for each crate
//   bun tooling/ci/tauri-crates.ts exclude-args        prints those `--exclude` arguments, one on each line

type Dependency = {name: string; path?: string};
type MetadataPackage = {name: string; manifest_path: string; dependencies: Dependency[]};

function workspaceCrates(): MetadataPackage[] {
  const result = spawnSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {encoding: 'utf8'});
  if (result.status !== 0) {
    throw new Error(`cargo metadata failed:\n${result.stderr}`);
  }
  return (JSON.parse(result.stdout) as {packages: MetadataPackage[]}).packages;
}

export function tauriCrates(crates: readonly MetadataPackage[]): string[] {
  const names = new Set(crates.map(crate => crate.name));
  const dependsOnTauri = (crate: MetadataPackage, seen: ReadonlySet<string>): boolean =>
    crate.dependencies.some(dependency => {
      if (dependency.name === 'tauri') {
        return true;
      }
      const member = crates.find(other => other.name === dependency.name);
      return (
        dependency.path !== undefined &&
        names.has(dependency.name) &&
        member !== undefined &&
        !seen.has(member.name) &&
        dependsOnTauri(member, new Set([...seen, member.name]))
      );
    });
  return crates
    .filter(crate => dependsOnTauri(crate, new Set([crate.name])))
    .map(crate => crate.name)
    .sort();
}

function excludeArgs(): string[] {
  return tauriCrates(workspaceCrates()).flatMap(name => ['--exclude', name]);
}

function main(): number {
  const [command, ...rest] = process.argv.slice(2);
  switch (command) {
    case 'exclude-args':
      console.log(excludeArgs().join('\n'));
      return 0;
    case 'coverage': {
      const result = spawnSync('mise', ['run', 'cargo:coverage', '--', ...excludeArgs(), ...rest], {stdio: 'inherit'});
      return result.status ?? 1;
    }
    default:
      console.error('usage: bun tooling/ci/tauri-crates.ts coverage [args...] | exclude-args');
      return 2;
  }
}

if (import.meta.main) {
  process.exitCode = main();
}
