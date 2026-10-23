#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import {pathToFileURL} from "node:url";
import {readFreeze, readPlan, equal, sha256, displayPath, requireValue} from "./p2g_inputs.mjs";

export const VERSION = "p2g40-receipt-validator/1.2.0";
const GENERATOR_VERSION = "p2g40-receipt-generator/1.2.0";
const sortedUnique = values => [...new Set(values)].sort();
const sameSet = (a,b) => equal(sortedUnique(a),sortedUnique(b));
const countBy = values => values.reduce((m,v) => m.set(v,(m.get(v)??0)+1),new Map());

function usage() {
  return `Usage: node core_enhanced/docs/tools/validate_p2g40_receipt.mjs [--write-receipt]
  --plan <plan.json|plan.md> --freeze <freeze.json|freeze.md> --receipt <receipt.json>
Checks exact references, coverage, graph and validation ownership. It does not judge
policy approval/design correctness or execute game tests. Existing Receipts are immutable.`;
}
function parseArgs(argv) {
  const args={};
  for(let i=0;i<argv.length;i++) {
    const arg=argv[i];
    if(arg==="--help"||arg==="-h") {args.help=true;continue;}
    if(arg==="--write-receipt") {args.writeReceipt=true;continue;}
    requireValue(["--plan","--freeze","--receipt"].includes(arg),"ARGUMENT",arg);
    requireValue(argv[i+1]&&!argv[i+1].startsWith("--"),"ARGUMENT",`missing ${arg}`);
    args[arg.slice(2)]=argv[++i];
  }
  return args;
}

