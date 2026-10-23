import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawn} from 'node:child_process';
const dir=path.dirname(new URL(import.meta.url).pathname);
const cwd='/tmp/r06-f01-audit.oZdgjJ';
const [tag,filter]=process.argv.slice(2);
if(!tag||!filter)throw Error('tag and filter required');
const args=['test','--manifest-path',`${cwd}/core_enhanced/Cargo.toml`,'--offline','--release','--lib',filter,'--','--nocapture','--test-threads=1'];
const log=fs.createWriteStream(path.join(dir,`${tag}.log`),{flags:'wx'});
const start=new Date().toISOString();
const child=spawn('cargo',args,{cwd,env:{...process.env,CARGO_TARGET_DIR:`${cwd}/target`,CARGO_BUILD_JOBS:'4'},stdio:['ignore','pipe','pipe']});
for(const stream of [child.stdout,child.stderr])stream.on('data',b=>{log.write(b);process.stdout.write(b)});
child.on('error',e=>{log.write(String(e));console.error(e)});
child.on('close',(code,signal)=>{log.end();fs.writeFileSync(path.join(dir,`${tag}.json`),JSON.stringify({start,end:new Date().toISOString(),cwd,command:['cargo',...args],code,signal,
  probe_sha256:crypto.createHash('sha256').update(fs.readFileSync(path.join(dir,'probes.rs'))).digest('hex')},null,2)+'\n',{flag:'wx'});process.exitCode=code??1;});
