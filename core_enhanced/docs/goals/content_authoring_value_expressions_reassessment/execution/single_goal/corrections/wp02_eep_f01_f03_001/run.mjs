import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawn} from 'node:child_process';
const here=path.dirname(new URL(import.meta.url).pathname);
const [tag,command,...args]=process.argv.slice(2);
const cwd=process.env.CORRECTION_CWD||'/mnt/f/work/simulator';
const dir=here+'/evidence';fs.mkdirSync(dir,{recursive:true});
const baseline=JSON.parse(fs.readFileSync(here+'/baseline/entry_001/MANIFEST.json'));
const identity=()=>baseline.source.files.map(f=>({path:f.path,sha256:crypto.createHash('sha256').update(fs.readFileSync('/mnt/f/work/simulator/'+f.path)).digest('hex')}));
const source_before=identity(),start=new Date().toISOString();
const log=fs.createWriteStream(dir+'/'+tag+'.log',{flags:'wx'});
const env={...process.env,CARGO_BUILD_JOBS:'4',CARGO_TARGET_DIR:process.env.CORRECTION_TARGET||'/tmp/r06-wp02-validation-target'};
const child=spawn(command,args,{cwd,env});
for(const s of [child.stdout,child.stderr])s.on('data',b=>{log.write(b);process.stdout.write(b)});
child.on('error',error=>log.write(String(error)));
child.on('close',code=>log.end(()=>{
 fs.writeFileSync(dir+'/'+tag+'.json',JSON.stringify({cwd,command:[command,...args],start,end:new Date().toISOString(),exit_code:code,env:{CARGO_BUILD_JOBS:env.CARGO_BUILD_JOBS,CARGO_TARGET_DIR:env.CARGO_TARGET_DIR},source_before,source_after:identity()},null,2)+'\n',{flag:'wx'});process.exitCode=code??1;
}));