// Parse once, then use the same relations for generation and verification.
export function analyze(planPath, freezePath) {
  const plan=readPlan(planPath), freeze=readFreeze(freezePath), failures=[];
  const check=(ok,code,detail)=>{if(!ok)failures.push({code,detail});};
  const unique=(values,code)=>check(values.length===new Set(values).size,code,JSON.stringify(values));
  const nonempty=(value)=>typeof value==="string"&&value.trim().length>0;
  check(nonempty(plan.ref)&&nonempty(plan.receiptRef)&&nonempty(freeze.ref),"IDENTITY_REQUIRED","Plan/Receipt/Freeze");
  check(plan.freezeRef===freeze.ref,"REF_FREEZE",`${plan.freezeRef} / ${freeze.ref}`);
  check(["p2g40-receipt-validator/1.0.0","p2g40-receipt-validator/1.1.0",VERSION].includes(plan.method),"VALIDATOR_VERSION",plan.method);
  if(plan.format==="json"||freeze.format==="json") check(plan.method===VERSION,"STRUCTURED_INPUT_VERSION",VERSION);
  if(plan.format==="json") {
    check(equal(plan["Bound Final Policy Freeze Ref"],freeze.exactRef),"EXACT_FREEZE_REF","typed Freeze mismatch");
    const basis=plan["Exact Prerequisite Composite Basis"];
    check(basis&&equal(basis["Working Revision Ref"],freeze.payload?.["Working Revision Ref"])&&
      equal(basis["Final Policy Freeze Ref"],freeze.exactRef),"PREREQUISITE_BASIS","Working Revision/Freeze");
    for(const [name,value] of [["obligations",plan.repair],["resolutions",plan.resolution],["supersessions",plan.supersession]])
      check(Array.isArray(value),"REPAIR_BASIS",name);
  } else check(Boolean(plan.repairBasis),"REPAIR_BASIS","missing repair basis");
  const elements=freeze.members.map(m=>m.element);
  check(elements.length>0,"FREEZE_MEMBERS","empty Freeze"); unique(elements,"FREEZE_AMBIGUOUS");
  const wpIds=plan.work_packages.map(w=>w.id);
  check(wpIds.length>0&&wpIds.every(nonempty),"WORK_PACKAGES","empty/missing Work Package ID");unique(wpIds,"WP_DUPLICATE");
  for(const w of plan.work_packages) {
    const selectors=Object.values(w.selectors).flat();
    check(selectors.length>0,"WP_SELECTOR_EMPTY",w.id);
    check(selectors.every(s=>elements.includes(s)),"WP_SELECTOR_UNRESOLVED",w.id);
    check(w.rolePresent!==false,"WP_SELECTOR_ROLE",w.id);
    for(const value of Object.values(w.selectors)) unique(value,"WP_SELECTOR_DUPLICATE");
  }
  const edges=plan.dependencies.map(e=>[e.from,e.to]);
  check(edges.every(([a,b])=>wpIds.includes(a)&&wpIds.includes(b)),"EDGE_ENDPOINT","unknown Work Package");
  const safeEdges=edges.filter(([a,b])=>wpIds.includes(a)&&wpIds.includes(b));
  const components=tarjan(wpIds,safeEdges);
  const cyclic=components.filter(c=>c.length>1||safeEdges.some(([a,b])=>a===c[0]&&b===c[0]));
  const documented=plan.cyclic_sccs.map(s=>s.members);
  check(sameSet(canonicalComponents(cyclic),canonicalComponents(documented)),"SCC_COVERAGE","computed/documented cycles");
  const groupIds=plan.atomic_groups.map(g=>g.id);unique(groupIds,"GROUP_DUPLICATE");
  check(groupIds.every(nonempty),"GROUP_ID","missing Atomic Group ID");
  const groups=new Map(plan.atomic_groups.map(g=>[g.id,g.members]));
  const standalone=plan.standalone;
  const partition=[...groups.values()].flat().concat(standalone);
  check(sameSet(partition,wpIds),"PARTITION_COVERAGE","unknown/missing group member or standalone node");
  unique(partition,"PARTITION_OVERLAP");
  for(const g of plan.atomic_groups)check(g.members.length>0,"EMPTY_GROUP",g.id);
  for(const scc of plan.cyclic_sccs) {
    check(scc.disposition==="coimplement_atomic","SCC_DISPOSITION","remaining cycles must be coimplemented after seam reconstruction");
    check([...groups.values()].some(ms=>scc.members.every(m=>ms.includes(m))),"SCC_ATOMIC_BOUNDARY",scc.members.join(","));
  }
  const wpToFinal=new Map(standalone.map(w=>[w,w]));
  for(const [g,ms] of groups)for(const w of ms)wpToFinal.set(w,g);
  const finalNodes=[...groups.keys(),...standalone];unique(finalNodes,"NODE_ID_DUPLICATE");
  const condensedEdges=[...new Map(safeEdges.map(([a,b])=>[wpToFinal.get(a),wpToFinal.get(b)])
    .filter(([a,b])=>a&&b&&a!==b).map(e=>[JSON.stringify(e),e])).values()];
  check(isAcyclic(finalNodes,condensedEdges),"CONDENSATION_CYCLE","cycle after atomic grouping");
  const phaseIds=plan.phases.map(p=>p.id);unique(phaseIds,"PHASE_DUPLICATE");
  check(phaseIds.length>0&&phaseIds.every(nonempty),"PHASES_REQUIRED","empty/missing Phase ID");
  const primaryAssignments=plan.phases.flatMap(p=>p.primary.map(element_id=>({element_id,phase_id:p.id})));
  const primary=primaryAssignments.map(a=>a.element_id);unique(primary,"PRIMARY_DUPLICATE");
  check(sameSet(primary,elements),"PRIMARY_COVERAGE","primary selectors != frozen member IDs");
  const assignment=plan.phases.flatMap(p=>p.nodes.map(n=>[n,p.id]));
  unique(assignment.map(([n])=>n),"PHASE_NODE_DUPLICATE");
  check(sameSet(assignment.map(([n])=>n),finalNodes),"PHASE_NODE_COVERAGE","final nodes != assigned nodes");
  const nodeAssignments=Object.fromEntries(assignment);
  for(const [a,b] of condensedEdges)check(phaseIds.indexOf(nodeAssignments[a])>=0&&phaseIds.indexOf(nodeAssignments[a])<=phaseIds.indexOf(nodeAssignments[b]),"PHASE_ORDER",`${a} -> ${b}`);
  check(sameSet(plan.audits.map(a=>a.phase),phaseIds)&&plan.audits.length===phaseIds.length,"AUDIT_COVERAGE","one audit per Phase");
  unique(plan.integration.map(i=>i.id),"INTEGRATION_DUPLICATE");
  check(plan.integration.every(i=>nonempty(i.id)),"INTEGRATION_ID","missing integration ID");
  check(plan.integration.length>0,"INTEGRATION_REQUIRED","missing integration ownership");
  for(const i of plan.integration) {
    check(["assigned_execution_node","existing_read_only","audit_local_read_only"].includes(i.disposition),"INTEGRATION_SEAM",i.id);
    if(i.disposition==="assigned_execution_node") {
      check(i.nodes.length>0&&i.nodes.every(n=>finalNodes.includes(n)),"INTEGRATION_OWNER_NODE",i.id);
      check(i.phases.length>0&&i.phases.every(p=>phaseIds.includes(p)),"INTEGRATION_OWNER_PHASE",i.id);
      check(i.nodes.every(n=>i.phases.includes(nodeAssignments[n])),"INTEGRATION_OWNER_ASSIGNMENT",i.id);
    } else check(nonempty(i.locator)&&nonempty(i.evidence),"READ_ONLY_SEAM_EVIDENCE",i.id);
  }
  const goalFact=phaseIds.length===1?"single_phase":"multi_phase";
  check(plan.goalFact===goalFact,"GOAL_STRUCTURE",goalFact);
  check(plan.parallelization==="none approved"||Boolean(plan.parallelEvidence?.length),"PARALLELIZATION","missing disposition/evidence");
  return {plan,freeze,failures,wpIds,edges,cyclic,groups,standalone,finalNodes,condensedEdges,phaseIds,primaryAssignments,nodeAssignments,goalFact};
}

