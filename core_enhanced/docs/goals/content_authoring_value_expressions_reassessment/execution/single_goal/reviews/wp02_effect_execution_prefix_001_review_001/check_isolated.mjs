import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
const root='/mnt/f/work/simulator';
const here=path.dirname(new URL(import.meta.url).pathname);
const isolated='/tmp/r06-prefix-review.WcaQXG';
const target=root+'/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp02_effect_execution_prefix_001/target/effect_execution_prefix_001';
const m=JSON.parse(fs.readFileSync(target+'/MANIFEST.json'));
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const hook='    #[path = "/tmp/r06-prefix-review.WcaQXG/core_enhanced/src/game/battle/core/prefix_review_probes.rs"]\n    mod prefix_review_probes;\n';
for(const f of m.source.files){
 let b=fs.readFileSync(isolated+'/'+f.path);
 if(f.path.endsWith('/core/value_expr_runtime.rs')){
  assert.equal(b.toString().split(hook).length,2);
  b=Buffer.from(b.toString().replace(hook,''));
 }
 assert.equal(hash(b),f.after.sha256,f.path);
}
const probe=fs.readFileSync(isolated+'/core_enhanced/src/game/battle/core/prefix_review_probes.rs');
fs.writeFileSync(here+'/probes.rs',probe,{flag:'wx'});
fs.writeFileSync(here+'/isolated_identity.json',JSON.stringify({source_files_verified:m.source.file_count,instrumentation:'one private test-module hook and probes.rs',probe_sha256:hash(probe)},null,2)+'\n',{flag:'wx'});
console.log('333 source verified; final probe preserved');
