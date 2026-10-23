#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import {readFreeze, readPlan} from "./p2g_inputs.mjs";
import {validateReceipt} from "./validate_p2g40_receipt.mjs";

const VERSION = "p2g50-goal-set-validator/1.3.0";
const QUALITY_DIMENSIONS = [
  "success",
  "invalid_input",
  "failure_rollback",
  "retry_idempotency",
  "ordering_concurrency",
  "boundary_empty_max",
  "disconnect_restart_persistence",
  "live_data",
  "producer_consumer",
  "legacy_fallback_absence"
];

function usage() {
  return `Usage:
  node core_enhanced/docs/tools/validate_p2g50_goal_set.mjs \\
    --freeze <final-freeze.md> \\
    --plan <phase-plan.md> \\
    --receipt <phase-plan-validation-receipt.json> \\
    --quality-contract <1|2> \\
    --goal-set <goal-set.md>

Validates P2G-50 structure and exact upstream mappings. It does not judge design
quality, run game tests, or modify any file. Use quality contract 1 explicitly
for historical Goal Sets; current P2G-50 authoring requires quality contract 2.`;
}

function parseArgs(argv) {
  const result = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      result.help = true;
      continue;
    }
    if (!["--freeze", "--plan", "--receipt", "--quality-contract", "--goal-set"].includes(arg)) {
      throw new Error(`unknown argument: ${arg}`);
    }
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) throw new Error(`missing value for ${arg}`);
    const key = arg.slice(2)
      .replace("goal-set", "goalSet")
      .replace("quality-contract", "qualityContract");
    result[key] = value;
    index += 1;
  }
  return result;
}

function sha256(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function stripCode(value) {
  const text = String(value ?? "").trim();
  return text.startsWith("`") && text.endsWith("`") ? text.slice(1, -1) : text;
}

function field(markdown, label) {
  const match = markdown.match(new RegExp(`^- ${escapeRegex(label)}:\\s*(.+)$`, "m"));
  return match ? stripCode(match[1]) : null;
}

function fieldBlock(markdown, label) {
  const lines = markdown.split(/\r?\n/);
  const prefix = `- ${label}:`;
  const start = lines.findIndex((line) => line.startsWith(prefix));
  if (start < 0) return null;
  const values = [lines[start].slice(prefix.length).trim()];
  for (let index = start + 1; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.trim() === "" || !/^\s+/.test(line)) break;
    values.push(line.trim());
  }
  return values.join(" ");
}