function buildReceipt(planPath,freezePath,model) {
  const {plan,freeze,wpIds,edges,cyclic,groups,standalone,finalNodes,condensedEdges,phaseIds,primaryAssignments,nodeAssignments,goalFact}=model;
  const planRef=plan.ref, receiptRef=plan.receiptRef, freezeRef=plan.freezeRef, freezeDocumentRef=freeze.ref;
  const planBuffer=plan.bytes, freezeBuffer=freeze.bytes;
  const revisionRefs=freeze.members.map(m=>m.revision), revisionByElement=new Map(freeze.members.map(m=>[m.element,m.revision]));
  const sccRows=plan.cyclic_sccs, auditRows=plan.audits, integrationRows=plan.integration;
  const assignedIntegrationCount=integrationRows.filter(i=>i.disposition==="assigned_execution_node").length;
  const existingIntegrationCount=integrationRows.filter(i=>i.disposition==="existing_read_only").length;
  const auditLocalIntegrationCount=integrationRows.filter(i=>i.disposition==="audit_local_read_only").length;
  const parallelization=plan.parallelization, codeReviewBindings=freeze.bindings;
  const obligationCount=plan.repair.length,resolutionCount=plan.resolution.length,supersessionCount=plan.supersession.length;

  return {
    schema_version: "p2g40.phase_plan_validation_receipt/v1",
    phase_plan_validation_receipt_ref: receiptRef,
    evaluated_phase_plan: {
      ref: planRef,
      path: displayPath(planPath),
      sha256: sha256(planBuffer)
    },
    bound_final_policy_freeze: {
      ref: freezeRef,
      path: displayPath(freezePath),
      sha256: sha256(freezeBuffer)
    },
    validator: {
      method_version: plan.method,
      invocation: "node core_enhanced/docs/tools/validate_p2g40_receipt.mjs --plan <plan> --freeze <freeze> --receipt <receipt>",
      methods: [
        "sha256sum exact-input binding",
        "stable selector set comparison",
        "Tarjan SCC and atomic-group condensation audit",
        "final-execution-node cardinality comparison",
        "phase/audit/integration row cardinality comparison"
      ]
    },
    freeze_prerequisite_repair_basis_exact_match: {
      result: "passed",
      working_revision_ref: freeze.working,
      review_batch_ref: freeze.review,
      reviewed_source_basis_ref: freeze.source,
      code_review_binding_count: codeReviewBindings.length,
      bound_final_policy_freeze_ref: freezeDocumentRef,
      applicable_p2g40_obligation_key_count: obligationCount,
      resolution_record_ref_count: resolutionCount,
      obligation_supersession_record_ref_count: supersessionCount
    },
    selector_resolution: {
      result: "passed",
      resolved_unique_count: revisionRefs.length,
      missing_count: 0,
      ambiguous_count: 0,
      extra_count: 0,
      exact_resolution: [...revisionByElement].sort(([left], [right]) => left.localeCompare(right))
        .map(([elementId, revisionRef]) => ({element_id: elementId, policy_element_revision_ref: revisionRef}))
    },
    primary_phase_coverage: {
      result: "passed",
      covered_count: revisionRefs.length,
      missing_count: 0,
      duplicate_count: 0,
      extra_count: 0,
      assignments: primaryAssignments
    },
    work_package_atomic_group_partition: {
      result: "passed",
      work_package_count: wpIds.length,
      atomic_group_count: groups.size,
      atomic_group_member_count: [...groups.values()].flat().length,
      standalone_work_package_count: standalone.length,
      missing_count: 0,
      duplicate_or_overlap_count: 0,
      atomic_groups: Object.fromEntries(groups),
      standalone_work_packages: standalone
    },
    dependency_graph: {
      result: "passed",
      work_package_node_count: wpIds.length,
      edge_count: edges.length,
      cyclic_scc_count: cyclic.length,
      cyclic_scc_disposition_count: sccRows.length,
      cyclic_sccs: [...cyclic].sort((left, right) => left.join(",").localeCompare(right.join(","))),
      final_execution_node_count: finalNodes.length,
      condensation_edge_count: condensedEdges.length,
      condensation_acyclic: isAcyclic(finalNodes, condensedEdges)
    },
    final_execution_node_to_phase_cardinality: {
      result: "passed",
      final_execution_node_count: finalNodes.length,
      assigned_once_count: Object.keys(nodeAssignments).length,
      missing_count: 0,
      duplicate_count: 0,
      ordered_phase_count: phaseIds.length,
      assignment: nodeAssignments
    },
    intermediate_state_audit_row_completeness: {
      result: "passed",
      ordered_phase_count: phaseIds.length,
      audit_row_count: auditRows.length,
      missing_count: 0,
      duplicate_count: 0,
      extra_count: 0
    },
    parallelization_evidence_check: {
      result: parallelization === "none approved" ? "not_claimed" : "passed",
      plan_disposition: parallelization ?? "evidence provided",
      approved_pair_count: 0
    },
    integration_validation_ownership_coverage: {
      result: "passed",
      required_acceptance_count: integrationRows.length,
      mapped_count: integrationRows.length,
      assigned_execution_node_count: assignedIntegrationCount,
      existing_read_only_count: existingIntegrationCount,
      audit_local_read_only_count: auditLocalIntegrationCount,
      missing_owner_count: 0,
      invalid_seam_disposition_count: 0
    },
    goal_structure_fact_check: {
      result: "passed",
      ordered_phase_count: phaseIds.length,
      expected: goalFact,
      actual: plan.goalFact
    },
    failure_summary: [],
    overall_result: "passed"
  };
}

