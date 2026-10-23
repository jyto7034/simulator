import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
const root='/mnt/f/work/simulator';
const dir=path.dirname(new URL(import.meta.url).pathname);
const isolated='/tmp/r06-f01-r01-review.d7WUvr';
const segment=path.join(root,'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_f01_r01_001');
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const manifestBytes=fs.readFileSync(path.join(segment,'target/f01_r01_correction_001/MANIFEST.json'));
assert.equal(hash(manifestBytes),'ba25a8fbedfeb4b43c13d8f3bce88340e3ffafc4eb6dba2424169028bad7be98');
const manifest=JSON.parse(manifestBytes);
const hook=`\n    #[path = "${dir}/probes.rs"]\n    mod reviewer_probes;\n`;
let instrumentationFiles=0;
for(const item of manifest.source.files){
  let bytes=fs.readFileSync(path.join(isolated,item.path));
  if(hash(bytes)!==item.sha256){
    assert.equal(item.path,'core_enhanced/src/game/battle/core/value_expr_runtime.rs');
    const text=bytes.toString();
    assert.equal(text.split(hook).length,2);
    bytes=Buffer.from(text.replace(hook,''));
    instrumentationFiles++;
  }
  assert.equal(bytes.length,item.bytes,item.path);
  assert.equal(hash(bytes),item.sha256,item.path);
}
assert.equal(instrumentationFiles,1);
const predecessorPath=path.join(root,manifest.predecessor_target.path);
const predecessor=JSON.parse(fs.readFileSync(predecessorPath));
const predecessorArchives=['source','evidence'].map(key=>{
  const expected=predecessor[key].archive;
  const bytes=fs.readFileSync(path.join(path.dirname(predecessorPath),expected.path));
  assert.equal(bytes.length,expected.bytes);assert.equal(hash(bytes),expected.sha256);
  return {kind:key,...expected};
});
const command=['python3',path.join(segment,'verification/verify_target.py')];
const result=spawnSync(command[0],command.slice(1),{cwd:root,encoding:'utf8'});
assert.equal(result.status,0,result.stderr);
const currentness=JSON.parse(result.stdout);
const original=fs.readFileSync(path.join(segment,'verification/probes.rs'));
assert.ok(fs.readFileSync(path.join(dir,'probes.rs')).subarray(0,original.length).equals(original),'producer probes altered');
const artifacts=fs.readdirSync(dir).filter(name=>name!=='verification.json').sort().map(name=>{
  const bytes=fs.readFileSync(path.join(dir,name));
  return {path:name,bytes:bytes.length,sha256:hash(bytes)};
});
const receipt={timestamp:new Date().toISOString(),manifest_sha256:hash(manifestBytes),isolated_source:isolated,verified_source_files:manifest.source.files.length,instrumentation_only_files:instrumentationFiles,producer_probes_preserved_sha256:hash(original),predecessorArchives,currentness_command:command,currentness,review_artifacts:artifacts};
fs.writeFileSync(path.join(dir,'verification.json'),JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(receipt,null,2));
