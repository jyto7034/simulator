"""Bind S1 acceptance and inherited selectors to completed current evidence."""
import copy
import hashlib
import json
import re
from pathlib import Path
import run

H, R = run.HERE, run.ROOT
E = H / "evidence"
S = H / "execution/s1"
SUITES = ["all_default_002", "all_feature_002", "server_all_targets_001"]
current = run.identity()
passed = {}
suite_counts = {}


def read(name):
    return json.loads((E / (name + ".json")).read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


for suite in SUITES:
    receipt = read(suite)
    log = E / (suite + ".log")
    assert receipt["exit_code"] == 0 and receipt["source_unchanged"]
    assert receipt["source_after"] == current
    assert receipt["log_sha256"] == sha(log)
    text = log.read_text()
    passed[suite] = set(re.findall(r"^test (\S+) \.\.\. ok$", text, re.M))
    totals = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
    suite_counts[suite] = {key: sum(int(row[index]) for row in totals) for index, key in enumerate(("passed", "failed", "ignored"))}
    assert suite_counts[suite]["passed"] > 0 and not suite_counts[suite]["failed"] and not suite_counts[suite]["ignored"]

core = passed[SUITES[0]] & passed[SUITES[1]]
host = passed[SUITES[2]]


def select(fragments, candidates=core):
    selected = set()
    for fragment in fragments:
        found = {name for name in candidates if fragment in name}
        assert found, fragment
        selected.update(found)
    return sorted(selected)


contracts = {
    "V01": ("All38 original numbers; default RON/no-file effective identity and unchanged gameplay/error oracles", ["original_defaults_and_no_file_profile"], ["default_parity_002", "public_profile_003", "public_api_cli_001", "public_oracle_001", "public_cli_001"]),
    "V02": ("Bounded strict schema and file errors before content publication", ["settings_reject_schema", "structural_errors_identify", "parser_is_bounded"], ["public_profile_003"]),
    "V03": ("Raw/effective identity, concurrent compile profiles, immutable generation and explicit mismatch", ["raw_and_effective_identity", "external_generation_keeps_semantics", "public_content_parser_compiler"], ["public_profile_003"]),
    "V04": ("First native operation, deferred/nested allowance, current-host checkpoint and opaque rollback", ["selected_native_limits", "native_deferred_world_budget", "native_clone_limit_is_cumulative", "native_retained_admission_does_not_refund", "runtime_safety_checkpoint_restore", "persistence_rollback_tests::"], ["public_boundary_002"]),
    "V05": ("Profile authoring admission, trial work/receipt scope, existing session survives replacement failure, real API/CLI options", ["explicit_authoring_admission", "real_trial_replacement_failures", "authoring::trial::tests::"], ["public_profile_003", "public_api_cli_001", "public_cli_001"]),
    "V06": ("Selected expression parser/compiler/evaluator and unchanged readonly preview semantics and legal typed range", ["public_content_parser_compiler", "compiler_limits_preserve", "game::value_expr::tests::", "game::value_expr::parse::tests::", "authoring::trial::tests::amount"], ["public_api_cli_001", "public_cli_001", "pc_compare_002"]),
    "V07": ("Host-window import/streaming deserialize, dirty edit accounting, cursor recovery, actual battle beyond small span and final statistics", ["history_import_and_deserialize", "a_short_history_span", "game::battle::event_log::tests::", "expired_live_history_rejects_strict_cursor", "selected_native_limits"], []),
    "V08": ("Real bootstrap, public command/authority/reconnect and persistence/monotonic rollback", [], ["public_profile_003", "public_boundary_002", "server_all_targets_001", "clippy_server_001", "release_001"]),
    "V12": ("Full current Core/host; inherited F01/R06/R07 and current PC36/R05 guard evidence", [], ["check_001", "event_log_check_001", "clippy_core_001", "clippy_server_001", "fmt_001", "release_001", "public_oracle_001", "public_trace_001", "public_cli_001", "pc_compare_002", "r05_compare_002"]),
}
host_selectors = {
    "V01": ["runtime_safety_host_file"],
    "V02": ["runtime_safety_host_file", "real_pack_bootstrap"],
    "V03": ["real_pack_bootstrap"],
    "V04": ["first_public_battle_entry"],
    "V08": ["bootstrap::tests::", "game::player_game_actor::handlers::tests::", "persistence::active_run_store::tests::", "persistence::player_profile_store::tests::", "messages::tests::"],
}
validations = []
for key, (requirement, selectors, receipts) in contracts.items():
    validations.append(dict(id=key, requirement=requirement, core_selectors=select(selectors),
                            host_selectors=select(host_selectors.get(key, []), host),
                            suite_receipts=SUITES, direct_receipts=receipts,
                            status="self_verified; independent semantic audit pending"))
required = sorted({name for _, _, names in contracts.values() for name in names})
receipt_bindings = []
for name in required:
    receipt = read(name)
    assert receipt["exit_code"] == 0 and receipt["source_unchanged"], name
    assert receipt["log_sha256"] == sha(E / (name + ".log")), name
    # Earlier runtime probes exclude only the newly captured test-only includes.
    old = {row["path"]: row for row in receipt["source_after"]["files"]}
    now = {row["path"]: row for row in current["files"]}
    assert all(now.get(path) == row for path, row in old.items()), name
    added = sorted(set(now) - set(old))
    assert all(path in {row["path"] for row in json.loads((S / "baseline/COMPILE_INPUTS_BEFORE.json").read_text())["files"]} for path in added)
    receipt_bindings.append(dict(name=name, receipt_sha256=sha(E / (name + ".json")),
                                 exact_current_membership=not added, additional_test_only_inputs=added))

freeze = json.loads((H / "P2G_30_FREEZE_001.json").read_text())["artifacts"][0]
acceptance = copy.deepcopy(freeze["payload"]["Observable Policy Acceptance Set"][:7])
for index, entry in enumerate(acceptance, 1):
    entry["validation_refs"] = [f"V{index:02d}"] + (["V08"] if index in (1, 2, 3, 4, 5) else [])
dimensions = {
    "success": ["V01", "V03", "V05", "V08"],
    "invalid_input": ["V02"],
    "failure_rollback": ["V04", "V05", "V08"],
    "retry_idempotency": ["V04", "V08"],
    "ordering_concurrency": ["V03", "V04"],
    "boundary_empty_max": ["V02", "V06", "V07"],
    "disconnect_restart_persistence": ["V04", "V08"],
    "live_data": ["V01", "V05", "V08"],
    "producer_consumer": ["V03", "V04", "V05", "V06", "V07", "V08"],
    "legacy_fallback_absence": ["V01", "V02", "V08"],
}
prior = R / "core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/corrections/f01_action_flow_20261004_001"
old_coverage = json.loads((prior / "COVERAGE_CURRENT.json").read_text())
inherited = copy.deepcopy(old_coverage["conditions"])
for condition in inherited:
    assert set(condition["selectors"]) <= core, condition["id"]
    condition["predecessor_directly_affected"] = condition.pop("directly_affected", None)
    condition["current_impact"] = "default/profile/generation and public port changes; current selectors and semantic audit required"
    condition["receipts"] = SUITES[:2]
    condition["disposition"] = "selectors rerun; default profile/history/host impact requires this S1 independent audit"
for selector in old_coverage["correction_selectors"]:
    assert selector in core
impact = dict(predecessor_coverage_sha256=sha(prior / "COVERAGE_CURRENT.json"),
              predecessor_manifest_sha256=sha(prior / "target/f01_action_flow_001/MANIFEST.json"),
              current_source_sha256=current["sha256"], inherited_conditions=inherited,
              r06_acceptances=old_coverage["r06_acceptances"], f01_selectors=old_coverage["correction_selectors"],
              public_oracle="../../evidence/public_after/SUMMARY.json", public_trace="../../evidence/public_after_trace/SUMMARY.json",
              public_cli="../../evidence/cli_current/CLI_FINAL_SUMMARY_002.json", independent_audit=False)
result = dict(goal="GOAL-CRSO-S1", source_sha256=current["sha256"], frozen_policy_ref=freeze["ref"],
              phase="PHASE-CRSO-S1", status="self_verification_complete; independent audit pending",
              acceptance=acceptance, validations=validations, required_dimensions=dimensions,
              suites=suite_counts, receipt_bindings=receipt_bindings,
              cost_evidence={"PC36": "../../evidence/PC_SUMMARY_CURRENT_002.json", "R05": "../../evidence/R05_SUMMARY_CURRENT_002.json"},
              limitations=["Clippy warnings are retained; no zero-warning claim.", "R05 raw outcome and approved concurrent-peak-only warning classification are distinct.", "No deployed Unity or remote service claim; actual server tests used temporary local endpoints.", "S2 observation/export/Grafana is excluded and waits for S1 independent acceptance."])
for name, value in (("COVERAGE.json", result), ("PREDECESSOR_IMPACT.json", impact)):
    with (S / name).open("x") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
print(json.dumps(dict(validations=len(validations), frozen_acceptances=len(acceptance), dimensions=len(dimensions), inherited_conditions=len(inherited), suites=suite_counts)))
