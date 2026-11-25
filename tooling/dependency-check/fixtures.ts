import {spawnSync} from 'node:child_process';
import {existsSync, readdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import path from 'node:path';

// Runs the checks on their fixture cases and compares each result with the case's expectation.
// Usage: bun fixtures.ts. Exits with 1 when a result differs.
//
// A case of the checks of the build units and the modules is a directory of `fixtures/` with `expected.units.txt`,
// `expected.modules.txt` or both: the exit status on the first line, then the standard output of the check. The case of
// Biome is `apps/desktop/dependency-fixtures/` with `expected.txt`: the exit status of Biome, then its diagnostics.

const here = import.meta.dirname;
const repository = path.resolve(here, '..', '..');
const biomeFixtures = 'apps/desktop/dependency-fixtures';

type Result = {readonly name: string; readonly expected: string; readonly actual: string};

function run(
  command: string,
  args: readonly string[],
  cwd: string,
  env: NodeJS.ProcessEnv = process.env,
): {status: number; stdout: string} {
  const result = spawnSync(command, args, {cwd, env, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024});
  if (result.error) {
    throw result.error;
  }
  return {status: result.status ?? -1, stdout: result.stdout};
}

function readExpected(file: string): string {
  return existsSync(file) ? normalize(readFileSync(file, 'utf8')) : '(no expectation)\n';
}

function normalize(text: string): string {
  return `${text.replace(/\r\n/g, '\n').trimEnd()}\n`;
}

function checkResults(): Result[] {
  const fixtures = path.join(here, 'fixtures');
  const cases = readdirSync(fixtures, {withFileTypes: true}).filter(entry => entry.isDirectory());
  return cases.flatMap(entry =>
    (['units', 'modules'] as const).flatMap(check => {
      const expectedFile = path.join(fixtures, entry.name, `expected.${check}.txt`);
      if (!existsSync(expectedFile)) {
        return [];
      }
      // Warnings denied as in CI, so that a warning that fails the check there fails it here as well.
      const {status, stdout} = run(
        process.execPath,
        [path.join(here, `${check}.ts`), '.'],
        path.join(fixtures, entry.name),
        {...process.env, CARGO_BUILD_WARNINGS: 'deny'},
      );
      return [
        {
          name: `${entry.name} (${check})`,
          expected: readExpected(expectedFile),
          actual: normalize(`${status}\n${stdout}`),
        },
      ];
    }),
  );
}

// Strips the comments of JSONC, outside strings.
function stripComments(text: string): string {
  return text.replace(/("(?:[^"\\]|\\.)*")|\/\/[^\n]*|\/\*[\s\S]*?\*\//g, (_, string?: string) => string ?? '');
}

// The repository's configuration of Biome without the exclusion of the fixtures, as a file beside it: Biome resolves
// the globs of a configuration from the directory of the file. `.gitignore` names the file, so that Biome leaves it
// out of the lint of the tree.
const fixtureConfiguration = path.join(repository, '.dependency-fixtures.biome.json');

function writeFixtureConfiguration(): void {
  const configuration = JSON.parse(stripComments(readFileSync(path.join(repository, 'biome.jsonc'), 'utf8'))) as {
    files: {includes: string[]};
  };
  const includes = configuration.files.includes.filter(glob => glob !== `!${biomeFixtures}`);
  writeFileSync(fixtureConfiguration, JSON.stringify({...configuration, files: {...configuration.files, includes}}));
}

// `::error title=<rule>,file=<file>,line=<line>,…,col=<column>,…::<message>`, one line for each diagnostic.
function diagnostics(output: string): string[] {
  return output
    .split('\n')
    .flatMap(line => {
      const match = /^::\w+ title=([^,]+),file=([^,]+),line=(\d+),endLine=\d+,col=(\d+),endColumn=\d+::(.*)$/.exec(
        line,
      );
      if (match === null) {
        return [];
      }
      const [, rule, file = '', row, column, message] = match;
      const relative = path.relative(repository, path.resolve(repository, file)).split(path.sep).join('/');
      return [`${relative}:${row}:${column} ${rule} ${message}`];
    })
    .sort();
}

function biomeResult(): Result {
  writeFixtureConfiguration();
  try {
    const args = ['x', '--no-install', 'biome', 'lint', `--config-path=${fixtureConfiguration}`];
    const {status, stdout} = run(
      process.execPath,
      [...args, '--only=style/noRestrictedImports', '--reporter=github', '--max-diagnostics=none', biomeFixtures],
      repository,
    );
    return {
      name: biomeFixtures,
      expected: readExpected(path.join(repository, biomeFixtures, 'expected.txt')),
      actual: normalize(`${status}\n${diagnostics(stdout).join('\n')}`),
    };
  } finally {
    rmSync(fixtureConfiguration, {force: true});
  }
}

function main(): number {
  const results = [...checkResults(), biomeResult()];
  const failed = results.filter(result => result.expected !== result.actual);
  for (const result of results) {
    console.log(`${failed.includes(result) ? 'FAIL' : 'ok'} ${result.name}`);
  }
  for (const result of failed) {
    console.log(`\n${result.name}\n--- expected\n${result.expected}--- actual\n${result.actual}`);
  }
  return failed.length > 0 ? 1 : 0;
}

process.exitCode = main();
