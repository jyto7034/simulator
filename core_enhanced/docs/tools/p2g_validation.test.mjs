import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";
import {analyze,generateReceipt,validateReceipt} from "./validate_p2g40_receipt.mjs";
import {readFreeze,readPlan} from "./p2g_inputs.mjs";

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),"../../..");
const tool40=path.join(root,"core_enhanced/docs/tools/validate_p2g40_receipt.mjs");
const tool50=path.join(root,"core_enhanced/docs/tools/validate_p2g50_goal_set.mjs");
const ref=(type,id)=>({"Artifact Type":type,"Artifact ID":id,"Content Digest":null});
const artifact=(type,id,payload)=>({ref:ref(type,id),payload});
const write=(file,value)=>fs.writeFileSync(file,JSON.stringify(value,null,2)+"\n");
function fixture(t) {
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),"p2g-validation-"));
  t.after(()=>fs.rmSync(dir,{recursive:true,force:true}));
  const f=ref("Final Policy Freeze","freeze-fixture");
  const w=ref("Working Revision","working-fixture");
  const rb=ref("Review Batch","review-fixture");
  const sb=ref("Reviewed Source Basis","source-fixture");
  const binding=ref("Code Review Binding","binding-fixture");
  const elements=["작성.효과", "second_element"];
  const members=elements.map((id,i)=>artifact("Policy Element Revision",`opaque-revision-${i}`,{
    "Stable Element ID":id,"Intrinsic status":"code_reviewed",
    "Decision Revision Ref":ref("Decision Revision",`decision-${i}`),"Observable Policy Acceptance":[`acceptance ${i}`]}));
  const source={artifacts:[{ref:w,payload:{"Current Policy Element Revision Map":Object.fromEntries(members.map(m=>[m.payload["Stable Element ID"],m.ref]))}},
    ...members,...elements.map((_,i)=>artifact("Decision Revision",`decision-${i}`,{})),
    {ref:rb,payload:{}},{ref:sb,payload:{}},{ref:binding,payload:{}}]};
  const freeze={artifacts:[{ref:f,payload:{"Working Revision Ref":w,"Review Batch Ref":rb,"Reviewed Source Basis Ref":sb,
    "Current Code Review Binding Ref Set":[binding],"Decision Revision Ref Set":members.map(m=>m.payload["Decision Revision Ref"]),
    "Ordered code_reviewed Policy Element Revision Ref Set":members.map(m=>m.ref),
    "Observable Policy Acceptance Set":members.map(m=>({"Policy Element Revision Ref":m.ref,"Observable Policy Acceptance":m.payload["Observable Policy Acceptance"]}))}}]};
  const plan={artifacts:[artifact("Phase Plan","plan-fixture",{
    "Bound Final Policy Freeze Ref":f,"Exact Prerequisite Composite Basis":{"Working Revision Ref":w,"Final Policy Freeze Ref":f},
    "Applicable P2G-40 Obligation Key Set":[],"P2G-40 Resolution Record Ref Set":[],"Applicable Obligation Supersession Record Ref Set":[],
    "Goal Structure Fact":"multi_phase",Parallelization:"none approved",
    work_packages:[{id:"producer",selectors:{implementation:[elements[0]]}},{id:"consumer",selectors:{implementation:[elements[1]]}}],
    dependencies:[{from:"producer",to:"consumer",type:"producer_consumer"}],atomic_groups:[],standalone:["producer","consumer"],cyclic_sccs:[],
    phases:[{id:"first",primary:[elements[0]],nodes:["producer"]},{id:"second",primary:[elements[1]],nodes:["consumer"]}],
    audits:[{phase:"first"},{phase:"second"}],integration:[{id:"integration",disposition:"assigned_execution_node",nodes:["consumer"],phases:["second"]}]
  })],validation_receipt_ref:ref("Phase Plan Validation Receipt","receipt-fixture"),validator_method_version:"p2g40-receipt-validator/1.2.0"};
  const files={plan:path.join(dir,"plan.json"),freeze:path.join(dir,"freeze.json"),source:path.join(dir,"source.json"),receipt:path.join(dir,"receipt.json")};
  write(files.plan,plan);write(files.freeze,freeze);write(files.source,source);
  return {...files,dir,planData:plan,freezeData:freeze,sourceData:source,p:plan.artifacts[0].payload,f:freeze.artifacts[0].payload};
}
function rejects(f) {
  try {assert.notEqual(generateReceipt(f.plan,f.freeze,f.receipt).validation.result,"passed");}
  catch(e){if(e.code==="ERR_ASSERTION")throw e;}
  assert.equal(fs.existsSync(f.receipt),false);
}

