#!/usr/bin/env python3
"""Compare actual invalid content producers across public API and release CLI."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replace_once(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


def manifest(pack, kind, definition, filename, target):
    return f'''(pack_id:"{pack}",pack_version:"1",namespace:Global,
contract:(pack_format:1,catalog_schema:3,authoring:1,execution:2,fingerprint:1),
required_capabilities:[],dependencies:["official"],
files:[(kind:{kind},path:"{filename}")],
operations:[(target:(kind:{definition},id:"{target}"),operation:Replace,after:["official"])])'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--official-pack", type=Path, required=True)
    parser.add_argument("--scenario", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repository.resolve(strict=True)
    original = args.official_pack.resolve(strict=True)
    output = args.output_dir.absolute()
    output.mkdir()
    packs = output / "packs"
    packs.mkdir()
    before = {str(p.relative_to(original)): sha(p) for p in original.rglob("*") if p.is_file()}
    shutil.copytree(original, packs / "valid/official")
    cases = []

    def fixture(name, phase, code, source=None, pack="official", related=()):
        root = packs / name
        shutil.copytree(original, root / "official")
        case = {"name": name, "phase": phase, "code": code, "source": source,
                "pack": pack, "related": list(related), "selection": {"packs": ["official"], "winners": []}}
        cases.append(case)
        return root, case

    def mutation(name, path, old, new, phase, code):
        root, case = fixture(name, phase, code, "official/" + path)
        target = root / "official" / path
        target.write_text(replace_once(target.read_text(), old, new))
        return root, case

    root, _ = fixture("syntax", "parse", "ron_parse", "official/data/skills/base.ron")
    (root / "official/data/skills/base.ron").write_text("SkillDatabase(skills: [@])")
    root, _ = fixture("unclosed", "structure", "content_structure", "official/data/skills/base.ron")
    (root / "official/data/skills/base.ron").write_text("SkillDatabase(skills: [")
    mutation("unknown_field", "data/skills/base.ron", "SkillDatabase(",
             "SkillDatabase(unknown_authoring_field: true,", "parse", "ron_parse")
    root, _ = fixture("malformed_manifest", "parse", "ron_parse", "official/pack.ron", pack=None)
    (root / "official/pack.ron").write_text("()")
    # The production packager emits compact RON manifests.
    mutation("contract", "pack.ron", "execution:2", "execution:999", "structure", "content_contract_unsupported")
    mutation("capability", "pack.ron", "required_capabilities:[", 'required_capabilities:["unsupported_authoring_probe",',
             "structure", "content_capability_unsupported")
    mutation("final_reference", "data/summoned_units/base.ron", "skill_id: None",
             'skill_id: Some("missing_authoring_skill")', "final_validation", "content_validation")
    mutation("summon_compile", "data/summoned_units/base.ron", "max_alive: 1", "max_alive: 0", "compile", "content_compile")
    mutation("boss_compile", "data/boss_runtime/base.ron", 'entry_state_id: "acquire_target"',
             'entry_state_id: "missing_authoring_state"', "compile", "content_compile")

    policy = (original / "data/run/policy.ron").read_text()
    root, case = fixture("ambiguous", "resolution", "content_resolution", pack=None,
                         related=("left/pack.ron", "right/pack.ron"))
    for name in ("left", "right"):
        (root / name).mkdir()
        (root / name / "policy.ron").write_text(policy)
        (root / name / "pack.ron").write_text(manifest(name, "RunPolicy", "RunPolicy", "policy.ron", "run_policy"))
        case["selection"]["packs"].append(name)

    wave = '''extra_waves: [(id:"invalid_authoring_wave",
activation:(trigger_id:"invalid_authoring_wave",priority:0,primary:AtTimeMs(time_ms:5000)),
spawn_zone_ids:["missing_authoring_zone"],route_id:Some("defense_main"),required_for_victory:true,
source:Manual([CorrodedEmployee(profile_id:"corroded_guard",tier:I,count:1)]))]'''
    pos = policy.index('id: "default_facility"')
    invalid_policy = policy[:pos] + policy[pos:].replace("extra_waves: []", wave, 1)
    for name, custom in (("generated_policy", False), ("custom_generated_policy", True)):
        path = "custom/policy.ron" if custom else "official/data/run/policy.ron"
        root, case = fixture(name, "generated_setup", "generated_setup", path,
                             pack="custom" if custom else "official",
                             related=("official/data/pve/encounters.ron", "official/data/map/battlefield_templates.ron"))
        if custom:
            (root / "custom").mkdir()
            (root / "custom/pack.ron").write_text(manifest("custom", "RunPolicy", "RunPolicy", "policy.ron", "run_policy"))
            case["selection"]["packs"].append("custom")
        (root / path).write_text(invalid_policy)
        case["field"] = "combat_difficulty.profiles[1].ordinary_bands[0].tuning.extra_waves[0].spawn_zone_ids[0]"

    boss = (original / "data/boss_runtime/base.ron").read_text()
    start = boss.rfind("(", 0, boss.index('id: "cwbs_reference_summon_cycle"'))
    depth = 0
    end = None
    # Selected prototype pattern has no parentheses in strings or comments.
    for i in range(start, len(boss)):
        depth += (boss[i] == "(") - (boss[i] == ")")
        if depth == 0:
            end = i + 1
            break
    assert end
    pattern = replace_once(boss[start:end], "placement: AroundSummoner(radius_tiles: 2)",
                           'placement: SpawnZone(zone_id: "missing_authoring_summon_zone")')
    root, case = fixture("custom_generated_boss", "generated_setup", "generated_setup", "official/data/pve/encounters.ron",
                         related=("custom/pattern.ron", "official/data/boss_runtime/base.ron"))
    (root / "custom").mkdir()
    (root / "custom/pattern.ron").write_text("(patterns:[" + pattern + "])")
    (root / "custom/pack.ron").write_text(manifest("custom", "BossRuntime", "BossPattern", "pattern.ron", "cwbs_reference_summon_cycle"))
    case["selection"]["packs"].append("custom")

    for case in cases:
        (packs / case["name"] / "selection.json").write_text(json.dumps(case["selection"]))
    cases_file = output / "cases.json"
    cases_file.write_text(json.dumps(cases, indent=2) + "\n")
    fixture_before = {str(p.relative_to(packs)): sha(p) for p in packs.rglob("*") if p.is_file()}
    commands = []

    def run(name, command, expected):
        started = time.perf_counter()
        result = subprocess.run(command, cwd=repo, capture_output=True, timeout=300)
        for suffix, data in (("stdout", result.stdout), ("stderr", result.stderr)):
            (output / f"{name}.{suffix}").write_bytes(data)
        commands.append({"name": name, "command": list(map(str, command)), "expected_exit": expected,
                         "exit_code": result.returncode, "elapsed_seconds": time.perf_counter() - started})
        (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        assert result.returncode == expected, (name, result.returncode, result.stderr.decode(errors="replace"))
        print(json.dumps({"name": name, "exit_verified": expected}), flush=True)
        return result

    api = output / "api"
    binary = repo / "target/release/examples/invalid"
    cli = repo / "target/release/author_content"
    binaries = {str(p): sha(p) for p in (binary, cli)}
    run("api", [binary, packs, cases_file, args.scenario.resolve(strict=True), api], 0)
    for case in cases:
        name = case["name"]
        expected = json.loads((api / f"{name}.json").read_bytes())
        for attempt in range(2):
            result = run(f"{name}_{attempt}", [cli, "validate", packs / name, packs / name / "selection.json"], 1)
            assert not result.stdout, name
            assert json.loads(result.stderr) == expected, name
        diagnostic = expected["diagnostic"]
        assert diagnostic["phase"] == case["phase"], (name, diagnostic)
        assert diagnostic["code"] == case["code"], (name, diagnostic)
        assert diagnostic.get("pack_id") == case["pack"], (name, diagnostic)
        if case["source"]:
            assert diagnostic["source_file"] == case["source"], (name, diagnostic)
        for related in case["related"]:
            assert any(s["source_file"] == related for s in diagnostic["related_sources"]), (name, diagnostic)
        if "field" in case:
            assert diagnostic["field_path"] == case["field"], (name, diagnostic)
        if name in ("contract", "capability"):
            assert diagnostic.get("expected") is not None and diagnostic.get("actual") is not None
        if name == "syntax":
            assert diagnostic.get("position") is not None and diagnostic.get("parser_reason") is not None
        if name in ("unknown_field", "malformed_manifest"):
            assert diagnostic.get("position") is None and diagnostic.get("parser_reason") is None
        if name == "custom_generated_boss":
            assert "missing summon spawn zone" in diagnostic["cause"]
            assert any(s["source_file"] == "custom/pattern.ron" and s["pack_id"] == "custom"
                       and s["object_id"] == "cwbs_reference_summon_cycle" and s["field_path"] == "patterns[0]"
                       and s["definition_kind"] == "BossPattern" for s in diagnostic["related_sources"])
    for attempt in range(2):
        result = run(f"invalid_authority_{attempt}", [cli, "simulate", packs / "valid",
                     packs / cases[0]["name"] / "selection.json", api / "invalid_authority_scenario.json"], 1)
        assert not result.stdout
        assert json.loads(result.stderr) == json.loads((api / "invalid_authority.json").read_bytes())
    assert {str(p.relative_to(original)): sha(p) for p in original.rglob("*") if p.is_file()} == before
    assert {str(p.relative_to(packs)): sha(p) for p in packs.rglob("*") if p.is_file()} == fixture_before
    assert {str(p): sha(p) for p in (binary, cli)} == binaries
    receipt = {"passed": True, "cases": len(cases), "api_invalid_attempts": len(cases)*2,
               "cli_invalid_attempts": len(cases)*2, "commands": len(commands), "binary_sha256": binaries,
               "scenario_sha256": sha(args.scenario), "source_pack_sha256": before,
               "files": {str(p.relative_to(output)): sha(p) for p in output.rglob("*") if p.is_file()}}
    (output / "RESULT.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({k:v for k,v in receipt.items() if k not in ("files", "binary_sha256", "source_pack_sha256")}), flush=True)


if __name__ == "__main__":
    main()
