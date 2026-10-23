import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';

const root='/mnt/f/work/simulator';
const dir=path.dirname(new URL(import.meta.url).pathname);
const isolated='/tmp/r06-f01-audit.oZdgjJ';
const segment=path.join(root,'core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_f01_f03_cap_001');
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const manifestBytes=fs.readFileSync(path.join(segment,'target/f01_admission_001/MANIFEST.json'));
assert.equal(hash(manifestBytes),'8ac6a2c6197ce19e1e841d52128758b14b8079762a11536620131884fa344370');
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
const support=['game_server/Cargo.toml','game_server/src/main.rs'].map(relative=>{
  const bytes=fs.readFileSync(path.join(isolated,relative));
  assert.equal(hash(bytes),hash(fs.readFileSync(path.join(root,relative))));
  return {path:relative,bytes:bytes.length,sha256:hash(bytes),purpose:'workspace manifest resolution only; game_server not compiled'};
});
const command=['python3',path.join(segment,'verification/verify_target.py')];
const result=spawnSync(command[0],command.slice(1),{cwd:root,encoding:'utf8'});
assert.equal(result.status,0,result.stderr);
const currentness=JSON.parse(result.stdout);
const artifacts=fs.readdirSync(dir).filter(name=>name!=='verification.json').sort().map(name=>{
  const bytes=fs.readFileSync(path.join(dir,name));
  return {path:name,bytes:bytes.length,sha256:hash(bytes)};
});
const receipt={timestamp:new Date().toISOString(),manifest_sha256:hash(manifestBytes),isolated_source:isolated,verified_source_files:manifest.source.files.length,instrumentation_only_files:instrumentationFiles,build_support:support,currentness_command:command,currentness,review_artifacts:artifacts};
fs.writeFileSync(path.join(dir,'verification.json'),JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(receipt,null,2));