test("explicit Unicode element IDs and opaque revisions; CLI generation/default/idempotence",t=>{
  const f=fixture(t),run=(args)=>spawnSync(process.execPath,[tool40,...args,"--plan",f.plan,"--freeze",f.freeze,"--receipt",f.receipt],{encoding:"utf8",cwd:root});
  const made=run(["--write-receipt"]);assert.ifError(made.error);assert.equal(made.status,0,made.stderr);assert.equal(JSON.parse(made.stdout).status,"created");
  assert.equal(run([]).status,0);
  const bytes=fs.readFileSync(f.receipt),time=fs.statSync(f.receipt).mtimeMs;
  assert.equal(JSON.parse(run(["--write-receipt"]).stdout).status,"unchanged");assert.deepEqual(fs.readFileSync(f.receipt),bytes);assert.equal(fs.statSync(f.receipt).mtimeMs,time);
  const receipt=JSON.parse(bytes);assert.deepEqual(receipt.selector_resolution.exact_resolution.map(x=>x.element_id).sort(),["second_element","작성.효과"].sort());
  assert.equal(receipt.structural_input_evidence.resolved_artifact_sources.length,2);
});

const badPlans={
  "missing primary":p=>p.phases[1].primary=[],
  "duplicate primary":p=>p.phases[1].primary.push(p.phases[0].primary[0]),
  "unknown selector":p=>p.work_packages[0].selectors.implementation.push("unapproved"),
  "dangling edge":p=>p.dependencies[0].to="missing",
  "cycle without atomic owner":p=>p.dependencies.push({from:"consumer",to:"producer"}),
  "reversed Phase order":p=>p.phases.reverse(),
  "duplicate execution owner":p=>p.phases[1].nodes.push("producer"),
  "unknown group member":p=>{p.atomic_groups=[{id:"group",members:["producer","missing"]}];},
  "overlapping groups":p=>{p.atomic_groups=[{id:"g1",members:["producer"]},{id:"g2",members:["producer"]}];},
  "missing audit":p=>p.audits.pop(),
  "integration wrong existing Phase":p=>p.integration[0].phases=["first"],
  "integration missing node":p=>p.integration[0].nodes=[],
  "empty integration map":p=>p.integration=[],
  "missing explicit repair set":p=>delete p["P2G-40 Resolution Record Ref Set"],
  "wrong prerequisite revision":p=>p["Exact Prerequisite Composite Basis"]["Working Revision Ref"]["Artifact ID"]="other",
  "wrong Goal structure":p=>p["Goal Structure Fact"]="single_phase"
};
for(const [name,mutate]of Object.entries(badPlans))test(`reject Plan: ${name}`,t=>{const f=fixture(t);mutate(f.p);write(f.plan,f.planData);rejects(f);});

const badFreeze={
  "missing member":f=>f.f["Ordered code_reviewed Policy Element Revision Ref Set"].pop(),
  "duplicate member":f=>f.f["Ordered code_reviewed Policy Element Revision Ref Set"].push(f.f["Ordered code_reviewed Policy Element Revision Ref Set"][0]),
  "wrong typed Ref":f=>f.f["Working Revision Ref"]["Artifact Type"]="Decision Revision",
  "wrong digest":f=>f.f["Working Revision Ref"]["Content Digest"]="different",
  "missing artifact":f=>f.sourceData.artifacts.pop(),
  "ambiguous artifact":f=>f.sourceData.artifacts.push(structuredClone(f.sourceData.artifacts[0])),
  "duplicate stable ID":f=>f.sourceData.artifacts[2].payload["Stable Element ID"]=f.sourceData.artifacts[1].payload["Stable Element ID"],
  "not reviewed":f=>f.sourceData.artifacts[1].payload["Intrinsic status"]="open",
  "changed acceptance projection":f=>f.f["Observable Policy Acceptance Set"][0]["Observable Policy Acceptance"]=["changed"],
  "reference cycle":f=>f.sourceData.artifacts.at(-1).payload.ref=f.freezeData.artifacts[0].ref
};
for(const[name,mutate]of Object.entries(badFreeze))test(`reject Freeze: ${name}`,t=>{const f=fixture(t);mutate(f);write(f.freeze,f.freezeData);write(f.source,f.sourceData);rejects(f);});

test("same Ref but changed source bytes invalidate Receipt",t=>{
  const f=fixture(t);generateReceipt(f.plan,f.freeze,f.receipt);
  f.sourceData.artifacts[0].payload.note="changed exact input";write(f.source,f.sourceData);
  assert.equal(validateReceipt(f.plan,f.freeze,f.receipt).result,"failed");
});
test("tampered counts and immutable target cannot be blessed or overwritten",t=>{
  const f=fixture(t);generateReceipt(f.plan,f.freeze,f.receipt);
  const r=JSON.parse(fs.readFileSync(f.receipt));r.primary_phase_coverage.missing_count=1;write(f.receipt,r);
  const bytes=fs.readFileSync(f.receipt);assert.equal(validateReceipt(f.plan,f.freeze,f.receipt).result,"failed");
  assert.throws(()=>generateReceipt(f.plan,f.freeze,f.receipt),/IMMUTABLE_RECEIPT/);assert.deepEqual(fs.readFileSync(f.receipt),bytes);
});
test("atomic SCC condenses into one node and Phase",t=>{
  const f=fixture(t);f.p.dependencies.push({from:"consumer",to:"producer"});
  f.p.atomic_groups=[{id:"atomic",members:["producer","consumer"]}];f.p.standalone=[];
  f.p.cyclic_sccs=[{members:["producer","consumer"],disposition:"coimplement_atomic"}];
  f.p.phases=[{id:"only",primary:["작성.효과","second_element"],nodes:["atomic"]}];f.p.audits=[{phase:"only"}];
  f.p.integration[0].nodes=["atomic"];f.p.integration[0].phases=["only"];f.p["Goal Structure Fact"]="single_phase";write(f.plan,f.planData);
  assert.equal(generateReceipt(f.plan,f.freeze,f.receipt).validation.result,"passed");assert.equal(validateReceipt(f.plan,f.freeze,f.receipt).result,"passed");
});

