import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
const root='/mnt/f/work/simulator';
const here=path.dirname(new URL(import.meta.url).pathname);
const dir=fs.mkdtempSync('/tmp/r06-eep-correction.');
const baseline=JSON.parse(fs.readFileSync(here+'/baseline/entry_001/MANIFEST.json'));
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const identities=[];
for(const f of baseline.source.files){
 const source=fs.readFileSync(root+'/'+f.path);fs.mkdirSync(path.dirname(dir+'/'+f.path),{recursive:true});fs.writeFileSync(dir+'/'+f.path,source);
 identities.push({path:f.path,sha256:hash(source)});
}
const review=root+'/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_effect_execution_prefix_001_review_001/probes.rs';
const original=fs.readFileSync(review,'utf8');
// Share the PC probe allocator; the original five reviewer test bodies remain byte-identical.
let probes=original.slice(original.indexOf('#[test]'));
probes='use super::*;\nuse std::cell::Cell;\nthread_local! { static TRACK: Cell<(bool,usize,usize)> = const { Cell::new((false,0,0)) }; }\npub(super) fn requested(n:usize) { let _=TRACK.try_with(|s|{let(a,v,m)=s.get();if a{s.set((a,v+n,m.max(n)))}}); }\n'+probes;
probes+=fs.readFileSync(here+'/boundary_probes.rs','utf8');
fs.writeFileSync(dir+'/review_probes.rs',probes);
const pcPath=root+'/core_enhanced/docs/tmp/r06_pc_baseline_20260926/probe.rs';
const pc=fs.readFileSync(pcPath,'utf8').replace('fn requested(bytes: usize) {','fn requested(bytes: usize) {\n    super::review_probes::requested(bytes);');
fs.writeFileSync(dir+'/pc_probe.rs',pc);
const adapter='core_enhanced/src/game/battle/core/value_expr_runtime.rs';
const native='core_enhanced/src/game/battle/runtime_control.rs';
const text=fs.readFileSync(dir+'/'+adapter,'utf8').trimEnd();
if(!text.endsWith('}'))throw Error('module boundary missing');
fs.writeFileSync(dir+'/'+adapter,text.slice(0,-1)+`\n#[path = "${dir}/review_probes.rs"] mod review_probes;\n#[path = "${dir}/pc_probe.rs"] mod load_baseline;\n}\n`);
fs.appendFileSync(dir+'/'+native,'\n#[cfg(test)]\npub(crate) fn baseline_usage() -> Option<(u64, usize, Option<usize>)> {\n ACTIVE.with(|slot| slot.borrow().as_ref().map(|b| (b.work, b.clone_slots, b.retained_admission)))\n}\n');
fs.mkdirSync(here+'/evidence',{recursive:true});
fs.writeFileSync(here+'/evidence/isolate_identity.json',JSON.stringify({dir,source:identities,instrumented:[adapter,native],review_original_sha256:hash(original),review_instrumented_sha256:hash(probes),pc_original_sha256:hash(fs.readFileSync(pcPath)),pc_instrumented_sha256:hash(pc)},null,2)+'\n',{flag:'wx'});
console.log(dir);
