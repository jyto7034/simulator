import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const directory = path.dirname(fileURLToPath(import.meta.url));
const [label, cwd, command, ...args] = process.argv.slice(2);
if (!/^[a-z0-9_]+$/.test(label) || !cwd || !command) throw Error('label cwd command required');
const logPath = path.join(directory, `${label}.log`);
const receiptPath = path.join(directory, `${label}.json`);
if (fs.existsSync(logPath) || fs.existsSync(receiptPath)) throw Error('Evidence already exists');
const start = new Date().toISOString();
const output = fs.createWriteStream(logPath, {flags:'wx'});
const child = spawn(command, args, {cwd, env:process.env});
for (const stream of [child.stdout, child.stderr]) stream.on('data', data => output.write(data));
child.on('error', error => output.write(String(error)));
child.on('close', (code, signal) => output.end(() => {
  const log = fs.readFileSync(logPath);
  const receipt = {start, end:new Date().toISOString(), cwd, command:[command,...args], code, signal,
    log:path.basename(logPath), sha256:crypto.createHash('sha256').update(log).digest('hex')};
  fs.writeFileSync(receiptPath, JSON.stringify(receipt,null,2)+'\n', {flag:'wx'});
  console.log(JSON.stringify(receipt));
  console.log(log.toString().slice(-15000));
  process.exitCode = code ?? 1;
}));
