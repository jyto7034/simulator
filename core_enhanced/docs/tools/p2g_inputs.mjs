// Input adapters only. Policy approval, currentness and design quality remain stage reviews.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

export const sha256 = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");
export const stripCode = (value) => String(value ?? "").trim().replace(/^`(.*)`$/, "$1");
export const equal = (a, b) => JSON.stringify(canonical(a)) === JSON.stringify(canonical(b));
function canonical(v) {
  if (Array.isArray(v)) return v.map(canonical);
  if (v && typeof v === "object") return Object.fromEntries(Object.keys(v).sort().map(k => [k, canonical(v[k])]));
  return v;
}
export function requireValue(ok, code, detail) {
  if (!ok) throw new Error(`${code}: ${detail}`);
}
export function displayPath(file) {
  const resolved = path.resolve(file), relative = path.relative(process.cwd(), resolved);
  return (relative && !relative.startsWith("..") ? relative : resolved).split(path.sep).join("/");
}
export function field(md, label) {
  const escaped = label.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return stripCode(md.match(new RegExp(`^- ${escaped}:\\s*(.+)$`, "m"))?.[1]) || null;
}
export function fieldBlock(md, label) {
  const lines = md.split(/\r?\n/), prefix = `- ${label}:`;
  const start = lines.findIndex(line => line.startsWith(prefix));
  if (start < 0) return null;
  const values = [lines[start].slice(prefix.length).trim()];
  for (const line of lines.slice(start + 1)) {
    if (!line.trim() || !/^\s+/.test(line)) break;
    values.push(line.trim());
  }
  return values.join(" ");
}
export function section(md, title) {
  const lines = md.split(/\r?\n/);
  title = title.replace(/^#+\s*/, "");
  const start = lines.findIndex(line => line.match(/^#+\s+(.*)$/)?.[1].trim() === title);
  if (start < 0) return "";
  const level = lines[start].match(/^#+/)[0].length;
  const rest = lines.slice(start + 1);
  const end = rest.findIndex(line => (line.match(/^(#+)\s/)?.[1].length ?? Infinity) <= level);
  return rest.slice(0, end < 0 ? undefined : end).join("\n");
}
export function tableRows(md, pattern) {
  return md.split(/\r?\n/).filter(line => line.startsWith("|"))
    .map(line => line.split("|").slice(1, -1).map(stripCode))
    .filter(row => pattern.test((row[0] ?? "").replaceAll("`", "")));
}
export const ids = (text, prefix) => [...String(text ?? "").matchAll(new RegExp(`${prefix}-[A-Z0-9-]+`, "g"))].map(m => m[0]);
// Legacy comma-separated selector cells; do not infer policy IDs from their prefix.
export function selectorIds(text) {
  return String(text ?? "").replace(/`/g, "")
    .replace(/\b(?:implementation|supporting|validation_only):/g, "")
    .split(/[,;\s]+/).filter(Boolean);
}
export function exactRef(ref, type) {
  requireValue(ref && typeof ref === "object" && !Array.isArray(ref), "EXACT_REF", type);
  requireValue(typeof ref["Artifact ID"] === "string" && ref["Artifact ID"].trim() &&
    typeof ref["Artifact Type"] === "string" && (!type || ref["Artifact Type"] === type) &&
    Object.hasOwn(ref, "Content Digest") && (ref["Content Digest"] === null || typeof ref["Content Digest"] === "string"), "EXACT_REF", JSON.stringify(ref));
  return ref["Artifact ID"];
}
function artifactRefs(v) {
  if (Array.isArray(v)) return v.flatMap(artifactRefs);
  if (!v || typeof v !== "object") return [];
  if (Object.hasOwn(v, "Artifact Type") || Object.hasOwn(v, "Artifact ID")) { exactRef(v); return [v]; }
  return Object.values(v).flatMap(artifactRefs);
}
const list = (v, name) => {
  requireValue(Array.isArray(v), "ARRAY_REQUIRED", name); return v;
};
function unique(values, name) {
  requireValue(values.length === new Set(values).size, "DUPLICATE", name);
}
function read(file) {
  const bytes = fs.readFileSync(file), text = bytes.toString("utf8");
  return {bytes, text, json: /^[\s]*[\[{]/.test(text) ? JSON.parse(text) : null};
}

export function readFreeze(file) {
  const input = read(file);
  if (!input.json) {
    // Historical Markdown naming conventions are isolated here, never applied to JSON IDs.
    const block = input.text.match(/- Ordered (?:`code_reviewed`|code-reviewed) Policy Element Revision Ref Set:([\s\S]*?)- Decision Revision Ref Set:/)?.[1] ?? "";
    const revisions = ids(block, "PER"); unique(revisions, "Freeze members");
    const members = revisions.map(revision => ({revision, element: revision.replace(/^PER-/, "PE-")
      .replace(/-(OPEN|DECIDED|CODE)-\d+$/, "").replace(/-R\d+-(OPEN|DECIDED|CODE-REVIEWED)$/, "")}));
    const decisionBlock = input.text.match(/- Decision Revision Ref Set:([\s\S]*?)- Current Code Review Binding Ref Set:/)?.[1] ?? "";
    const decisions = ids(decisionBlock, "DR").length ? decisionBlock :
      input.text.match(/- Current Decision Revision Ref Set:([\s\S]*?)(?=\n- )/)?.[1] ?? "";
    return {...input, format: "legacy_markdown", ref: field(input.text, "Final Policy Freeze Ref"), members,
      decisions: [...new Set(ids(decisions, "DR"))], working: field(input.text, "Working Revision Ref"),
      review: field(input.text, "Review Batch Ref"), source: field(input.text, "Current Reviewed Source Basis Ref"),
      bindings: ids(fieldBlock(input.text, "Current Code Review Binding Ref Set"), "CRB"), resolvedSources: []};
  }
  const freezes = list(input.json.artifacts, "artifacts").filter(a => a.ref?.["Artifact Type"] === "Final Policy Freeze");
  requireValue(freezes.length === 1, "FREEZE_IDENTITY", "exactly one Freeze per input container");
  const freeze = freezes[0], ref = exactRef(freeze.ref, "Final Policy Freeze");
  // Resolve only within this Topic, with no recursive repository scan or invented aliases.
  const registry = new Map(), sources = new Map();
  for (const name of fs.readdirSync(path.dirname(file)).filter(n => n.endsWith(".json")).sort()) {
    const sourceFile = path.join(path.dirname(file), name);
    let doc;
    try { doc = JSON.parse(fs.readFileSync(sourceFile, "utf8")); } catch { continue; }
    if (!Array.isArray(doc.artifacts)) continue;
    for (const artifact of doc.artifacts) {
      if (!artifact.ref) continue;
      const key = JSON.stringify([artifact.ref["Artifact Type"], artifact.ref["Artifact ID"]]);
      if (!registry.has(key)) registry.set(key, []);
      registry.get(key).push({artifact, sourceFile});
    }
  }
  const done = new Set();
  function resolve(r, stack = new Set()) {
    exactRef(r);
    const key = JSON.stringify([r["Artifact Type"], r["Artifact ID"]]);
    const found = registry.get(key) ?? [];
    requireValue(found.length === 1, "REF_RESOLUTION", `${key}: ${found.length} matches`);
    const {artifact, sourceFile} = found[0];
    requireValue(equal(r, artifact.ref), "REF_MISMATCH", key);
    requireValue(artifact.payload && typeof artifact.payload === "object", "PAYLOAD_MISSING", key);
    requireValue(!stack.has(key), "REF_CYCLE", key);
    sources.set(displayPath(sourceFile), sha256(fs.readFileSync(sourceFile)));
    if (!done.has(key)) {
      const next = new Set([...stack, key]);
      for (const dependency of artifactRefs(artifact.payload)) resolve(dependency, next);
      done.add(key);
    }
    return artifact.payload;
  }
  const p = resolve(freeze.ref);
  const revisions = list(p["Ordered code_reviewed Policy Element Revision Ref Set"], "Freeze members");
  unique(revisions.map(r => exactRef(r, "Policy Element Revision")), "Freeze members");
  const members = revisions.map(r => {
    const element = resolve(r);
    requireValue(element["Intrinsic status"] === "code_reviewed", "MEMBER_STATUS", r["Artifact ID"]);
    requireValue(typeof element["Stable Element ID"] === "string" && element["Stable Element ID"].trim(), "ELEMENT_ID", r["Artifact ID"]);
    requireValue(list(element["Observable Policy Acceptance"], "member acceptance").length > 0, "ACCEPTANCE_EMPTY", r["Artifact ID"]);
    return {element: element["Stable Element ID"], revision: r["Artifact ID"], ref: r,
      decision: element["Decision Revision Ref"], acceptance: element["Observable Policy Acceptance"]};
  });
  unique(members.map(m => m.element), "Stable Element ID");
  const working = resolve(p["Working Revision Ref"]);
  requireValue(equal(working["Current Policy Element Revision Map"], Object.fromEntries(members.map(m => [m.element, m.ref]))), "WR_MEMBER_MAP", ref);
  requireValue(equal(p["Observable Policy Acceptance Set"], members.map(m => ({"Policy Element Revision Ref": m.ref, "Observable Policy Acceptance": m.acceptance}))), "ACCEPTANCE_PROJECTION", ref);
  requireValue(members.every(m => p["Decision Revision Ref Set"].some(d => equal(d, m.decision))), "MEMBER_DECISION", ref);
  return {...input, format: "json", ref, exactRef: freeze.ref, payload: p, members,
    decisions: p["Decision Revision Ref Set"].map(r => exactRef(r, "Decision Revision")),
    working: exactRef(p["Working Revision Ref"], "Working Revision"), review: exactRef(p["Review Batch Ref"], "Review Batch"),
    source: exactRef(p["Reviewed Source Basis Ref"], "Reviewed Source Basis"),
    bindings: p["Current Code Review Binding Ref Set"].map(r => exactRef(r, "Code Review Binding")),
    resolvedSources: [...sources].sort(([a], [b]) => a.localeCompare(b)).map(([path, sha256]) => ({path, sha256}))};
}

export function readPlan(file) {
  const input = read(file);
  if (input.json) {
    const plans = list(input.json.artifacts, "artifacts").filter(a => a.ref?.["Artifact Type"] === "Phase Plan");
    requireValue(plans.length === 1, "PLAN_IDENTITY", "exactly one Phase Plan per input container");
    const a = plans[0], p = a.payload;
    const ref = exactRef(a.ref, "Phase Plan");
    for (const key of ["work_packages", "dependencies", "atomic_groups", "standalone", "phases", "audits", "integration", "cyclic_sccs"]) list(p[key], key);
    for (const w of p.work_packages) {
      requireValue(w.selectors && Object.keys(w.selectors).every(k => ["implementation", "supporting", "validation_only"].includes(k)), "SELECTOR_ROLE", w.id);
      Object.values(w.selectors).forEach(v => list(v, "selectors"));
    }
    for (const phase of p.phases) {list(phase.primary, "primary"); list(phase.nodes, "nodes");}
    for (const group of p.atomic_groups) list(group.members, "atomic members");
    return {...input, ...p, format: "json", ref, exactRef: a.ref,
      freezeRef: exactRef(p["Bound Final Policy Freeze Ref"], "Final Policy Freeze"),
      receiptRef: exactRef(input.json.validation_receipt_ref, "Phase Plan Validation Receipt"),
      method: input.json.validator_method_version,
      repair: p["Applicable P2G-40 Obligation Key Set"], resolution: p["P2G-40 Resolution Record Ref Set"],
      supersession: p["Applicable Obligation Supersession Record Ref Set"],
      goalFact: p["Goal Structure Fact"], parallelization: p.Parallelization};
  }
  const md = input.text;
  const rows = (title, prefix) => tableRows(section(md, title), new RegExp(`^${prefix}-`));
  const repairBasis = fieldBlock(md, "Repair Basis");
  function repairSet(label) {
    const value = fieldBlock(md, label);
    if (!value && /explicit empty/.test(repairBasis ?? "")) return [];
    requireValue(value, "REPAIR_BASIS", label);
    if (/^`?(explicit empty|empty|N\/A)`?$/i.test(value)) return [];
    const members = [...value.matchAll(/`([^`]+)`/g)].map(m => m[1]);
    requireValue(members.length, "REPAIR_BASIS", label); return members;
  }
  return {...input, format: "legacy_markdown", ref: field(md, "Phase Plan Ref"),
    receiptRef: field(md, "Phase Plan Validation Receipt Ref"), freezeRef: field(md, "Bound Final Policy Freeze Ref"),
    method: field(md, "Validator/Method Version"), goalFact: field(md, "Goal Structure Fact"),
    parallelization: field(md, "Parallelization"), parallelEvidence: section(md, "Parallelization Evidence"), repairBasis,
    repair: repairSet("Applicable P2G-40 Obligation Key Set"), resolution: repairSet("P2G-40 Resolution Record Ref Set"),
    supersession: repairSet("Applicable Obligation Supersession Record Ref Set"),
    work_packages: rows("Work Package Set", "WP").map(r => ({id:r[0], selectors:{implementation:selectorIds(r[1])}, rolePresent:/implementation:|supporting:|validation_only:/.test(r[1])})),
    dependencies: rows("Dependency Graph", "WP").map(r => ({from:r[0], to:r[1], type:r[2], evidence:r[3], broken:r[4]})),
    atomic_groups: rows("Atomic Group Set", "AG").map(r => ({id:r[0], members:ids(r[1],"WP"), evidence:r[2]})),
    standalone: ids(section(md,"Atomic Group Set").split(/\r?\n/).find(l => l.startsWith("Standalone Final Execution Nodes:")),"WP"),
    phases: rows("Ordered Phase Set","PHASE").map(r => ({id:r[0], primary:selectorIds(r[1]), nodes:[...ids(r[3],"AG"),...ids(r[3],"WP")]})),
    audits: rows("Intermediate-State Audit","PHASE").map(r => ({phase:r[0], state:r[1], checks:r[2], evidence:r[3], limitation:r[4]})),
    cyclic_sccs: rows("Cyclic SCC Disposition Set","WP").map(r => ({members:ids(r[0],"WP"), disposition:r[1], evidence:r[3]})),
    integration: rows("Integration Validation Ownership Map","IVA").map(r => ({id:stripCode(r[0].split(/\s/)[0]), source:r[0], disposition:r[1], locator:r[2], nodes:[...ids(r[3],"AG"),...ids(r[3],"WP")], phases:ids(r[3],"PHASE"), prerequisite:r[4], evidence:r[5]}))};
}
