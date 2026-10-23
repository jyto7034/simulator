import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const root='/mnt/f/work/simulator';
const harness='/tmp/r06-wp02-review.E34TyB';
const goal='core_enhanced/docs/goals/content_authoring_value_expressions_reassessment';
const review=goal+'/execution/single_goal/reviews/wp02_runtime_frame_capture_001_review_001';
const manifestPath=goal+'/execution/single_goal/segments/wp02_runtime_frame_capture_001/target/target_001/MANIFEST.json';
const hash=data=>crypto.createHash('sha256').update(data).digest('hex');
const manifest=JSON.parse(fs.readFileSync(path.join(root,manifestPath)));
const injection=`    include!("${root}/${review}/probes.rs");\n`;
const modifications=[];
for(const entry of manifest.current_source.files) {
  const current=fs.readFileSync(path.join(harness,entry.path));
  if(hash(current)===entry.sha256)continue;
  if(entry.path!=='core_enhanced/src/game/battle/core/value_expr_runtime.rs')throw Error('Unexpected harness delta: '+entry.path);
  const original=current.toString().replace(injection,'');
  if(hash(original)!==entry.sha256)throw Error('Harness changed more than test include');
  modifications.push({path:entry.path,frozen:entry.sha256,instrumented:hash(current),onlyChange:injection.trim()});
}
const artifacts=['probes_initial.rs','probes_before_production_death.rs','probes.rs','run.mjs','verify_review_target.py'].map(file=>({file,sha256:hash(fs.readFileSync(path.join(root,review,file)))}));
console.log(JSON.stringify({harness,baseline:manifestPath,sourceCount:manifest.current_source.files.length,
  modifications, supportFile:{path:'game_server/Cargo.toml',reason:'workspace resolution; missing from frozen archive; copied current bytes',
  sha256:hash(fs.readFileSync(path.join(harness,'game_server/Cargo.toml')))},artifacts,
  rustc:execFileSync('rustc',['--version'],{encoding:'utf8'}).trim(),cargo:execFileSync('cargo',['--version'],{encoding:'utf8'}).trim(),
  targetDirectory:'/mnt/f/work/simulator/target',productSourceMutation:false},null,2));
