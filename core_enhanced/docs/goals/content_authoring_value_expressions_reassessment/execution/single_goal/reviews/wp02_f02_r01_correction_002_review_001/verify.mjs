import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
const root='/mnt/f/work/simulator';
const dir=path.dirname(new URL(import.meta.url).pathname);
const isolated='/tmp/r06-f02-r01-audit.SDwxhw';
const base=path.join(root,'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal');
const segment=path.join(base,'corrections/wp02_f02_r01_001');
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const bytes=fs.readFileSync(path.join(segment,'target/f02_r01_correction_002/MANIFEST.json'));
assert.equal(hash(bytes),'ae2c540157c9caa769febb56e2664da7be8b5807b917e4ecfcb685103aa5f0f7');
const manifest=JSON.parse(bytes);
const names=['original_probes.rs','cast_probes.rs','producer_probes.rs','additional_probes.rs'];
const hook='\n    mod reviewer_probes {\n        use super::*;\n'+names.map(n=>`        include!("${dir}/${n}");\n`).join('')+'    }\n';
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
const sourceSet=new Set(manifest.source.files.map(i=>i.path));
function walk(relative){for(const entry of fs.readdirSync(path.join(root,relative),{withFileTypes:true})){
  const next=path.posix.join(relative,entry.name);if(entry.isDirectory())walk(next);else if(entry.isFile())assert.ok(sourceSet.has(next),`unmanifested source: ${next}`);
}}
for(const subtree of ['core_enhanced/src','core_enhanced/tests','game_resources/data'])walk(subtree);
const mappings={
  'original_probes.rs':'reviews/wp02_f02_f03_correction_001_review_001/original_probes.rs',
  'cast_probes.rs':'reviews/wp02_f02_f03_correction_001_review_001/additional_probes.rs',
  'producer_probes.rs':'corrections/wp02_f02_r01_001/verification/f02_r01_probes.rs',
};
for(const [local,original] of Object.entries(mappings))assert.ok(fs.readFileSync(path.join(dir,local)).equals(fs.readFileSync(path.join(base,original))));
const first=manifest.supersedes_failed_integrity_candidate;
const firstBytes=fs.readFileSync(path.join(root,first.path));assert.equal(firstBytes.length,first.bytes);assert.equal(hash(firstBytes),first.sha256);
const predecessorPath=path.join(root,manifest.predecessor.path);
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
const receipt={timestamp:new Date().toISOString(),target_sha256:hash(bytes),isolated,source_files_verified:manifest.source.files.length,instrumentation_only_files:changed,reused_probe_bytes_verified:mappings,first_candidate_manifest_verified:true,predecessor_archives_verified:true,currentness_command:command,currentness:JSON.parse(result.stdout),artifacts};
fs.writeFileSync(path.join(dir,'verification.json'),JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(receipt,null,2));
