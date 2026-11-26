import {spawnSync} from 'node:child_process';
import path from 'node:path';

// Runs the tests of the package in the working directory with coverage, into a blob report under `.vitest-reports/`
// at the root of the repository, named after the package's directory. `app:test:coverage` merges the reports.
// Written in TypeScript so that the task runs in the shell that mise picks on every operating system.

const root = path.resolve(import.meta.dirname, '..', '..');
const name = path.relative(root, process.cwd()).split(path.sep).join('_');
const output = path.join(root, '.vitest-reports', `${name}.json`);
const args = ['x', '--no-install', 'vitest', 'run', '--coverage', '--reporter=blob', `--outputFile=${output}`];
const result = spawnSync(process.execPath, args, {stdio: 'inherit'});
process.exitCode = result.status ?? 1;
