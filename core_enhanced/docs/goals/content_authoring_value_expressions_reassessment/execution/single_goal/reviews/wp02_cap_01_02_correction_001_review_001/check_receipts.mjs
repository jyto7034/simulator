import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
const here=path.dirname(new URL(import.meta.url).pathname);
const base=path.resolve(here,'../../corrections/wp02_cap_01_02_001/verification');
const read=n=>JSON.parse(fs.readFileSync(base+'/'+n));
const family=r=>r.family==='battle'?(r.chunk_ms?'literal_battle_public_step':'damage_burst_private_transaction'):r.family;
const key=r=>JSON.stringify([family(r),r.enemies??r.target_equivalent??r.members,r.chunk_ms??null,r.burst_sources??r.source_equivalent??r.repetitions??null,r.reads_per_expr??null,r.grouped_per_tile??null]);
const error=r=>r.error??r.setup_error??null;
const grouped={};
for(const name of ['before','after']){
 const rows=read('pc_'+name+'_raw.json').rows;
 assert.equal(rows.length,576);
 const groups=new Map();
 for(const r of rows){const k=key(r);if(!groups.has(k))groups.set(k,[]);groups.get(k).push(r);}
 assert.equal(groups.size,36);
 for(const rs of groups.values()){
  assert.equal(rs.length,16);
  assert.equal(rs.filter(r=>r.warmup).length,2);
  assert.equal(rs.filter(r=>!r.warmup&&(r.measurement??r.setup).allocation_observer).length,3);
  assert.equal(new Set(rs.map(error)).size,1);
 }
 grouped[name]=groups;
}
assert.deepEqual([...grouped.before.keys()].sort(),[...grouped.after.keys()].sort());
const rejected={};
for(const name of ['before','after'])rejected[name]=[...grouped[name].values()].filter(rs=>error(rs[0])).length;
assert.deepEqual(rejected,{before:11,after:5});
let preserved=0;
for(const [k,rs]of grouped.after){
 const old=grouped.before.get(k)[0];
 for(const r of rs){
  if(r.error){assert.match(r.error,/value_expr_evaluation_retained/);continue;}
  if(r.family==='damage_burst_private_transaction'){
   assert.equal(r.hp_loss,r.enemies*r.burst_sources);
   assert.equal(r.final_facts.damage_mutations-r.initial_facts.damage_mutations,r.enemies*r.burst_sources);
  }
  if(r.enemies===500&&r.units!==undefined)assert.equal(r.units,512);
  if(!error(old)&&old.after_log_sha256){assert.equal(r.after_log_sha256,old.after_log_sha256);assert.equal(r.hp_loss,old.hp_loss);}
 }
 if(!error(old)&&old.after_log_sha256)preserved++;
}
const r05=read('r05_guard_final_exact.json');
const peak=Math.max(...r05.concurrent_sessions.allocation_raw_samples.map(r=>r.heap_allocation.peak_delta_bytes));
assert.equal(r05.concurrent_sessions.allocation_retained_peak_guard_pass,true);
assert.ok(peak<=9030188);
const result={kind:'independent reanalysis of producer receipts; not new PC/R05 measurement',conditions:36,samples_per_condition:16,timing_samples:11,allocation_samples:3,warmups:2,rejected,preserved_completed_battle_conditions:preserved,r05_concurrent_peak:peak,r05_cap:9030188,r05_headroom:9030188-peak};
fs.writeFileSync(here+'/receipt_analysis.json',JSON.stringify(result,null,2)+'\n',{flag:'wx'});
console.log(result);
