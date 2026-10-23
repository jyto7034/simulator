import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
const root='/mnt/f/work/simulator';
const here=path.dirname(new URL(import.meta.url).pathname);
const segment=root+'/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_cap_01_02_001';
const before='/tmp/r06-cap-format-review.BXS2CH';
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const json=p=>JSON.parse(fs.readFileSync(p));
const run=(cmd,args)=>spawnSync(cmd,args,{encoding:'utf8',timeout:30000,maxBuffer:32*1024*1024});
const format=file=>{
 const r=run('rustfmt',['--edition','2021','--emit','stdout','--config','skip_children=true',file]);
 assert.equal(r.status,0,r.stderr);
 const prefix=file+':\n\n';
 assert.ok(r.stdout.startsWith(prefix));
 return r.stdout.slice(prefix.length);
};
const target=segment+'/target/cap_01_02_correction_002';
const previous=segment+'/target/cap_01_02_correction_001';
assert.equal(hash(fs.readFileSync(target+'/MANIFEST.json')),'6ee8ea5a889428eef4a0986cd83037ef8c51de6c751868ef9657adb11d3ce789');
assert.equal(hash(fs.readFileSync(previous+'/MANIFEST.json')),'179bff1593608283c82ed51a2923f183d47312b8df51ad3dcb7556fdca8a4a5f');
const integrity=run('python3',[segment+'/verification/verify_format_target.py']);
assert.equal(integrity.status,0,integrity.stderr);
const original=json(previous+'/MANIFEST.json');
const changed=[];
for(const f of original.source.files){
 const old=fs.readFileSync(before+'/'+f.path);
 assert.equal(hash(old),f.sha256,f.path);
 const current=fs.readFileSync(root+'/'+f.path);
 if(old.equals(current))continue;
 const a=format(before+'/'+f.path),b=format(root+'/'+f.path);
 assert.equal(a,b,f.path+': canonical rustfmt output differs');
 const diff=run('diff',['-u',before+'/'+f.path,root+'/'+f.path]);
 assert.equal(diff.status,1);
 fs.writeFileSync(here+'/'+path.basename(f.path)+'.diff',diff.stdout,{flag:'wx'});
 changed.push({path:f.path,before_sha256:hash(old),after_sha256:hash(current),canonical_rustfmt_sha256:hash(a)});
}
assert.equal(changed.length,5);
const fmt=run('rustfmt',['--edition','2021','--check','--color','never','--config','skip_children=true',...changed.map(f=>root+'/'+f.path)]);
assert.equal(fmt.status,1);
fs.writeFileSync(here+'/focused_rustfmt.log',fmt.stdout+fmt.stderr,{flag:'wx'});
const counts={};
for(const line of fmt.stdout.split('\n'))if(line.startsWith('Diff in ')){
 const name=line.match(/\/([^/:]+\.rs):\d+:/)[1];counts[name]=(counts[name]??0)+1;
}
assert.deepEqual(counts,{'control.rs':4,'queue.rs':25,'runtime_control.rs':6});
const testlog=fs.readFileSync(segment+'/verification/format_002_all_target_test.log','utf8');
const results=[...testlog.matchAll(/test result: ok\. (\d+) passed; (\d+) failed/g)].map(m=>({passed:Number(m[1]),failed:Number(m[2])}));
assert.equal(results.reduce((n,r)=>n+r.passed,0),1475);
assert.ok(results.every(r=>r.failed===0));
const result={scope:'format-only follow-up review',integrity:JSON.parse(integrity.stdout),prior_source_archive_files_verified:original.source.files.length,changed,canonical_rustfmt_equal_for_all_changed_files:true,focused_rustfmt:{exit_code:fmt.status,remaining_hunks:counts,total:35},tests:{producer_log_reused:true,passed:1475,reviewer_tests_rerun:false},rustfmt_version:run('rustfmt',['--version']).stdout.trim()};
fs.writeFileSync(here+'/verification.json',JSON.stringify(result,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(result,null,2));