function tarjan(nodes, edges) {
  const adjacency = new Map(nodes.map((node) => [node, []]));
  for (const [from, to] of edges) adjacency.get(from)?.push(to);
  let nextIndex = 0;
  const stack = [];
  const onStack = new Set();
  const indices = new Map();
  const low = new Map();
  const components = [];

  function visit(node) {
    indices.set(node, nextIndex);
    low.set(node, nextIndex);
    nextIndex += 1;
    stack.push(node);
    onStack.add(node);
    for (const adjacent of adjacency.get(node) ?? []) {
      if (!indices.has(adjacent)) {
        visit(adjacent);
        low.set(node, Math.min(low.get(node), low.get(adjacent)));
      } else if (onStack.has(adjacent)) {
        low.set(node, Math.min(low.get(node), indices.get(adjacent)));
      }
    }
    if (low.get(node) !== indices.get(node)) return;
    const component = [];
    let popped;
    do {
      popped = stack.pop();
      onStack.delete(popped);
      component.push(popped);
    } while (popped !== node);
    components.push(component.sort());
  }

  for (const node of nodes) if (!indices.has(node)) visit(node);
  return components;
}

function isAcyclic(nodes, edges) {
  const adjacency = new Map(nodes.map((node) => [node, []]));
  const indegree = new Map(nodes.map((node) => [node, 0]));
  for (const [from, to] of edges) {
    adjacency.get(from).push(to);
    indegree.set(to, indegree.get(to) + 1);
  }
  const queue = [...indegree].filter(([, degree]) => degree === 0).map(([node]) => node).sort();
  let visited = 0;
  while (queue.length > 0) {
    const node = queue.shift();
    visited += 1;
    for (const adjacent of adjacency.get(node)) {
      indegree.set(adjacent, indegree.get(adjacent) - 1);
      if (indegree.get(adjacent) === 0) {
        queue.push(adjacent);
        queue.sort();
      }
    }
  }
  return visited === nodes.length;
}

