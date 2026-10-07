#!/usr/bin/env python3
"""R05 synthetic typed resources through current default-release public consumers."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--cases", help="Comma-separated existing case names; omitted runs all cases")
    args = parser.parse_args()
    repo, source = args.repository.resolve(strict=True), args.fixture.resolve(strict=True)
    output = args.output_dir.absolute()
    output.mkdir()
    binaries = {name: repo / "target/release" / name for name in (
        "package_official_content", "consume_content_pack", "author_content",
        "examples/combat", "examples/typed_resources", "examples/typed_adversarial")}
    commands, results = [], []

    def run(label, command, allow_failure=False):
        started = time.perf_counter()
        command = list(map(str, command))
        result = subprocess.run(command, cwd=repo, capture_output=True, timeout=300)
        (output / f"{label}.stdout").write_bytes(result.stdout)
        (output / f"{label}.stderr").write_bytes(result.stderr)
        commands.append(dict(label=label, command=command, exit_code=result.returncode,
                             elapsed_seconds=time.perf_counter() - started))
        (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        assert allow_failure or result.returncode == 0, (label, result.stderr.decode(errors="replace"))
        return result

    base = output / "official-current"
    run("package", [binaries["package_official_content"], repo / "game_resources", base])
    run("consume", [binaries["consume_content_pack"], output, "official-current"])
    old_data = source / "packs/official/data"
    differences = [str(p.relative_to(old_data)) for p in sorted(old_data.rglob("*.ron"))
                   if p.read_bytes() != (repo / "game_resources/data" / p.relative_to(old_data)).read_bytes()]
    assert differences == ["equipments/base.ron", "skills/base.ron"], differences
    source_hashes = {str(p.relative_to(source)): sha(p) for p in source.rglob("*") if p.is_file()}
    cases = [dict(name=life.lower(), lifetime=life, initial=7, cap=15, consume=2, triggers=1)
             for life in ("Run", "Attempt", "Deployment", "Activation", "Delivery", "Chain")]
    cases += [
        dict(name="initial", lifetime="Run", initial=9, cap=15, consume=2, triggers=1),
        dict(name="operation", lifetime="Run", initial=7, cap=15, consume=3, triggers=1),
        dict(name="cap", lifetime="Run", initial=7, cap=19, consume=2, triggers=1, grant=True),
        dict(name="new_id", lifetime="Run", initial=7, cap=15, consume=2, triggers=1, suffix="_new"),
        dict(name="independent_chain", lifetime="Chain", initial=7, cap=15, consume=2, triggers=2),
        dict(name="persistent_run", lifetime="Run", initial=7, cap=15, consume=2, triggers=2),
        dict(name="two_copies", lifetime="Run", initial=7, cap=15, consume=2, triggers=1, copies=2),
        dict(name="resource_32", lifetime="Run", initial=7, cap=15, consume=2, triggers=1, pairs=16),
        dict(name="resource_256", lifetime="Run", initial=7, cap=15, consume=2, triggers=1, pairs=128, expected_runtime_fault="battle_runtime_clone_limit"),
    ]
    if args.cases:
        selected = args.cases.split(",")
        assert set(selected) <= {case["name"] for case in cases} and len(set(selected)) == len(selected)
        cases = [case for case in cases if case["name"] in selected]
    for case in cases:
        name = case["name"]
        directory = output / name
        directory.mkdir()
        packs = directory / "packs"
        packs.mkdir()
        official = packs / "official"
        shutil.copytree(base, official)
        for relative in differences:
            shutil.copyfile(old_data / relative, official / "data" / relative)
        equipment = official / "data/equipments/base.ron"
        text = equipment.read_text()
        marker = "internal_cooldown_ms: 1500)"
        assert text.count(marker) == 1
        equipment.write_text(text.replace(marker, f"internal_cooldown_ms: 1500, max_triggers_per_battle: Some({case['triggers']}))"))
        pairs = case.get("pairs", 1)
        if case.get("copies") == 2:
            authored = equipment.read_text()
            start = authored.index('id: "standard_training_sword"')
            end = authored.index('id: "standard_armor"', start)
            sword = authored[start:end]
            old_range = 'rows: [".X.", ".@.", "..."]'
            assert sword.count(old_range) == 1
            sword = sword.replace(old_range, 'rows: ["XXXXXXX", "XXXXXXX", "XXXXXXX", "XXX@XXX", "XXXXXXX", "XXXXXXX", "XXXXXXX"]')
            equipment.write_text(authored[:start] + sword + authored[end:])
        ids = [prefix + case.get("suffix", "") + (f"_{n}" if pairs > 1 else "")
               for n in range(pairs) for prefix in ("employee_meter", "copy_meter")]
        (official / "data/resources/base.ron").write_text("(resources: [" + ",".join(
            f'(id:"{resource}",owner:{owner},lifetime:{case["lifetime"]},initial:{case["initial"]},cap:{case["cap"]})'
            for resource, owner in zip(ids, ("Employee", "EquipmentCopy") * pairs)) + "])\n")
        operations = []
        for resource, owner in zip(ids, ("SourceEmployee", "SourceEquipment") * pairs):
            mutations = (["Grant(30)", "Decay(3)"] if case.get("grant") else []) + [f'Consume({case["consume"]})']
            operations += [f'(resource:"{resource}",owner:{owner},operation:{mutation})' for mutation in mutations]
        skills = official / "data/skills/base.ron"
        text = skills.read_text()
        marker = 'effects: [(id: "repeat_hit", op: Damage(amount: 1, damage_type: Magic))]'
        assert text.count(marker) == 1
        text = text.replace(marker, 'effects: [(id:"typed_cost",op:Resources(operations:[' + ",".join(operations)
                            + '])),(id:"repeat_hit",op:Damage(amount:1,damage_type:Magic))]')
        text = text.replace('id: "delayed_repeat", delay_ms: 100, target: CastTarget, repeat: Times(count: 2)',
                            'id: "delayed_repeat", delay_ms: 100, target: CastTarget, repeat: Times(count: 1)')
        skills.write_text(text)
        for filename in ("selection.json", "scenario.json"):
            shutil.copyfile(source / filename, directory / filename)
        selection, scenario = directory / "selection.json", directory / "scenario.json"
        if case.get("copies") == 2:
            document = json.loads(scenario.read_text())
            second = json.loads(json.dumps(document["steps"][0]))
            for binding in second["bindings"]:
                if binding["to"].startswith("/context/"):
                    binding["from"]["step"] = 9
                elif binding["to"] == "/command/employee_uuid":
                    binding["from"]["pointer"] = binding["from"]["pointer"].replace("costs/0/", "costs/1/")
                elif binding["to"] == "/command/position":
                    binding["from"]["pointer"] = binding["from"]["pointer"].replace("cells/3", "cells/2")
            document["steps"].insert(1, second)
            scenario.write_text(json.dumps(document))
        describe = json.loads(run(name + "_describe", [binaries["author_content"], "describe", packs, selection]).stdout)
        assert describe["typed_resources"]["lifetimes"] == ["Run", "Attempt", "Deployment", "Activation", "Delivery", "Chain"]
        assert len(describe["catalog_kinds"]) == 25 and len(describe["definition_kinds"]) == 43
        query = directory / "resource_query.json"
        query.write_text(json.dumps({"query": "definition", "key": {"kind": "Resource", "id": ids[0]}}))
        explained = json.loads(run(name + "_explain", [binaries["author_content"], "explain", packs, selection, query]).stdout)
        assert explained["value"]["initial"] == case["initial"] and explained["value"]["cap"] == case["cap"]
        trial = run(name + "_trial", [binaries["examples/combat"], packs, selection, scenario, directory / "trial", "replay", "off"], allow_failure=bool(case.get("expected_runtime_fault")))
        report = json.loads((directory / "trial/RESULT.json").read_text())
        if case.get("expected_runtime_fault"):
            expected = case["expected_runtime_fault"]
            assert trial.returncode != 0 and report["accepted_steps"] == 11 and not report["is_complete"]
            assert report["failure"]["error"]["diagnostic"]["code"] == expected
            refused = run(name + "_cli_refused", [binaries["author_content"], "simulate", packs, selection, scenario], allow_failure=True)
            assert refused.returncode == 1 and json.loads(refused.stderr) == report["failure"]
            assert refused.stdout == (directory / "trial/receipts.ndjson").read_bytes()
            run(name + "_native_refused", [binaries["examples/typed_resources"], packs, selection, scenario, directory / "native_refused.json"])
            native = json.loads((directory / "native_refused.json").read_text())
            assert native["failure"]["code"] == expected and native["failure"]["before"] == native["failure"]["after"]
            run(name + "_adversarial", [binaries["examples/typed_adversarial"], packs, selection, scenario, directory / "adversarial.json"])
            adversarial = json.loads((directory / "adversarial.json").read_text())
            assert adversarial["passed"] and adversarial["resource_entries"] == 256
            split = directory / "adversarial.scenario.json"
            run(name + "_native_split", [binaries["examples/typed_resources"], packs, selection, split, directory / "native_split.json"])
            completed = json.loads((directory / "native_split.json").read_text())
            assert completed["failure"] is None and len(completed["final"]["run"]["entries"]) == 256
            assert completed["final"]["run"] == adversarial["split_final"]["observation"]["run_resources"]
            assert completed["terminal_facts"] == adversarial["split_final"]["observation"]["last_terminal"]["runtime_facts"]
            split_receipts = "".join(row + "\n" for row in adversarial["split_final"]["receipts"]).encode()
            assert run(name + "_cli_split", [binaries["author_content"], "simulate", packs, selection, split]).stdout == split_receipts
            results.append(dict(case=case, outcome="bounded_refusal_and_explicit_clock_completion", identity=completed["identity"]))
            (output / "progress.json").write_text(json.dumps(results, indent=2) + "\n")
            print(json.dumps({"case":name,"passed":True,"outcome":"bounded_refusal_and_explicit_clock_completion"}), flush=True)
            continue
        assert report["failure"] is None and report["is_complete"]
        facts = report["first_terminal"]["runtime_facts"]
        typed = [p["summary"]["latest_cost"] for p in facts["providers"]
                 if p["summary"].get("latest_cost") and isinstance(p["summary"]["latest_cost"]["resource"], dict)
                 and "typed" in p["summary"]["latest_cost"]["resource"]]
        assert typed, name
        count = case["triggers"] if case["lifetime"] == "Run" else 1
        expected = case["cap"] - 3 - case["consume"] if case.get("grant") else case["initial"] - count * case["consume"]
        assert all(cost["after"] == expected and cost["resource"]["typed"]["lifetime"] == case["lifetime"] for cost in typed), (name, typed, expected)
        assert all(cost["source"]["equipment"]["persistent_copy_uuid"] for cost in typed)
        if case.get("copies") == 2:
            assert len({cost["source"]["equipment"]["persistent_copy_uuid"] for cost in typed}) == 2
            assert len({cost["source"]["equipment"]["definition_uuid"] for cost in typed}) == 1
        run(name + "_native", [binaries["examples/typed_resources"], packs, selection, scenario, directory / "native.json"])
        native = json.loads((directory / "native.json").read_text())
        assert native["failure"] is None and native["steps"] == 12 + case.get("copies", 1)
        assert native["terminal_facts"] == facts
        entries = native["final"]["run"]["entries"]
        assert len(entries) == (2 * pairs * case.get("copies", 1) if case["lifetime"] == "Run" else 0)
        assert all(entry["balance"] == expected for entry in entries)
        assert native["final"]["battle"] is None
        receipts = (directory / "trial/receipts.ndjson").read_bytes()
        assert run(name + "_cli", [binaries["author_content"], "simulate", packs, selection, scenario]).stdout == receipts
        if name == "run":
            for mode in ("on", "gap", "off"):
                destination = directory / ("repeat_" + mode)
                run("run_" + mode, [binaries["examples/combat"], packs, selection, scenario, destination, "replay", mode])
                assert (destination / "receipts.ndjson").read_bytes() == receipts
                assert json.loads((destination / "RESULT.json").read_text()) == report
        results.append(dict(case=case, expected=expected, typed_costs=typed, identity=native["identity"]))
        (output / "progress.json").write_text(json.dumps(results, indent=2) + "\n")
        print(json.dumps({"case": name, "passed": True}), flush=True)
    assert source_hashes == {str(p.relative_to(source)): sha(p) for p in source.rglob("*") if p.is_file()}
    result = dict(passed=True, cases=results, commands=commands, source_differences=differences,
                  source_fixture=str(source), source_files=source_hashes,
                  binaries={name: sha(path) for name, path in binaries.items()},
                  files={str(p.relative_to(output)): sha(p) for p in output.rglob("*") if p.is_file()})
    (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"passed": True, "cases": len(results)}), flush=True)


if __name__ == "__main__":
    main()
