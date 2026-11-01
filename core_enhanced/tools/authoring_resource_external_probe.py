#!/usr/bin/env python3
"""Actual external RON resonance A/B through the public release API and CLI."""
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
    out = args.output_dir.absolute()
    out.mkdir()
    scenario = out / "scenario.json"
    scenario.write_bytes(args.scenario.read_bytes())
    selection = out / "selection.json"
    selection.write_text('{"packs":["official"],"winners":[]}')
    relative = Path("data/skills/base.ron")
    before = (original / relative).read_text()
    marker = 'id: "paradise_lost_judgement",'
    assert before.count(marker) == 1
    start = before.index(marker)
    end = before.index("SkillDef(", start)
    section = before[start:end]
    insertion = section.rindex("            ],")
    amounts = {"a": 17, "b": 31}
    for generation, amount in amounts.items():
        pack = out / generation / "official"
        shutil.copytree(original, pack)
        step = '''                SkillStepDef(
                    id: "authoring_resource",
                    delay_ms: 0,
                    target: SelfUnit,
                    delivery: Instant,
                    effects: [(id: "resource_gain", op: ModifyResonance(amount: AMOUNT))],
                ),
'''.replace("AMOUNT", str(amount))
        changed = before[:start] + section[:insertion] + step + section[insertion:] + before[end:]
        (pack / relative).write_text(changed)
    files = []
    for path in sorted((out / "a/official").rglob("*")):
        if path.is_file():
            name = path.relative_to(out / "a/official")
            files.append({"path": str(name), "a_sha256": digest(path),
                          "b_sha256": digest(out / "b/official" / name)})
    assert [f["path"] for f in files if f["a_sha256"] != f["b_sha256"]] == [str(relative)]
    assert (out / "a/official" / relative).read_text().replace(
        'op: ModifyResonance(amount: 17)', 'op: ModifyResonance(amount: 31)'
    ) == (out / "b/official" / relative).read_text()
    site = {"action": {"definition_id": "paradise_lost_judgement", "action_id": "authoring_resource"},
            "effect_id": "resource_gain"}
    (out / "FIXTURE.json").write_text(json.dumps({"prototype_only": True, "original": str(original),
        "shared_change": "existing skill gets a self-target resonance step", "amounts": amounts,
        "source_site": site, "files": files}, indent=2) + "\n")
    query = out / "query.json"
    query.write_text(json.dumps({"query": "effect", "site": site}))
    commands, results, witnesses = [], {}, {}

    def run(name, command):
        started = time.perf_counter()
        result = subprocess.run(command, cwd=repo, capture_output=True, timeout=300)
        (out / f"{name}.stdout").write_bytes(result.stdout)
        (out / f"{name}.stderr").write_bytes(result.stderr)
        commands.append({"name": name, "command": [str(p) for p in command], "exit_code": result.returncode,
                         "elapsed_seconds": time.perf_counter() - started})
        (out / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        assert result.returncode == 0, (name, result.returncode, result.stderr.decode(errors="replace"))
        print(json.dumps({"name": name, "exit_verified": 0}), flush=True)
        return result.stdout

    for generation in amounts:
        for mode in ("off", "on", "gap"):
            name = f"{generation}_{mode}"
            run(name, [repo / "target/release/examples/combat", out / generation, selection,
                       scenario, out / name, "replay", mode])
            result = json.loads((out / name / "RESULT.json").read_text())
            assert result["failure"] is None and result["is_complete"] and result["accepted_steps"] == 13
            assert result["first_terminal"] == result["final_observation"]["last_terminal"]
            assert result["final_observation"]["active"] is None
            receipts = [json.loads(line) for line in (out / name / "receipts.ndjson").read_text().splitlines()]
            assert len(receipts) == 15
            assert receipts[-1]["payload"]["request"]["command"]["type"] == "complete_combat_result"
            assert receipts[-1]["payload"]["public_result"]["disposition"] == "accepted"
            trace = json.loads((out / name / "terminal_trace.json").read_text())
            count = result["first_terminal"]["mandatory"]["applications"]
            assert trace["key"] == result["first_terminal"]["key"]
            if mode == "off":
                assert not trace["enabled"] and trace["rows"] == 0 and trace["gap"] is None
            elif mode == "gap":
                assert trace["rows"] == 0 and trace["gap"]["dropped_rows"] == count
                assert (out / name / "terminal_trace.ndjson").read_bytes() == b""
            else:
                rows = [json.loads(line) for line in (out / name / "terminal_trace.ndjson").read_text().splitlines()]
                assert trace["gap"] is None and len(rows) == trace["rows"] == count
                resources = [row for row in rows if row["change"]["channel"] == "resonance"
                             and row["source"] and row["source"]["authored"]["site"] == site]
                assert resources, "real sourced resource application is required"
                first = resources[0]
                assert first["change"]["after"] - first["change"]["before"] == amounts[generation]
                assert first["source_unit"] == first["target_unit"]
                witnesses[generation] = {"first": first, "applications": len(resources)}
            results[name] = result
        expected = (out / f"{generation}_off/receipts.ndjson").read_bytes()
        for mode in ("on", "gap"):
            assert results[f"{generation}_{mode}"] == results[f"{generation}_off"]
            assert (out / f"{generation}_{mode}/receipts.ndjson").read_bytes() == expected
        assert run(f"cli_{generation}", [repo / "target/release/author_content", "simulate",
                   out / generation, selection, scenario]) == expected
        explanation = json.loads(run(f"explain_{generation}", [repo / "target/release/author_content", "explain",
                                     out / generation, selection, query]))
        assert explanation["source"]["site"] == site
        assert explanation["effect"]["op"]["ModifyResonance"]["amount"] == amounts[generation]
        assert explanation["identity"] == results[f"{generation}_off"]["identity"]
    a, b = witnesses["a"]["first"], witnesses["b"]["first"]
    for field in ("ordinal", "time_ms", "source_unit", "target_unit", "source"):
        assert a[field] == b[field], field
    assert a["change"]["before"] == b["change"]["before"]
    assert results["a_off"]["identity"] != results["b_off"]["identity"]
    assert results["a_off"]["input_sha256"] == results["b_off"]["input_sha256"] == digest(scenario)
    result = {"passed": True, "commands": len(commands), "api_replays": 6, "cli_simulations": 2,
              "cli_explanations": 2, "witnesses": witnesses, "amounts": amounts, "input_sha256": digest(scenario),
              "files": [{"path": str(p.relative_to(out)), "sha256": digest(p)}
                        for p in sorted(out.rglob("*")) if p.is_file()]}
    (out / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({k: v for k, v in result.items() if k != "files"}), flush=True)


if __name__ == "__main__":
    main()
