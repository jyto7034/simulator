import {spawn} from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
const here = path.dirname(new URL(import.meta.url).pathname);
const [tag, filter] = process.argv.slice(2);
if (!tag || !filter || !/^[a-z0-9_]+$/.test(tag)) throw Error('tag and filter required');
const args=['test','--offline','--locked','-p','game_core','--release','--lib',filter,'--','--nocapture','--test-threads=1'];
const logfile=path.join(here, tag+'.log');
const out=fs.createWriteStream(logfile, {flags:'wx'});
const started=new Date().toISOString();
const child=spawn('cargo',args,{cwd:'/tmp/r06-cap-review.2Wuy7u',env:{...process.env,CARGO_BUILD_JOBS:'4',CARGO_TARGET_DIR:'/tmp/r06-cap-review.2Wuy7u/target'}});
for (const stream of [child.stdout,child.stderr]) stream.on('data',b=>{out.write(b);process.stdout.write(b);});
child.on('close',code=>out.end(()=>{
 const bytes=fs.readFileSync(logfile);
 fs.writeFileSync(path.join(here,tag+'.json'),JSON.stringify({started,finished:new Date().toISOString(),command:['cargo',...args],cwd:'/tmp/r06-cap-review.2Wuy7u',exit_code:code,log_sha256:crypto.createHash('sha256').update(bytes).digest('hex')},null,2)+'\n',{flag:'wx'});
 process.exitCode=code;
}));
