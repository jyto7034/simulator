#!/usr/bin/env python3
"""Exercise real delayed/repeated/projectile/persistent-area authored effects."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


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
    source_before = {str(p.relative_to(original)): digest(p) for p in original.rglob("*") if p.is_file()}
    path = pack / "data/skills/base.ron"
    text = path.read_text()
    marker = 'id: "paradise_lost_judgement",'
    assert text.count(marker) == 1
    start = text.rfind("SkillDef(", 0, text.index(marker))
    end = text.index("SkillDef(", text.index(marker))
    skill = '''SkillDef(
id: "paradise_lost_judgement", name: "Prototype delivery probe", kind: Targeted, focus_time_ms: 0,
cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: CurrentTarget),
reach: pattern, air_capable: false, target_range_preset: Some("front_3x2")),
steps: [
SkillStepDef(id: "delayed_repeat", delay_ms: 100, target: CastTarget, repeat: Times(count: 2),
delivery: Instant, effects: [(id: "repeat_hit", op: Damage(amount: 1, damage_type: Magic))]),
SkillStepDef(id: "delayed_projectile", delay_ms: 200, target: CastTarget,
delivery: Projectile(speed_units_per_ms: 500),
effects: [(id: "projectile_hit", op: Damage(amount: 1, damage_type: Magic))]),
SkillStepDef(id: "periodic_area", delay_ms: 300, target: CastTarget,
delivery: TileArea(footprint: (include_anchor_tile: true, rows: ["XXX", "X@X", "XXX"]),
area: (anchor: CastTarget, tile_origin: Anchor, hit_targets: Enemies, tick_policy: EveryTick,
duration_ms: 1200, tick_interval_ms: Some(300))),
effects: [(id: "area_hit", op: Damage(amount: 1, damage_type: Magic))]),
]),
'''
    path.write_text(text[:start] + skill + text[end:])
    selection = output / "selection.json"
    selection.write_text('{"packs":["official"],"winners":[]}')
    scenario = json.loads(args.scenario.read_text())
    scenario["steps"] = [scenario["steps"][0],
                         {"kind": "clock", "clock": {"mode": "elapsed", "delta_ms": 10000, "after_seq": 0}},
                         {"kind": "clock", "clock": {"mode": "elapsed", "delta_ms": 60000, "after_seq": 0}},
                         {"kind": "command", "command": {"type": "complete_combat_result"}}]
    scenario_path = output / "scenario.json"
    scenario_path.write_text(json.dumps(scenario, separators=(",", ":")))
    fixture = {"prototype_only": True, "original_pack": str(original), "original_files": source_before,
               "skill": skill, "scenario_sha256": digest(scenario_path),
               "pack_files": {str(p.relative_to(pack)): digest(p) for p in pack.rglob("*") if p.is_file()}}
    (output / "FIXTURE.json").write_text(json.dumps(fixture, indent=2) + "\n")
    binary = repo / "target/release/examples/combat"
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

    summaries = {}
    for name, mode in (("off", "off"), ("on", "on"), ("gap", "gap"), ("repeat", "on")):
        run(name, [binary, pack.parent, selection, scenario_path, output / name, "replay", mode])
        summaries[name] = json.loads((output / name / "RESULT.json").read_bytes())
        assert summaries[name]["failure"] is None and summaries[name]["is_complete"]
        assert summaries[name]["accepted_steps"] == 13
        assert summaries[name]["final_observation"]["active"] is None
        assert summaries[name]["first_terminal"] == summaries[name]["final_observation"]["last_terminal"]
    expected = (output / "off/receipts.ndjson").read_bytes()
    for name in ("on", "gap", "repeat"):
        assert (output / name / "receipts.ndjson").read_bytes() == expected, name
        assert summaries[name] == summaries["off"], name
    assert (output / "on/terminal_trace.ndjson").read_bytes() == (output / "repeat/terminal_trace.ndjson").read_bytes()
    rows = [json.loads(line) for line in (output / "on/terminal_trace.ndjson").read_bytes().splitlines()]
    assert len(rows) == summaries["on"]["first_terminal"]["mandatory"]["applications"]
    assert len({row["ordinal"] for row in rows}) == len(rows)
    trace = json.loads((output / "on/terminal_trace.json").read_bytes())
    assert trace["gap"] is None and trace["key"] == summaries["on"]["first_terminal"]["key"]
    gap = json.loads((output / "gap/terminal_trace.json").read_bytes())
    assert gap["rows"] == 0 and gap["gap"] is not None
    actions = {}
    for action, effect in (("delayed_repeat", "repeat_hit"), ("delayed_projectile", "projectile_hit"), ("periodic_area", "area_hit")):
        selected = [row for row in rows if row["source"] and row["source"]["authored"]["site"]["action"]["action_id"] == action]
        assert selected, action
        assert all(row["change"]["channel"] == "hp" and row["change"]["before"] > row["change"]["after"] for row in selected)
        assert all(row["source"]["authored"]["site"]["effect_id"] == effect for row in selected)
        assert all(row["source"]["skill_id"] == "paradise_lost_judgement" for row in selected)
        actions[action] = selected
        query = output / f"{action}_query.json"
        query.write_text(json.dumps({"query": "effect", "site": selected[0]["source"]["authored"]["site"]}))
        explanation = json.loads(run("explain_" + action, [cli, "explain", pack.parent, selection, query]))
        assert explanation["source"] == selected[0]["source"]["authored"]
        assert explanation["identity"] == summaries["off"]["identity"]
    repeats = actions["delayed_repeat"]
    pair = next((a, b) for a, b in zip(repeats, repeats[1:]) if a["source"] == b["source"] and a["time_ms"] == b["time_ms"] and a["target_unit"] == b["target_unit"])
    assert pair[0]["ordinal"] != pair[1]["ordinal"] and pair[0]["change"]["after"] == pair[1]["change"]["before"]
    area = actions["periodic_area"]
    ticks = next((a, b) for a, b in zip(area, area[1:]) if a["source"] == b["source"] and a["target_unit"] == b["target_unit"] and b["time_ms"] - a["time_ms"] == 300)
    assert ticks[0]["ordinal"] != ticks[1]["ordinal"]
    for mode in ("off", "on"):
        command = [cli, "simulate", pack.parent, selection, scenario_path]
        if mode == "on":
            command += ["--trace-output", output / "cli_trace.ndjson"]
        assert run("cli_" + mode, command) == expected
    assert {str(p.relative_to(original)): digest(p) for p in original.rglob("*") if p.is_file()} == source_before
    assert {str(p.relative_to(pack)): digest(p) for p in pack.rglob("*") if p.is_file()} == fixture["pack_files"]
    assert {str(p): digest(p) for p in (binary, cli)} == binaries
    result = {"passed": True, "commands": len(commands), "api_replays": 4, "cli_simulations": 2,
              "cli_explanations": 3, "binary_sha256": binaries, "repeat_pair": pair, "area_tick_pair": ticks,
              "action_counts": {name: len(values) for name, values in actions.items()},
              "terminal_time_ms": summaries["off"]["first_terminal"]["actual_time_ms"],
              "semantic_occurrence_key": "battle key + actual application ordinal; authored source, cast, target and time retained",
              "scope": "actual HP applications; native delivery UUID not exposed; log eviction/sequence-shift and Preview separate",
              "files": {str(p.relative_to(output)): digest(p) for p in output.rglob("*") if p.is_file()}}
    (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: v for k, v in result.items() if k not in ("files", "binary_sha256", "repeat_pair", "area_tick_pair")}), flush=True)


if __name__ == "__main__":
    main()
