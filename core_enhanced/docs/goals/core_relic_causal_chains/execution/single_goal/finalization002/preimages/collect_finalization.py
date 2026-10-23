#!/usr/bin/env python3
"""Collect explicit U10 evidence without running tests or changing old artifacts.

Authored by the independent test author. Execute only as the assigned sole Sol
runner after fixed-source functional/performance collection. This collector is
not a product oracle or an independent completion audit. Pending bindings stay
pending; a failed parent receipt never becomes a passing whole-run receipt.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from datetime import datetime, timezone
from functools import lru_cache
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / "Cargo.toml").is_file()
            and (p / "core_enhanced/src/lib.rs").is_file()
            and (p / "game_server/Cargo.toml").is_file())
EXECUTION = HERE.parent
GOAL = EXECUTION.parent.parent
PRIOR = ROOT / "core_enhanced/docs/goals/core_relic_structural_activation/execution/single_goal/finalization001/generated004"


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def digest(value) -> str:
    return hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


@lru_cache(maxsize=None)
def artifact_ref(path: Path):
    data = path.read_bytes()
    return {"path": str(path.relative_to(ROOT)), "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def ref(path: Path, pointer: str | None = None, row=None):
    result = dict(artifact_ref(path))
    if pointer is not None:
        result.update(json_pointer=pointer, canonical_row_sha256=digest(row))
    return result


def stamped(value):
    if not isinstance(value, dict) or not isinstance(value.get("path"), str):
        return None
    path = Path(value["path"])
    if not path.is_absolute():
        path = ROOT / path
    observed = ref(path)
    for key in ("sha256", "bytes"):
        if key in value and observed[key] != value[key]:
            raise ValueError(f"artifact drift: {path} {key}")
    return observed


def test_rows(stdout: str):
    starts = list(re.finditer(r"^test ([A-Za-z0-9_:]+) \.\.\. ", stdout, re.M))
    rows = []
    for index, match in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(stdout)
        block = stdout[match.end():end]
        # --nocapture prints may share the first line or precede the status.
        # Bound at the first terminal line so later failure listings/results
        # cannot turn an interrupted test into a PASS.
        status = re.search(r"^(ok|FAILED|ignored)(?:\s.*)?$", block, re.M)
        if status is None:
            raise ValueError(f"missing libtest terminal status: {match.group(1)}")
        rows.append({"selector": match.group(1), "status": status.group(1),
                     "raw_block": block[:status.end()].rstrip(),
                     "raw_block_sha256": hashlib.sha256(block[:status.end()].encode()).hexdigest()})
    return rows


def chosen(row, spec):
    name = row["selector"]
    prefixes = spec.get("include_prefixes", []) + ([spec["include_prefix"]] if "include_prefix" in spec else [])
    suffixes = spec.get("include_suffixes", [])
    positive = not (prefixes or suffixes) or any(name.startswith(x) for x in prefixes) or any(name.endswith("::" + x) for x in suffixes)
    return positive and not any(name.startswith(x) for x in spec.get("exclude_prefixes", [])) and not any(name.endswith("::" + x) for x in spec.get("exclude_suffixes", []))


def receipt_evidence(spec):
    path = EXECUTION / spec["receipt"]
    receipt = load(path)
    command = receipt
    if "command_index" in spec:
        command = receipt.get("command_results", receipt.get("commands", []))[spec["command_index"]]
    if spec.get("raw_format") == "generic_combined_log":
        stdout_ref = ref(path.with_suffix(".log"))
        if stdout_ref["sha256"] != command["log_sha256"]:
            raise ValueError(f"{spec['id']}: combined raw log drift")
        stderr_ref = None
    else:
        stdout_ref = stamped(command["stdout"])
        stderr_ref = stamped(command["stderr"])
    stdout = (ROOT / stdout_ref["path"]).read_text(encoding="utf-8")
    all_rows = test_rows(stdout)
    selected = [row for row in all_rows if chosen(row, spec)]
    if not selected or len(selected) != spec["selected_count"]:
        raise ValueError(f"{spec['id']}: expected {spec['selected_count']} selected tests; got {len(selected)}")
    if any(row["status"] != "ok" for row in selected):
        raise ValueError(f"{spec['id']}: selected row is not PASS")
    if command.get("exit_code") != 0 and not spec.get("preserve_enclosing_failure"):
        raise ValueError(f"{spec['id']}: failed receipt requires explicit partial-row selection")
    unchanged = command.get("source_unchanged", receipt.get("source_unchanged"))
    if unchanged is False:
        raise ValueError(f"{spec['id']}: source changed during evidence execution")
    source = command.get("source_before", command.get("input", receipt.get("source_before", receipt.get("input"))))
    source_ref = stamped(source)
    if source_ref is None and isinstance(source, dict):
        source_ref = {"embedded_in_receipt": True, "sha256": source.get("sha256"), "canonical_row_sha256": digest(source)}
    return {"id": spec["id"], "status": "named_pass_rows_available_currentness_review_required",
            "scope": spec["scope"], "receipt": ref(path), "command_index": spec.get("command_index"),
            "enclosing_exit_code": command.get("exit_code"), "enclosing_failure_preserved": command.get("exit_code") != 0,
            "source": source_ref, "source_unchanged": unchanged,
            "stdout": stdout_ref, "stderr": stderr_ref, "selected_count": len(selected),
            "selected_tests": selected,
            "unselected_rows": [{"selector": r["selector"], "status": r["status"]} for r in all_rows if r not in selected]}


def artifact_evidence(spec):
    path = EXECUTION / spec["path"]
    receipt = load(path)
    if spec.get("raw_format") == "generic_combined_log":
        log = ref(path.with_suffix(".log"))
        if receipt.get("exit_code") != 0 or receipt.get("source_unchanged") is False or log["sha256"] != receipt["log_sha256"]:
            raise ValueError(f"{spec['id']}: failed/drifted generic artifact")
        result_path = EXECUTION / spec["result_artifact"] if spec.get("result_artifact") else path.with_suffix(".log")
        result_ref = ref(result_path) if spec.get("result_artifact") else None
        observed_fields = {}
        if spec.get("expected_json_fields"):
            value = load(result_path)
            for field, expected in spec["expected_json_fields"].items():
                actual = value
                for key in field.split("."):
                    actual = actual[int(key)] if isinstance(actual, list) else actual[key]
                if actual != expected:
                    raise ValueError(f"{spec['id']}: {field}: expected {expected!r}, observed {actual!r}")
                observed_fields[field] = actual
        return {"id": spec["id"], "status": "bounded_commands_available_currentness_review_required",
                "scope": spec["scope"], "receipt": ref(path), "combined_log": log,
                "exit_code": receipt["exit_code"], "command": receipt["command"],
                "source_sha256": receipt.get("source_before", {}).get("sha256"),
                "source_unchanged": receipt.get("source_unchanged"), "observed_fields": observed_fields,
                "result_artifact": result_ref,
                "supporting_artifacts": [ref(ROOT / item) for item in spec.get("supporting_artifacts", [])]}
    results = receipt.get("command_results", receipt.get("commands", []))
    selected = []
    for index in spec["command_indices"]:
        command = results[index]
        if command.get("exit_code") != 0:
            raise ValueError(f"{spec['id']}: selected command{index} did not succeed")
        if command.get("source_unchanged") is False:
            raise ValueError(f"{spec['id']}: selected command source drift")
        selected.append({"command_index": index, "exit_code": command["exit_code"],
                         "stdout": stamped(command["stdout"]), "stderr": stamped(command["stderr"]),
                         "command": command.get("command", command.get("argv")),
                         "source": stamped(command.get("input", receipt.get("input")))})
    return {"id": spec["id"], "status": "bounded_commands_available_currentness_review_required",
            "scope": spec["scope"], "receipt": ref(path), "selected_commands": selected,
            "unselected_command_exits": [{"command_index": i, "exit_code": c.get("exit_code")} for i, c in enumerate(results) if i not in spec["command_indices"]]}


def original_literal(row):
    for key in ("requirement", "expected", "text", "Observable Policy Acceptance", "acceptance"):
        if key in row:
            return row[key]
    raise ValueError("original requirement literal not found")


def old_selectors(row):
    result = set(row.get("current_selectors", []))
    result.update(row.get("original_exact_selectors_reexecuted_if_any", []))
    for claim in row.get("claims", []):
        result.update(claim.get("selectors", []))
    return sorted(result)


def build_impact(bindings, evidence, bindings_path):
    old_impact_path = PRIOR / "PREDECESSOR_IMPACT.json"
    old_coverage_path = PRIOR / "COVERAGE.json"
    old, coverage = load(old_impact_path), load(old_coverage_path)
    prior_selection_path = PRIOR / "EVIDENCE_SELECTION.json"
    prior_evidence = load(prior_selection_path)["evidence"]

    def inherited_selectors(row):
        names = set(old_selectors(row))
        seen = set()
        def visit(key):
            if key in seen:
                return
            seen.add(key)
            item = prior_evidence[key]
            names.update(test["selector"] for test in item.get("selected_tests", []))
            for member in item.get("members", []):
                visit(member)
        for key in row.get("current_evidence_refs", row.get("evidence_refs", [])):
            visit(key)
        return names

    current_selectors = {r["selector"] for e in evidence.values() for r in e.get("selected_tests", [])}
    inputs = [(r["id"], r, old_impact_path, f"/rows/{i}", r["original_literal"], r["category"])
              for i, r in enumerate(old["rows"])]
    inputs += [(r["id"], r, old_coverage_path, f"/acceptances/{i}", r["text"], bindings["u09_sentence_categories"][r["id"]])
               for i, r in enumerate(coverage["acceptances"])]
    expected = bindings["expected_predecessor_ids"]
    if len(inputs) != 288 or len(set(x[0] for x in inputs)) != 288 or [x[0] for x in inputs] != expected:
        raise ValueError("predecessor latest254 + U09 sentences34 exact ordered ID mismatch")
    rows = []
    for identifier, row, path, pointer, literal, category_name in inputs:
        category = bindings["categories"][category_name]
        for affected in category["paths"]:
            if not (ROOT / affected).is_file():
                raise ValueError(f"missing affected source: {affected}")
        pending = [key for key in category["evidence"] if evidence[key]["status"] == "pending"]
        rows.append({"id": identifier, "original_literal": literal,
                     "original_owner": row.get("original_owner", ref(old_coverage_path)),
                     "original_ref_with_pointer_and_digest": row.get("original_ref_with_pointer_and_digest", ref(path, pointer, row)),
                     "latest_predecessor_row_ref": ref(path, pointer, row),
                     "original_evidence_ref": row.get("original_evidence_ref", ref(path, pointer + "/evidence_refs", row.get("evidence_refs", []))),
                     "inherited_current_evidence_refs": row.get("current_evidence_refs", row.get("evidence_refs", [])),
                     "category": category_name, "disposition": "split_inherited_component_and_current_changed_consumer",
                     "unchanged_component": category["unchanged"], "changed_component": category["changed"],
                     "changed_component_paths": category["paths"], "current_evidence_refs": category["evidence"],
                     "inherited_exact_selectors_selected_if_any": sorted(inherited_selectors(row) & current_selectors),
                     "pending_or_unresolved_evidence": pending,
                     "status": "pending_current_evidence" if pending else "split_evidence_available_root_currentness_and_audit_required"})
    return {"schema": "U10_PREDECESSOR_IMPACT_V1", "counts": {"inherited_latest_u09_rows": 254, "u09_sentences": 34, "union": 288},
            "review": ref(HERE / bindings["review"]), "bindings": ref(bindings_path),
            "source_requirements": [ref(old_impact_path), ref(old_coverage_path), ref(prior_selection_path)], "rows": rows, "limits": bindings["limits"]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-manifest", required=True, help="Current fixed source INPUT manifest; must match workspace")
    parser.add_argument("--output", required=True, help="Fresh output directory, never a historical evidence directory")
    parser.add_argument("--late-bindings", help="Optional explicit additional receipt/artifact bindings for pending keys")
    parser.add_argument("--coverage-input", default=str(HERE / "COVERAGE_INPUT.json"),
                        help="Optional separate coverage mapping; original mapping is the default")
    parser.add_argument("--predecessor-bindings", default=str(HERE / "PREDECESSOR_BINDINGS.json"),
                        help="Optional separate predecessor mapping; original bindings are the default")
    args = parser.parse_args()
    source_path = ROOT / args.source_manifest
    source = load(source_path)
    for entry in source["files"]:
        stamped(entry)
    config = load(HERE / "EVIDENCE_INPUT.json")
    mapping_path = ROOT / args.coverage_input
    bindings_path = ROOT / args.predecessor_bindings
    mapping = load(mapping_path)
    bindings = load(bindings_path)
    evidence = {spec["id"]: receipt_evidence(spec) for spec in config["receipts"]}
    evidence.update({spec["id"]: artifact_evidence(spec) for spec in config["additional_artifact_bindings"]})
    for spec in config["bounded_subcases"]:
        path = EXECUTION / spec["receipt"]
        receipt = load(path)
        stdout = stamped(receipt["stdout"])
        body = (ROOT / stdout["path"]).read_text(encoding="utf-8")
        if spec["required_marker"] not in body or receipt["exit_code"] == 0:
            raise ValueError("bounded historical failure subcase marker/status changed")
        evidence[spec["id"]] = {**spec, "status": "bounded_historical_subcase_root_review_required", "receipt": ref(path), "stdout": stdout, "enclosing_exit_code": receipt["exit_code"],
                                "bounded_artifacts": [ref(EXECUTION / item) for item in spec.get("artifact_paths", [])]}
    for spec in config["pending_bindings"]:
        evidence[spec["id"]] = {**spec, "status": "pending", "recorded_pending_stage": spec["status"]}
    late = {}
    if args.late_bindings:
        late_path = ROOT / args.late_bindings
        late = load(late_path)
        for spec in late.get("receipts", []):
            evidence[spec["id"]] = receipt_evidence(spec)
        for spec in late.get("artifacts", []):
            evidence[spec["id"]] = artifact_evidence(spec)
        for item in late.get("composites", []):
            children = [evidence[key] for key in item["members"]]
            if not children or any(child["status"] == "pending" for child in children):
                raise ValueError(f"incomplete composite binding {item['id']}")
            evidence[item["id"]] = {**item, "status": "composite_available_currentness_review_required"}
    impact = build_impact(bindings, evidence, bindings_path)
    # Mapping can be self-reviewed before the independent completion audit.
    # Keep that audit pending in its own rows without making the evidence
    # inventory depend circularly on the future auditor's verdict.
    impact_pending = any(set(row["pending_or_unresolved_evidence"]) - {"independent_completion"} for row in impact["rows"])
    evidence["predecessor_impact"] = {"id": "predecessor_impact", "status": "pending" if impact_pending else "mapped_root_review_required", "artifact": "PREDECESSOR_IMPACT.json", "counts": impact["counts"]}
    policy_path = GOAL / "P2G_30_FREEZE_001.json"
    policy = load(policy_path)["artifacts"][0]["payload"]["Observable Policy Acceptance Set"]
    acceptances, seams = [], []
    for index, element in enumerate(policy, 1):
        sentences = element["Observable Policy Acceptance"]
        if len(sentences) != 2:
            raise ValueError("unexpected frozen sentence count")
        ids = []
        for ordinal, literal in enumerate(sentences, 1):
            identifier = f"PE-CRCE-{index:02d}.{ordinal}"
            binding = mapping["requirement_evidence"][identifier]
            pending = [key for key in binding["evidence"] if evidence[key]["status"] == "pending"]
            acceptances.append({"id": identifier, "text": literal, "policy_ref": element["Policy Element Revision Ref"],
                                "independent_oracle": binding["oracle"], "evidence_refs": binding["evidence"],
                                "pending_evidence": pending,
                                "status": "pending" if pending else "evidence_available_currentness_review_required"})
            ids.append(identifier)
    phase_path = GOAL / "P2G_40_PHASE_PLAN_001.json"
    phase = load(phase_path)["artifacts"][0]["payload"]
    for index, frozen in enumerate(phase["integration"]):
        binding = mapping["integration_evidence"][frozen["id"]]
        seams.append({**frozen, "phase_plan_row_ref": ref(phase_path, f"/artifacts/0/payload/integration/{index}", frozen),
                      "sentence_refs": binding.get("sentence_refs", []), "evidence_refs": binding["evidence"],
                      "pending_evidence": [key for key in binding["evidence"] if evidence[key]["status"] == "pending"]})
    node_ids = [node for item in phase["phases"] for node in item["nodes"]]
    dimensions = {"success", "invalid_input", "failure_rollback", "retry_idempotency", "ordering_concurrency",
                  "boundary_empty_max", "disconnect_restart_persistence", "live_data", "producer_consumer", "legacy_fallback_absence"}
    if (len(acceptances) != 28 or len(seams) != 19 or set(mapping["dimension_evidence"]) != dimensions
            or len(node_ids) != 3 or set(mapping["execution_nodes"]) != set(node_ids)
            or set(mapping["integration_evidence"]) != {row["id"] for row in phase["integration"]}
            or set(mapping["requirement_evidence"]) != {row["id"] for row in acceptances}):
        raise ValueError("current goal exact coverage membership mismatch")
    covered = {key for row in acceptances for key in row["evidence_refs"]}
    for key in covered:
        if key not in evidence:
            raise ValueError(f"missing evidence key {key}")
    common = {"recorded_at_utc": datetime.now(timezone.utc).isoformat(), "goal": "GOAL-CRCE-01", "goal_set": "GS-CRCE-001",
              "source": ref(source_path), "source_sha256": source["sha256"], "collector": ref(Path(__file__).resolve()),
              "independent_completion": "pending_nonparticipant_audit_not_implied_by_collection"}
    selection = {**common, "schema": "U10_EVIDENCE_SELECTION_V1", "selection_input": ref(HERE / "EVIDENCE_INPUT.json"),
                 "late_bindings": ref(ROOT / args.late_bindings) if args.late_bindings else None,
                 "late_failed_attempts_to_preserve": [ref(EXECUTION / name) for name in late.get("preserved_failed_attempts", [])],
                 "source_review": ref(EXECUTION / config["currentness_review"]), "evidence": evidence,
                 "historical_failures_to_preserve": config["historical_failures_to_preserve"]}
    coverage = {**common, "schema": "U10_COVERAGE_V1", "status": "pending_currentness_performance_restore_and_independent_completion",
                "frozen_policy": ref(policy_path), "mapping_input": ref(mapping_path),
                "acceptances": acceptances, "integration_seams": seams,
                "required_dimensions": [{"id": key, "evidence_refs": value, "pending_evidence": [x for x in value if evidence[x]["status"] == "pending"]} for key, value in mapping["dimension_evidence"].items()],
                "execution_nodes": [{"id": key, **value, "pending_evidence": [x for x in value["evidence"] if evidence[x]["status"] == "pending"]} for key, value in mapping["execution_nodes"].items()], "limits": mapping["noncompletion_limits"]}
    output = ROOT / args.output
    output.mkdir(parents=True, exist_ok=False)
    for filename, payload in (("EVIDENCE_SELECTION.json", selection), ("PREDECESSOR_IMPACT.json", {**common, **impact}), ("COVERAGE.json", coverage)):
        (output / filename).write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(output.relative_to(ROOT)), "named_selected_tests": sum(e.get("selected_count", 0) for e in evidence.values()),
                      "acceptance_sentences": len(acceptances), "integration_seams": len(seams), "dimensions": len(mapping["dimension_evidence"]),
                      "predecessor_rows": len(impact["rows"]), "pending_evidence": [key for key, item in evidence.items() if item["status"] == "pending"],
                      "completion": "not_claimed"}, ensure_ascii=False))


if __name__ == "__main__":
    main()
