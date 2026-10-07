#!/usr/bin/env python3
"""Prepare positive SkillPreview through external rewards and public gameplay."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def once(text, before, after):
    assert text.count(before) == 1, before
    return text.replace(before, after)


def edit_definition(path, marker, next_marker, edit):
    text = path.read_text()
    matches = list(re.finditer(r"(?m)^[ \t]*" + re.escape(marker), text))
    assert len(matches) == 1, marker
    start = matches[0].start()
    end = text.index(next_marker, start)
    path.write_text(text[:start] + edit(text[start:end]) + text[end:])


def binding(step, pointer, to, kind="string"):
    return {"from": {"step": step, "pointer": pointer}, "to": to, "value_type": kind}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--prototype-pack", type=Path, required=True)
    parser.add_argument("--scenario", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repository.resolve(strict=True)
    original = args.prototype_pack.resolve(strict=True)
    output = args.output_dir.absolute()
    output.mkdir()
    pack = output / "packs/official"
    shutil.copytree(original, pack)
    before = {str(p.relative_to(original)): digest(p) for p in original.rglob("*") if p.is_file()}
    fragment = "authoring_preview_fragment"
    skill_id = "fragment_freischutz_black_round"
    edit_definition(pack / "data/events/rewards/base.ron", 'id: "enkephalin_reward",', "RewardMetadata(",
                    lambda s: once(s, "GrantEnkephalin(amount: 100),", 'GrantEnkephalin(amount: 100),\n                GrantSkillFragment(fragment_id: "'+fragment+'"),'))
    # Encounter-source fragments are reserved for encounter progression rewards.
    # This prototype is independently authored for the supported ordinary source.
    fragments = pack / "data/skill_fragments/base.ron"
    metadata = '''SkillFragmentMetadata(
id:"authoring_preview_fragment",uuid:"00000000-0000-0000-0000-00000000ca05",
name:"Prototype manual preview",description:"Isolated query acceptance fixture",rarity:Rare,
equip_limit:OwnedCopies,origin:Some(Concept(concept_id:"authoring_preview")),sources:[RareReward],
dependencies:[],compatibility:(),effect:ActiveSkill(imitation_skill_id:"fragment_freischutz_black_round",
upgrade_skill_ids:{},awakened_skill_id:None)),
'''
    fragments.write_text(once(fragments.read_text(), "fragments: [", "fragments: [\n" + metadata))
    skill_path = pack / "data/skills/base.ron"
    edit_definition(skill_path, f'id: "{skill_id}",', "SkillDef(", lambda s:
                    once(once(once(s, "reach: pattern,", "reach: global,"),
                              'target_range_preset: Some("front_3x2"),', "target_range_preset: None,"),
                         "Damage(amount: 42, damage_type: Magic)", "Damage(amount: 5, damage_type: Magic)"))
    edit_definition(skill_path, 'id: "paradise_lost_judgement",', "SkillDef(", lambda s:
                    once(s, "            ],\n        ),", '''                SkillStepDef(id: "preview_charge", delay_ms: 0, target: SelfUnit,
                    delivery: Instant, effects: [(id: "charge", op: ModifyResonance(amount: 100))]),
            ],
        ),'''))
    templates = pack / "data/map/facility_templates.ron"
    edge = '(from_slot_id: "intake_corridor", to_slot_id: "supply_annex", direction: Bidirectional),'
    templates.write_text(once(templates.read_text(), edge, edge + '\n                (from_slot_id: "intake_corridor", to_slot_id: "containment_guard", direction: Bidirectional),'))
    scenario = json.loads(args.scenario.read_text())
    steps = scenario["steps"]
    assert len(scenario["preparation_steps"]) == 9 and len(steps) == 4
    steps.append({"kind": "command", "command": {"type": "equip_skill_fragment", "employee_uuid": None, "fragment_id": fragment},
                  "bindings": [binding(9, "/payload/employee_uuid", "/command/employee_uuid")]})
    steps.append({"kind": "command", "command": {"type": "select_map_node", "node_id": None},
                  "bindings": [binding(12, "/payload/completion/payload/map/nodes/0/id", "/command/node_id")]})
    steps.append({"kind": "command", "command": {"type": "confirm_enter_node"}})
    pause = copy.deepcopy(scenario["preparation_steps"][6])
    for b in pause["bindings"]:
        assert b["from"]["step"] == 5
        b["from"]["step"] = 15
    steps.append(pause)
    steps.append({"kind": "snapshot", "snapshot": "run"})
    deploy = copy.deepcopy(steps[0])
    for b in deploy["bindings"]:
        if b["to"] == "/command/employee_uuid":
            b["from"] = {"step": 9, "pointer": "/payload/employee_uuid"}
        else:
            b["from"]["step"] = {4: 14, 6: 16}[b["from"]["step"]]
    steps.append(deploy)
    steps.append({"kind": "clock", "clock": {"mode": "elapsed", "delta_ms": 10000, "after_seq": 0}})
    assert len(steps) == 11
    scenario_path = output / "discovery_scenario.json"
    scenario_path.write_text(json.dumps(scenario, separators=(",", ":")))
    selection = output / "selection.json"
    selection.write_text('{"packs":["official"],"winners":[]}')
    after = {str(p.relative_to(pack)): digest(p) for p in pack.rglob("*") if p.is_file()}
    changed = sorted(k for k in before if before[k] != after[k])
    assert changed == ["data/events/rewards/base.ron", "data/map/facility_templates.ron", "data/skill_fragments/base.ron", "data/skills/base.ron"]
    (output / "FIXTURE.json").write_text(json.dumps({"prototype_only": True, "original": str(original),
        "original_files": before, "pack_files": after, "changed_files": changed,
        "purpose": "actual reward/equip/second battle/manual focus and positive readonly query; no private mutation"}, indent=2) + "\n")
    binary = repo / "target/release/examples/inspection"
    cli = repo / "target/release/author_content"
    binaries = {str(p): digest(p) for p in (binary, cli)}
    commands = []

    def run(name, command):
        started = time.perf_counter()
        result = subprocess.run(command, cwd=repo, capture_output=True, timeout=300)
        (output / f"{name}.stdout").write_bytes(result.stdout)
        (output / f"{name}.stderr").write_bytes(result.stderr)
        commands.append({"name": name, "command": list(map(str, command)), "exit_code": result.returncode,
                         "elapsed_seconds": time.perf_counter() - started})
        (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        assert result.returncode == 0, (name, result.returncode, result.stderr.decode(errors="replace"))
        print(json.dumps({"name": name, "exit_verified": 0}), flush=True)
        return result.stdout

    run("validate", [cli, "validate", pack.parent, selection])
    api = output / "api"
    run("api", [binary, pack.parent, selection, scenario_path, api])
    expected = (api / "baseline_receipts.ndjson").read_bytes()
    for mode in ("off", "on"):
        command = [cli, "simulate", pack.parent, selection, api / "scenario.json"]
        if mode == "on":
            command += ["--trace-output", output / "cli_trace.ndjson"]
        assert run("cli_" + mode, command) == expected
    assert {str(p.relative_to(original)): digest(p) for p in original.rglob("*") if p.is_file()} == before
    assert {str(p.relative_to(pack)): digest(p) for p in pack.rglob("*") if p.is_file()} == after
    assert {str(p): digest(p) for p in (binary, cli)} == binaries
    report = json.loads((api / "RESULT.json").read_bytes())
    result = {"passed": True, "commands": len(commands), "api_replays": report["api_replays"],
              "discovery_replays": report["discovery_replays"], "positive_previews": report["positive_previews"],
              "binary_sha256": binaries, "files": {str(p.relative_to(output)): digest(p) for p in output.rglob("*") if p.is_file()}}
    (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: v for k, v in result.items() if k not in ("files", "binary_sha256")}), flush=True)


if __name__ == "__main__":
    main()