function canonicalComponents(components) {
  return components.map((component) => [...component].sort().join(",")).sort();
}



function expectedReceipt(planPath, freezePath, model) {
  const receipt=buildReceipt(planPath,freezePath,model);
  if(model.plan.method===VERSION) {
    receipt.structural_input_evidence={
      scope:"references, coverage, graph, validation ownership; semantic review remains separate",
      exact_freeze_ref:model.freeze.exactRef??null,
      exact_plan_ref:model.plan.exactRef??null,
      resolved_artifact_sources:model.freeze.resolvedSources
    };
  }
  return receipt;
}

export function validateReceipt(planPath,freezePath,receiptInput) {
  const model=analyze(planPath,freezePath);
  const receipt=typeof receiptInput==="string"?JSON.parse(fs.readFileSync(receiptInput,"utf8")):receiptInput;
  const failures=[...model.failures];
  if(failures.length===0) {
    const expected=expectedReceipt(planPath,freezePath,model);
    // Compare every computed relation/count, not just a self-reported passed flag.
    // Historical method descriptions differ; their declared version still must match.
    for(const [key,value] of Object.entries(expected)) {
      if(key==="validator") {
        if(receipt.validator?.method_version!==model.plan.method)failures.push({code:"VALIDATOR_VERSION",detail:"Plan/Receipt version mismatch"});
      } else if(!equal(value,receipt[key]))failures.push({code:"RECEIPT_MISMATCH",detail:key});
    }
  }
  return result(model,failures);
}
function result(m,failures=m.failures) {
  return {validator:VERSION,result:failures.length?"failed":"passed",counts:{freeze_members:m.freeze.members.length,work_packages:m.wpIds.length,dependency_edges:m.edges.length,cyclic_sccs:m.cyclic.length,atomic_groups:m.groups.size,final_execution_nodes:m.finalNodes.length,phases:m.phaseIds.length,audit_rows:m.plan.audits.length,integration_rows:m.plan.integration.length},failures};
}
export function generateReceipt(planPath,freezePath,receiptPath) {
  const model=analyze(planPath,freezePath), validation=result(model);
  const output={generator:GENERATOR_VERSION,status:"not_written",receipt:displayPath(receiptPath),validation};
  if(validation.result!=="passed")return output;
  const receipt=expectedReceipt(planPath,freezePath,model);
  if(fs.existsSync(receiptPath)) {
    // Validate historical immutable receipts without rewriting descriptive metadata.
    const existing=JSON.parse(fs.readFileSync(receiptPath,"utf8"));
    const verified=validateReceipt(planPath,freezePath,existing);
    requireValue(verified.result==="passed","IMMUTABLE_RECEIPT","different target exists; issue successor Ref/path");
    return {...output,status:"unchanged"};
  }
  fs.writeFileSync(receiptPath,JSON.stringify(receipt,null,2)+"\n",{flag:"wx"});
  return {...output,status:"created"};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    const args=parseArgs(process.argv.slice(2));
    if(args.help){console.log(usage());process.exit(0);}
    for(const key of ["plan","freeze","receipt"])requireValue(args[key],"ARGUMENT",`--${key} required`);
    const output=args.writeReceipt?generateReceipt(args.plan,args.freeze,args.receipt):validateReceipt(args.plan,args.freeze,args.receipt);
    console.log(JSON.stringify(output,null,2));
    process.exit((args.writeReceipt?output.validation:output).result==="passed"?0:1);
  } catch(error) {
    console.error(`${VERSION}: ${error.message}`);process.exit(2);
  }
}
