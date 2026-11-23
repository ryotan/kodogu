import {spawnSync} from 'node:child_process';
import {rmSync} from 'node:fs';
import path from 'node:path';

// Runs `test:coverage:blob` in every frontend package and merges their blob reports into `coverage/` at the root of the
// repository. Written in TypeScript so that the task runs in the shell that mise picks on every operating system.

const root = path.resolve(import.meta.dirname, '..', '..');
const reports = path.join(root, '.vitest-reports');

function run(command: string, args: readonly string[]): void {
  const result = spawnSync(command, args, {cwd: root, stdio: 'inherit'});
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

rmSync(reports, {recursive: true, force: true});
run('mise', ['run', '--tag', 'frontend', '//...:test:coverage:blob']);
run(process.execPath, ['x', '--no-install', 'vitest', 'run', `--merge-reports=${reports}`, '--coverage']);
