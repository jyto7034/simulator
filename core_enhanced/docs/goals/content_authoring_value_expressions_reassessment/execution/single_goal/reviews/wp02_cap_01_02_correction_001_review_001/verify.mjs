import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
const root='/mnt/f/work/simulator';
const here=path.dirname(new URL(import.meta.url).pathname);
const segment=root+'/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_cap_01_02_001';
const target=segment+'/target/cap_01_02_correction_001';
const isolated='/tmp/r06-cap-review.2Wuy7u';
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const json=p=>JSON.parse(fs.readFileSync(p));
const run=(cmd,args)=>spawnSync(cmd,args,{encoding:'utf8',maxBuffer:32*1024*1024});
const manifest=json(target+'/MANIFEST.json');
assert.equal(hash(fs.readFileSync(target+'/MANIFEST.json')),'179bff1593608283c82ed51a2923f183d47312b8df51ad3dcb7556fdca8a4a5f');
const verified=run('python3',[segment+'/verification/verify_target.py']);
assert.equal(verified.status,0,verified.stderr);
const hook='\n#[cfg(test)]\n#[path = "cap_review_probes.rs"]\nmod cap_review_probes;\n';
for(const f of manifest.source.files){
 let bytes=fs.readFileSync(isolated+'/'+f.path);
 if(f.path.endsWith('/core/queue.rs')){
  assert.equal(bytes.toString().split(hook).length,2);
  bytes=Buffer.from(bytes.toString().replace(hook,''));
 }
 assert.equal(hash(bytes),f.sha256,f.path);
}
fs.copyFileSync(isolated+'/core_enhanced/src/game/battle/core/cap_review_probes.rs',here+'/probes.rs',fs.constants.COPYFILE_EXCL);
const before=isolated+'/before';
fs.mkdirSync(before);
assert.equal(run('tar',['-xzf',segment+'/baseline/entry_001/SOURCE_AND_INPUTS_BEFORE_CORRECTION.tar.gz','-C',before]).status,0);
const changed=json(target+'/SOURCE_DELTA.json').modified.map(f=>f.path);
const formatting={};
for(const [name,base]of [['before',before],['after',root]]){
 const r=run('rustfmt',['--edition','2021','--check','--color','never','--config','skip_children=true',...changed.map(p=>base+'/'+p)]);
 fs.writeFileSync(here+'/format_'+name+'.log',r.stdout+r.stderr,{flag:'wx'});
 formatting[name]={exit_code:r.status,hunks:(r.stdout.match(/^Diff in /gm)||[]).length};
}
const summary={target:JSON.parse(verified.stdout),isolated_source_files_verified:manifest.source.files.length,instrumentation:'one queue.rs cfg(test) module; probes.rs archived here',probe_sha256:hash(fs.readFileSync(here+'/probes.rs')),formatting};
fs.writeFileSync(here+'/verification.json',JSON.stringify(summary,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(summary,null,2));
