import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const site = fileURLToPath(new URL('../', import.meta.url));
const script = fileURLToPath(new URL('./export_content.py', import.meta.url));
const candidates = process.platform === 'win32'
  ? [['py', ['-3']], ['python', []], ['python3', []]]
  : [['python3', []], ['python', []]];

for (const [command, args] of candidates) {
  // Execute a probe: Windows Store aliases may exist without a Python runtime.
  const probe = spawnSync(command, [...args, '-X', 'utf8', '-c',
    'import sys; sys.exit(0 if sys.version_info >= (3, 9) else 1)'],
  { cwd: site, stdio: 'ignore', timeout: 10000, windowsHide: true });
  if (probe.error || probe.status !== 0) continue;

  console.log(`Generating wiki content with ${command}${args.length ? ' ' + args.join(' ') : ''}`);
  const result = spawnSync(command, [...args, '-X', 'utf8', script],
    { cwd: site, stdio: 'inherit', windowsHide: true });
  if (result.error) console.error(result.error.message);
  // A content error must remain a failure; do not retry with another interpreter.
  process.exit(result.status ?? 1);
}

console.error('Python 3.9+ was not found. Install Python, reopen the terminal, and run npm run dev again.');
console.error('On Windows, verify the installation with: py -3 --version');
process.exit(1);
