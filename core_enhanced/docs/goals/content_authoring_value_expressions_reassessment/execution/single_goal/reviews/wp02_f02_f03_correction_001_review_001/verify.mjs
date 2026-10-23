import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
const root='/mnt/f/work/simulator';
const dir=path.dirname(new URL(import.meta.url).pathname);
const isolated='/tmp/r06-f02-f03-review.5nvNWT';
const base=path.join(root,'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal');
const segment=path.join(base,'corrections/wp02_f02_f03_001');
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const bytes=fs.readFileSync(path.join(segment,'target/f02_f03_correction_001/MANIFEST.json'));
assert.equal(hash(bytes),'84a7b1a6f4d40973c5fbc148c00457911e966a20f046e4aa6afbeac609897b23');
const manifest=JSON.parse(bytes);
const hook=`\n    mod reviewer_probes {\n        use super::*;\n        include!("${dir}/original_probes.rs");\n        include!("${dir}/additional_probes.rs");\n    }\n`;
let changed=0;
for(const item of manifest.source.files){
  let payload=fs.readFileSync(path.join(isolated,item.path));
  if(hash(payload)!==item.sha256){
    assert.equal(item.path,'core_enhanced/src/game/battle/core/value_expr_runtime.rs');
    const text=payload.toString();assert.equal(text.split(hook).length,2);
    payload=Buffer.from(text.replace(hook,''));changed++;
  }
  assert.equal(payload.length,item.bytes,item.path);assert.equal(hash(payload),item.sha256,item.path);
}
assert.equal(changed,1);
const original=fs.readFileSync(path.join(base,'reviews/wp02_runtime_frame_capture_001_review_001/probes.rs'));
assert.ok(fs.readFileSync(path.join(dir,'original_probes.rs')).equals(original));
const predecessorPath=path.join(root,manifest.predecessor_target.path);
const predecessor=JSON.parse(fs.readFileSync(predecessorPath));
for(const key of ['source','evidence']){
  const item=predecessor[key].archive;
  const payload=fs.readFileSync(path.join(path.dirname(predecessorPath),item.path));
  assert.equal(payload.length,item.bytes);assert.equal(hash(payload),item.sha256);
}
const command=['python3',path.join(segment,'verification/verify_target.py')];
const result=spawnSync(command[0],command.slice(1),{cwd:root,encoding:'utf8'});
assert.equal(result.status,0,result.stderr);
const artifacts=fs.readdirSync(dir).filter(n=>n!=='verification.json').sort().map(n=>{const b=fs.readFileSync(path.join(dir,n));return {path:n,bytes:b.length,sha256:hash(b)};});
const receipt={timestamp:new Date().toISOString(),target_sha256:hash(bytes),isolated,source_files_verified:manifest.source.files.length,instrumentation_only_files:changed,original_probe_sha256:hash(original),predecessor_archives_verified:true,currentness_command:command,currentness:JSON.parse(result.stdout),artifacts};
fs.writeFileSync(path.join(dir,'verification.json'),JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(receipt,null,2));