const topic=path.join(root,"core_enhanced/docs/goals/content_authoring_minimal_effect_contract");
test("actual Topic: six exact members and all Phase nodes resolve without renaming",()=>{
  const m=analyze(path.join(topic,"P2G_40_PHASE_PLAN_002.json"),path.join(topic,"P2G_30_FREEZE_001.json"));
  assert.deepEqual(m.failures,[]);assert.equal(m.freeze.members.length,6);assert.equal(m.finalNodes.length,2);
  assert.deepEqual(m.freeze.members.map(m=>m.element),[1,2,3,4,5,6].map(n=>`CAMEC-E0${n}`));
});

const legacyDir=path.join(root,"core_enhanced/docs/goals");
for(const topicName of ["current_runtime_architecture_remediation","conditional_wave_and_boss_summon_policy"]){
  const dir=path.join(legacyDir,topicName);
  for(const name of fs.readdirSync(dir).filter(n=>/^P2G_40.*RECEIPT.*\.json$/.test(n))){
    test(`legacy immutable Receipt: ${topicName}/${name}`,()=>{
      const receipt=path.join(dir,name),bytes=fs.readFileSync(receipt),r=JSON.parse(bytes);
      const plan=path.resolve(root,r.evaluated_phase_plan.path),freeze=path.resolve(root,r.bound_final_policy_freeze.path);
      const output=validateReceipt(plan,freeze,receipt);assert.equal(output.result,"passed",JSON.stringify(output.failures));
      assert.equal(generateReceipt(plan,freeze,receipt).status,"unchanged");assert.deepEqual(fs.readFileSync(receipt),bytes);
    });
  }
}

test("P2G-50 directly consumes JSON upstream and rejects a changed resolution source",t=>{
  // A temporary deliberately incomplete Goal file exercises only upstream consumption.
  // This is a tooling fixture, not a P2G-50 result or Goal authoring for the Topic.
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),"p2g-consumer-"));t.after(()=>fs.rmSync(dir,{recursive:true,force:true}));
  for(const name of ["P2G_00_INITIAL_001.json","P2G_10_POLICY_001.json","P2G_20_REVIEW_001.json","P2G_20_SOURCE_PLAN_001.json","P2G_30_FREEZE_001.json","P2G_40_PHASE_PLAN_002.json"])fs.copyFileSync(path.join(topic,name),path.join(dir,name));
  const plan=path.join(dir,"P2G_40_PHASE_PLAN_002.json"),freeze=path.join(dir,"P2G_30_FREEZE_001.json"),receipt=path.join(dir,"receipt.json"),goal=path.join(dir,"goal.md");
  generateReceipt(plan,freeze,receipt);fs.writeFileSync(goal,"# Deliberately incomplete consumer fixture\n");
  const run=()=>spawnSync(process.execPath,[tool50,"--plan",plan,"--freeze",freeze,"--receipt",receipt,"--quality-contract","2","--goal-set",goal],{cwd:root,encoding:"utf8"});
  let r=run();assert.ifError(r.error);assert.equal(r.status,1,r.stderr);let out=JSON.parse(r.stdout);
  assert.equal(out.counts.freeze_members,6);assert.equal(out.failures.some(f=>f.code==="UPSTREAM_RECEIPT_VALIDATION"),false);
  const source=path.join(dir,"P2G_10_POLICY_001.json");fs.appendFileSync(source,"\n");
  r=run();assert.equal(r.status,1,r.stderr);out=JSON.parse(r.stdout);assert(out.failures.some(f=>f.code==="UPSTREAM_RECEIPT_VALIDATION"));
});

test("P2G-50 historical complete Goal Set still validates with its immutable Receipt",()=>{
  const dir=path.join(legacyDir,"current_runtime_architecture_remediation");
  const receipt=path.join(dir,"P2G_40_CORE_ONLY_PHASE_PLAN_VALIDATION_RECEIPT_002.json");
  const r=JSON.parse(fs.readFileSync(receipt));
  const out=spawnSync(process.execPath,[tool50,"--plan",path.resolve(root,r.evaluated_phase_plan.path),"--freeze",path.resolve(root,r.bound_final_policy_freeze.path),"--receipt",receipt,"--quality-contract","2","--goal-set",path.join(dir,"P2G_50_CORE_ONLY_GOAL_SET_002.ko.md")],{cwd:root,encoding:"utf8"});
  assert.ifError(out.error);assert.equal(out.status,0,out.stderr+out.stdout);
  assert.equal(JSON.parse(out.stdout).result,"passed");
});