function section(markdown, title) {
  const lines = markdown.split(/\r?\n/);
  const start = lines.findIndex((line) => line.match(/^#+\s+(.*)$/)?.[1].trim() === title);
  if (start < 0) return "";
  const level = lines[start].match(/^#+/)?.[0].length ?? 2;
  let end = lines.length;
  for (let index = start + 1; index < lines.length; index += 1) {
    const heading = lines[index].match(/^(#+)\s/);
    if (heading && heading[1].length <= level) {
      end = index;
      break;
    }
  }
  return lines.slice(start + 1, end).join("\n");
}

function memberSections(markdown) {
  const lines = markdown.split(/\r?\n/);
  const starts = [];
  for (let index = 0; index < lines.length; index += 1) {
    const heading = lines[index].match(/^(#+)\s+(.+)$/);
    if (!heading) continue;
    if (/^(Master Goal Member|Phase Goal Member|Single Goal Member|Single\/Phase Goal Member)/.test(heading[2])) {
      starts.push({index, level: heading[1].length, title: heading[2]});
    }
  }
  return starts.map((entry) => {
    let end = lines.length;
    for (let index = entry.index + 1; index < lines.length; index += 1) {
      const heading = lines[index].match(/^(#+)\s/);
      if (heading && heading[1].length <= entry.level) {
        end = index;
        break;
      }
    }
    return {title: entry.title, body: lines.slice(entry.index + 1, end).join("\n")};
  });
}

function tableRows(markdownSection, firstCellPattern) {
  const rows = [];
  for (const line of markdownSection.split(/\r?\n/)) {
    if (!line.startsWith("|")) continue;
    const cells = line.split("|").slice(1, -1).map((cell) => cell.trim());
    if (cells.length > 0 && firstCellPattern.test(cells[0].replaceAll("`", ""))) rows.push(cells);
  }
  return rows;
}

function ids(value, prefix) {
  return [...String(value ?? "").matchAll(new RegExp(`${prefix}-[A-Z0-9-]+`, "g"))]
    .map((match) => match[0]);
}

function sortedUnique(values) {
  return [...new Set(values)].sort();
}

function sameSet(left, right) {
  const a = sortedUnique(left);
  const b = sortedUnique(right);
  return a.length === b.length && a.every((value, index) => value === b[index]);
}

function setFromField(markdown, label, prefix) {
  return ids(fieldBlock(markdown, label) ?? "", prefix);
}

function executionNodeIds(value) {
  const withoutMemberNotes = String(value ?? "").replace(/\([^)]*\)/g, "");
  return [...ids(withoutMemberNotes, "AG"), ...ids(withoutMemberNotes, "WP")];
}

function hasAnyField(markdown, labels) {
  return labels.some((label) => fieldBlock(markdown, label) !== null);
}

function isSubstantive(value) {
  const text = stripCode(value).trim();
  return text.length > 0 && !/^(?:n\/a|none|tbd|todo|discovery pending|-+)$/i.test(text);
}

function hasAnySubstantiveField(markdown, labels) {
  return labels.some((label) => isSubstantive(fieldBlock(markdown, label)));
}

function dataTableRows(markdownSection) {
  const rows = [];
  for (const line of markdownSection.split(/\r?\n/)) {
    if (!line.startsWith("|")) continue;
    const cells = line.split("|").slice(1, -1).map((cell) => cell.trim());
    if (cells.length === 0) continue;
    const first = stripCode(cells[0]);
    if (!first || /^:?-+:?$/.test(first) || first.toLowerCase() === "dimension") continue;
    rows.push(cells);
  }
  return rows;
}

function validate({freezePath, planPath, receiptPath, goalSetPath, qualityContractVersion}) {
  const freezeBuffer = fs.readFileSync(freezePath);
  const planBuffer = fs.readFileSync(planPath);
  const goalSetBuffer = fs.readFileSync(goalSetPath);
  const freezeModel = readFreeze(freezePath);
  const planModel = readPlan(planPath);
  const goalSet = goalSetBuffer.toString("utf8");
  const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
  const failures = [];
  const check = (condition, code, detail) => {
    if (!condition) failures.push({code, detail});
  };

  const freezeRef = freezeModel.ref;
  const planRef = planModel.ref;
  const planFreezeRef = planModel.freezeRef;
  const planReceiptRef = planModel.receiptRef;
  const goalPayload = section(goalSet, "Goal Set");
  const goalSetRef = field(goalPayload, "Goal Set Ref");
  check(Boolean(goalSetRef), "GOAL_SET_REF", "Goal Set Ref missing");
  if (qualityContractVersion === 2) {
    check(field(goalPayload, "Goal Quality Contract Version") === "2",
      "GOAL_QUALITY_CONTRACT", "Goal Quality Contract Version must be 2");
  }
  check(planFreezeRef === freezeRef, "PLAN_FREEZE_REF", `${planFreezeRef} != ${freezeRef}`);
  check(field(goalPayload, "Phase Plan Ref") === planRef, "GOAL_PLAN_REF", "Goal Set/Phase Plan Ref mismatch");
  check(field(goalPayload, "Final Policy Freeze Ref") === freezeRef, "GOAL_FREEZE_REF", "Goal Set/Freeze Ref mismatch");
  check(receipt.phase_plan_validation_receipt_ref === planReceiptRef, "RECEIPT_REF", "Plan/Receipt Ref mismatch");
  check(receipt.evaluated_phase_plan?.ref === planRef, "RECEIPT_PLAN_REF", "Receipt/Plan Ref mismatch");
  check(receipt.bound_final_policy_freeze?.ref === freezeRef, "RECEIPT_FREEZE_REF", "Receipt/Freeze Ref mismatch");
  check(receipt.evaluated_phase_plan?.sha256 === sha256(planBuffer), "RECEIPT_PLAN_HASH", "Receipt Plan hash mismatch");
  check(receipt.bound_final_policy_freeze?.sha256 === sha256(freezeBuffer), "RECEIPT_FREEZE_HASH", "Receipt Freeze hash mismatch");
  check(receipt.overall_result === "passed", "RECEIPT_RESULT", `overall_result=${receipt.overall_result}`);
  check(Array.isArray(receipt.failure_summary) && receipt.failure_summary.length === 0,
    "RECEIPT_FAILURES", "upstream Receipt failure summary is not empty");

  const upstream = validateReceipt(planPath, freezePath, receipt);
  check(upstream.result === "passed", "UPSTREAM_RECEIPT_VALIDATION", JSON.stringify(upstream.failures));
  const freezePers = freezeModel.members.map(m => m.revision);
  const freezeDrs = freezeModel.decisions;
  const perByElement = new Map(freezeModel.members.map(m => [m.element, m.revision]));
  check(freezePers.length > 0, "FREEZE_MEMBERS", "no ordered Freeze members parsed");
  check(perByElement.size === freezePers.length, "FREEZE_PER_AMBIGUOUS", "duplicate Stable Element ID in PER set");
  check(freezeDrs.length > 0, "FREEZE_DECISIONS", "no Freeze Decision revisions parsed");
  const phases = planModel.phases.map(p => p.id);
  const primaryByElement = new Map();
  const nodesByPhase = new Map();
  for (const phase of planModel.phases) {
    for (const element of phase.primary) {
      if (!primaryByElement.has(element)) primaryByElement.set(element, []);
      primaryByElement.get(element).push(phase.id);
    }
    nodesByPhase.set(phase.id, phase.nodes);
  }
  const planNodes = [...nodesByPhase.values()].flat();
  const expectedGoalFact = phases.length === 1 ? "single_phase" : "multi_phase";
  check(phases.length > 0 && sortedUnique(phases).length === phases.length, "PLAN_PHASES", "missing/duplicate Phase ID");
  check(planModel.goalFact === expectedGoalFact, "PLAN_GOAL_FACT", expectedGoalFact);
  check(field(goalPayload, "Goal Structure Fact") === expectedGoalFact, "GOAL_STRUCTURE", expectedGoalFact);
  check(sameSet([...primaryByElement.keys()], [...perByElement.keys()]) &&
    [...primaryByElement.values()].every((owners) => owners.length === 1),
  "PLAN_PRIMARY_COVERAGE", "primary Phase coverage is not exact");
  check(sortedUnique(planNodes).length === planNodes.length, "PLAN_NODE_DUPLICATE", "Final Execution Node duplicated across Phases");

  const mappingRows = tableRows(section(goalSet, "Phase-to-Goal Member Mapping"), /^PHASE-/);
  const phaseToGoal = new Map();
  const goalToNodes = new Map();
  for (const row of mappingRows) {
    const phase = stripCode(row[0]);
    const goal = stripCode(row[1]);
    if (!phaseToGoal.has(phase)) phaseToGoal.set(phase, []);
    phaseToGoal.get(phase).push(goal);
    goalToNodes.set(goal, [...ids(row[2], "AG"), ...ids(row[2], "WP")]);
  }
  const mappedGoals = [...phaseToGoal.values()].flat();
  const mappedNodes = [...goalToNodes.values()].flat();
  check(mappingRows.length === phases.length && sameSet([...phaseToGoal.keys()], phases),
    "PHASE_GOAL_COVERAGE", "Phase-to-Goal mapping missing/extra Phase");
  check([...phaseToGoal.values()].every((goals) => goals.length === 1) &&
    sortedUnique(mappedGoals).length === mappedGoals.length,
  "PHASE_GOAL_CARDINALITY", "each Phase must map to one unique executable Goal");
  check(sameSet(mappedNodes, planNodes) && mappedNodes.length === planNodes.length,
    "GOAL_NODE_COVERAGE", "Final Execution Node mapping mismatch");
  for (const phase of phases) {
    const goal = phaseToGoal.get(phase)?.[0];
    check(sameSet(goalToNodes.get(goal) ?? [], nodesByPhase.get(phase) ?? []),
      "GOAL_NODE_PHASE_DETAIL", `${phase} -> ${goal}`);
  }

  const members = memberSections(goalSet).map((entry) => {
    const goalId = field(entry.body, "Goal ID") ?? entry.title.match(/`(GOAL-[A-Z0-9-]+)`/)?.[1] ?? null;
    const combined = field(entry.body, "Goal Kind / Phase ID");
    const kind = field(entry.body, "Goal Kind") ?? combined?.split("/")[0]?.trim() ??
      (entry.title.startsWith("Master") ? "master" : entry.title.startsWith("Single") ? "single" : "phase");
    const phaseId = field(entry.body, "Phase ID") ?? ids(combined ?? "", "PHASE")[0] ?? null;
    return {...entry, goalId, kind: stripCode(kind), phaseId};
  });
  const memberIds = members.map((member) => member.goalId).filter(Boolean);
  const masterMembers = members.filter((member) => member.kind === "master");
  const executableMembers = members.filter((member) => member.kind === "single" || member.kind === "phase");
  check(memberIds.length === members.length && sortedUnique(memberIds).length === memberIds.length,
    "GOAL_MEMBER_IDS", "Goal member ID missing/duplicate");
  check(sameSet(setFromField(goalPayload, "Complete Goal Member Set", "GOAL"), memberIds),
    "GOAL_MEMBER_SET", "Complete Goal Member Set mismatch");
  check(sameSet(executableMembers.map((member) => member.goalId), mappedGoals),
    "EXECUTABLE_GOAL_SET", "mapped Goal/member mismatch");
  if (expectedGoalFact === "multi_phase") {
    check(masterMembers.length === 1 && executableMembers.length === phases.length &&
      executableMembers.every((member) => member.kind === "phase"),
    "MULTI_PHASE_SHAPE", "multi_phase requires one Master and one Phase Goal per Phase");
  } else {
    check(masterMembers.length === 0 && executableMembers.length === 1 && executableMembers[0]?.kind === "single",
      "SINGLE_PHASE_SHAPE", "single_phase requires one Single Goal and no Master");
  }

  if (masterMembers.length === 1) {
    const master = masterMembers[0];
    check(sameSet(setFromField(master.body, "Complete Phase Goal ID Set", "GOAL"),
      executableMembers.map((member) => member.goalId)),
    "MASTER_PHASE_GOALS", "Master complete Phase Goal set mismatch");
    check(/^None$/i.test(field(master.body, "Assigned Phase / Final Execution Node / Implementation Work Ownership") ?? ""),
      "MASTER_NO_IMPLEMENTATION", "Master must own no Phase/node/implementation work");
    check(Boolean(section(master.body, "Derived Cross-Phase Invariants") ||
      fieldBlock(master.body, "Derived Cross-Phase Invariants and Source Refs")),
    "MASTER_INVARIANTS", "Master cross-Phase invariant mapping missing");
    check(Boolean(fieldBlock(master.body, "Overall Completion Conditions")),
      "MASTER_COMPLETION", "Master overall completion conditions missing");
  }

  const traceRows = dataTableRows(section(goalSet, "Policy–Phase–Goal Traceability")).slice(1);
  const traceByElement = new Map();
  for (const row of traceRows) {
    const element = stripCode(row[0]);
    if (!traceByElement.has(element)) traceByElement.set(element, []);
    traceByElement.get(element).push({
      per: stripCode(row[1]),
      drs: sortedUnique(ids(row[2], "DR")),
      phase: stripCode(row[3]),
      goal: stripCode(row[4])
    });
  }
  check(sameSet([...traceByElement.keys()], [...perByElement.keys()]) &&
    [...traceByElement.values()].every((rows) => rows.length === 1),
  "TRACE_COVERAGE", "traceability row coverage/cardinality mismatch");
  for (const [element, rows] of traceByElement) {
    const row = rows[0];
    const expectedPhase = primaryByElement.get(element)?.[0];
    const expectedGoal = phaseToGoal.get(expectedPhase)?.[0];
    check(row.per === perByElement.get(element), "TRACE_PER", element);
    check(row.drs.length > 0 && row.drs.every((ref) => freezeDrs.includes(ref)), "TRACE_DR", element);
    check(row.phase === expectedPhase, "TRACE_PHASE", element);
    check(row.goal === expectedGoal, "TRACE_GOAL", element);
  }
  check(sameSet([...traceByElement.values()].flatMap((rows) => rows[0]?.drs ?? []), freezeDrs),
    "TRACE_DR_COVERAGE", "traceability Decision revision union does not equal Freeze Decision set");

  const requiredMemberFields = [
    ["Objective / Contract-Visible Result"],
    ["In-Scope", "In-Scope / Out-of-Scope"],
    ["Out-of-Scope", "In-Scope / Out-of-Scope"],
    ["Reviewed Current Behavior / Delta", "Reviewed Current Behavior / Implementation Delta"],
    ["Boundary", "Code/Data/Schema/Store/Transport/Consumer Boundary"],
    ["Dependencies / Prerequisites", "Goal-Local Dependencies / Execution Prerequisites"],
    ["Inherited Baseline / Impact Map", "Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan"],
    ["Assigned Durable Integration Seam", "Assigned Durable Integration Validation Seam Coverage"],
    ["Migration/Compatibility/Rollout/Rollback", "Applicable Migration/Compatibility/Rollout/Rollback"],
    ["Known Unknown / Blocker / Stop", "Goal-Local Known Unknown / Blocker / Stop Condition"],
    ["Risk / Reviewer", "Applicable Risk / Reviewer Requirement and Evidence"]
  ];
  const qualityMemberFields = [
    ["User-Visible Problem"],
    ["User-Visible Completion Condition Set"],
    ["Goal-Local Source of Truth / Owning Layer"],
    ["Goal-Local Invariant Set / Source Refs"],
    ["Consumer Verification Boundary"],
    ["Structural Completion / Legacy and Fallback Disposition"],
    ["Canonical Documentation Obligation"]
  ];
  let validationDimensionRowCount = 0;
  for (const member of executableMembers) {
    const mappedPhase = [...phaseToGoal].find(([, goals]) => goals.includes(member.goalId))?.[0];
    check(member.phaseId === mappedPhase, "MEMBER_PHASE", member.goalId);
    check(sameSet(executionNodeIds(fieldBlock(member.body, "Assigned Final Execution Node Set")),
    goalToNodes.get(member.goalId) ?? []), "MEMBER_NODES", member.goalId);
    const expectedElements = [...traceByElement].filter(([, rows]) => rows[0]?.goal === member.goalId)
      .map(([element]) => element);
    const expectedPers = expectedElements.map((element) => perByElement.get(element));
    const expectedDrs = sortedUnique(expectedElements.flatMap((element) =>
      traceByElement.get(element)?.[0]?.drs ?? []));
    check(sameSet(setFromField(member.body, "Covered Policy Element Revision Ref Set", "PER"), expectedPers),
      "MEMBER_PER_COVERAGE", member.goalId);
    check(sameSet(setFromField(member.body, "Covered Decision Revision Ref Set", "DR"), expectedDrs),
      "MEMBER_DR_COVERAGE", member.goalId);
    check(Boolean(fieldBlock(member.body, "Derived Observable Policy Acceptance Coverage")),
      "MEMBER_ACCEPTANCE", member.goalId);
    for (const labels of requiredMemberFields) {
      check(hasAnyField(member.body, labels), "MEMBER_REQUIRED_FIELD", `${member.goalId}: ${labels.join(" | ")}`);
    }
    const completionField = fieldBlock(member.body, "Validation Plan / Completion Matrix");
    const completionTable = /Completion Matrix/.test(member.body) &&
      member.body.split(/\r?\n/).some((line) => line.startsWith("|"));
    check(Boolean(completionField) || completionTable, "MEMBER_COMPLETION_MATRIX", member.goalId);
    if (qualityContractVersion === 2) {
      for (const labels of qualityMemberFields) {
        check(hasAnySubstantiveField(member.body, labels), "MEMBER_QUALITY_FIELD",
          `${member.goalId}: ${labels.join(" | ")}`);
      }

      const matrix = section(member.body, "Validation Dimension Matrix");
      const rows = dataTableRows(matrix);
      validationDimensionRowCount += rows.length;
      const rowsByDimension = new Map();
      for (const row of rows) {
        const dimension = stripCode(row[0]);
        if (!rowsByDimension.has(dimension)) rowsByDimension.set(dimension, []);
        rowsByDimension.get(dimension).push(row);
      }
      check(rows.length === QUALITY_DIMENSIONS.length &&
        sameSet([...rowsByDimension.keys()], QUALITY_DIMENSIONS) &&
        [...rowsByDimension.values()].every((entries) => entries.length === 1),
      "MEMBER_VALIDATION_DIMENSION_COVERAGE", member.goalId);

      for (const dimension of QUALITY_DIMENSIONS) {
        const row = rowsByDimension.get(dimension)?.[0];
        if (!row) continue;
        const disposition = stripCode(row[1]);
        check(disposition === "required" || disposition === "N/A",
          "MEMBER_VALIDATION_DIMENSION_DISPOSITION", `${member.goalId}: ${dimension}`);
        if (disposition === "required") {
          check(isSubstantive(row[2]) && isSubstantive(row[3]),
            "MEMBER_VALIDATION_DIMENSION_REQUIRED_EVIDENCE", `${member.goalId}: ${dimension}`);
        } else if (disposition === "N/A") {
          check(isSubstantive(row[3]), "MEMBER_VALIDATION_DIMENSION_NA_REASON",
            `${member.goalId}: ${dimension}`);
        }
      }
    }
  }

  const planIntegrationRows = planModel.integration;
  const goalIntegrationSection = section(goalSet, "Integration Acceptance Coordination") ||
    section(goalSet, "Integration Acceptance-to-Goal Mapping");
  const goalIntegrationRows = tableRows(goalIntegrationSection, /^IVA-/);
  const planIntegrationById = new Map(planIntegrationRows.map((row) => [row.id, row]));
  const goalIntegrationById = new Map(goalIntegrationRows.map((row) => [ids(row[0], "IVA")[0], row]));
  check(planIntegrationById.size === planIntegrationRows.length &&
    goalIntegrationById.size === goalIntegrationRows.length &&
    sameSet([...goalIntegrationById.keys()], [...planIntegrationById.keys()]),
  "INTEGRATION_COVERAGE", "Goal integration acceptance set mismatch");
  for (const [acceptanceId, planRow] of planIntegrationById) {
    const goalRow = goalIntegrationById.get(acceptanceId);
    if (!goalRow) continue;
    const disposition = planRow.disposition;
    check(stripCode(goalRow[1]) === disposition, "INTEGRATION_DISPOSITION", acceptanceId);
    if (disposition === "assigned_execution_node") {
      const ownerNodes = planRow.nodes;
      const mappedOwnerGoals = ownerNodes.map((node) => {
        const phase = [...nodesByPhase].find(([, nodes]) => nodes.includes(node))?.[0];
        return phaseToGoal.get(phase)?.[0];
      }).filter(Boolean);
      check(sameSet([...ids(goalRow[2], "AG"), ...ids(goalRow[2], "WP")], ownerNodes),
        "INTEGRATION_OWNER_NODE", acceptanceId);
      const explicitOwnerGoals = ids(goalRow[2], "GOAL");
      if (explicitOwnerGoals.length > 0) {
        check(sameSet(explicitOwnerGoals, mappedOwnerGoals), "INTEGRATION_OWNER_GOAL", acceptanceId);
      }
    }
  }

  const baselineSection = section(goalSet, "Shared Validation Baseline Reuse Contract");
  if (baselineSection) {
    const initialOwnerValue = fieldBlock(baselineSection, "Initial Full Baseline Owner / Timing") ??
      fieldBlock(baselineSection, "Initial Full Baseline Owner / Timing / Command Set") ?? "";
    const finalOwnerValue = fieldBlock(baselineSection, "Final Full Baseline Owner / Timing") ??
      fieldBlock(baselineSection, "Final Full Baseline Owner / Timing / Command Set") ?? "";
    const initialOwner = ids(initialOwnerValue, "GOAL")[0];
    const finalOwner = ids(finalOwnerValue, "GOAL")[0];
    const executableIds = executableMembers.map((member) => member.goalId);
    check(executableIds.includes(initialOwner), "BASELINE_INITIAL_OWNER", String(initialOwner));
    check(executableIds.includes(finalOwner), "BASELINE_FINAL_OWNER", String(finalOwner));
    check(Boolean(fieldBlock(baselineSection, "Phase-Start Rule")), "BASELINE_PHASE_START", "Phase-Start Rule missing");
    check(hasAnyField(baselineSection, ["Production Input Invalidation", "Production Input Invalidation / Focused Revalidation Map"]),
      "BASELINE_PRODUCTION_INVALIDATION", "Production invalidation rule missing");
    check(hasAnyField(baselineSection, ["Test/Fixture/Probe Change", "Test/Fixture/Probe Change Invalidation / Rebaseline Rule"]),
      "BASELINE_TEST_INVALIDATION", "Test/fixture/probe rule missing");
    check(hasAnyField(baselineSection, ["Shared Boundary Change", "Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule"]),
      "BASELINE_SHARED_INVALIDATION", "Shared boundary rule missing");
    check(Boolean(fieldBlock(baselineSection, "Normal Full-Validation Count")),
      "BASELINE_FULL_COUNT", "normal full-validation count missing");
  }

  check(field(goalPayload, "Authoring Status") === "Authored", "AUTHORING_STATUS", field(goalPayload, "Authoring Status"));
  check(field(goalPayload, "Activation Status") === "Not Activated", "ACTIVATION_STATUS", field(goalPayload, "Activation Status"));
  check(field(goalPayload, "Implementation Authorized") === "No", "IMPLEMENTATION_AUTHORIZATION", field(goalPayload, "Implementation Authorized"));

  return {
    validator: VERSION,
    quality_contract: qualityContractVersion,
    result: failures.length === 0 ? "passed" : "failed",
    counts: {
      freeze_members: freezePers.length,
      phases: phases.length,
      final_execution_nodes: planNodes.length,
      goal_members: members.length,
      executable_goal_members: executableMembers.length,
      phase_to_goal_rows: mappingRows.length,
      traceability_rows: traceRows.length,
      integration_rows: goalIntegrationRows.length,
      validation_dimension_rows: validationDimensionRowCount
    },
    failures
  };
}

try {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    console.log(usage());
    process.exit(0);
  }
  for (const required of ["freeze", "plan", "receipt", "goalSet"]) {
    if (!args[required]) throw new Error(`--${required === "goalSet" ? "goal-set" : required} is required`);
    if (!fs.existsSync(args[required])) throw new Error(`file not found: ${args[required]}`);
  }
  if (!args.qualityContract) {
    throw new Error("--quality-contract is required");
  }
  const qualityContractVersion = Number(args.qualityContract);
  if (![1, 2].includes(qualityContractVersion)) {
    throw new Error("--quality-contract must be 1 or 2");
  }
  const result = validate({
    freezePath: args.freeze,
    planPath: args.plan,
    receiptPath: args.receipt,
    goalSetPath: args.goalSet,
    qualityContractVersion
  });
  console.log(JSON.stringify(result, null, 2));
  process.exit(result.result === "passed" ? 0 : 1);
} catch (error) {
  console.error(`${VERSION}: ${error.message}`);
  console.error(usage());
  process.exit(2);
}
