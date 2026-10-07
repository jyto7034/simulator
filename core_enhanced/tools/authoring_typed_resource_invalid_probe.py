#!/usr/bin/env python3
"""Strict typed-resource negatives through production API/CLI, with immutable inputs."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time


def hashes(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in root.rglob("*") if p.is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("repository", "official-pack", "scenario", "output-dir"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    repo, original = args.repository.resolve(), args.official_pack.resolve()
    output = args.output_dir.absolute()
    output.mkdir()
    packs = output / "packs"
    packs.mkdir()
    source_hashes = hashes(original)
    shutil.copytree(original, packs / "valid/official")
    definitions, skills = "data/resources/base.ron", "data/skills/base.ron"
    mutations = [
        ("missing_initial", definitions, "initial:7,", "", "parse", "ron_parse"),
        ("missing_cap", definitions, ",cap:15", "", "parse", "ron_parse"),
        ("negative", definitions, "initial:7", "initial:-1", "parse", "ron_parse"),
        ("overflow", definitions, "cap:15", "cap:4294967296", "parse", "ron_parse"),
        ("nonfinite", definitions, "initial:7", "initial:NaN", "parse", "ron_parse"),
        ("wrong_type", definitions, "initial:7", 'initial:"7"', "parse", "ron_parse"),
        ("owner_kind", definitions, "owner:Employee", "owner:Account", "parse", "ron_parse"),
        ("lifetime", definitions, "lifetime:Run", "lifetime:Forever", "parse", "ron_parse"),
        ("above_cap", definitions, "initial:7", "initial:16", "final_validation", "content_validation"),
        ("operation", skills, "operation:Consume(2)", "operation:Refund(2)", "parse", "ron_parse"),
        ("selector", skills, "owner:SourceEmployee", "owner:SourceAccount", "parse", "ron_parse"),
        ("reference", skills, 'resource:"employee_meter"', 'resource:"missing_meter"', "final_validation", "content_validation"),
        ("owner_mismatch", skills, "owner:SourceEmployee", "owner:SourceEquipment", "final_validation", "content_validation"),
    ]
    cases = []
    for name, path, old, new, phase, code in mutations:
        root = packs / name
        shutil.copytree(original, root / "official")
        file = root / "official" / path
        text = file.read_text()
        assert old in text, (name, old)
        file.write_text(text.replace(old, new, 1))
        selection = {"packs": ["official"], "winners": []}
        (root / "selection.json").write_text(json.dumps(selection))
        cases.append(dict(name=name, selection=selection, phase=phase, code=code,
                          source="official/" + path))
    case_file = output / "cases.json"
    case_file.write_text(json.dumps(cases, indent=2) + "\n")
    before = hashes(packs)
    commands = []

    def run(name, command, expected):
        command = list(map(str, command))
        started = time.perf_counter()
        result = subprocess.run(command, cwd=repo, capture_output=True, timeout=300)
        for suffix, data in (("stdout", result.stdout), ("stderr", result.stderr)):
            (output / f"{name}.{suffix}").write_bytes(data)
        commands.append(dict(name=name, command=command, exit_code=result.returncode,
                             elapsed_seconds=time.perf_counter() - started))
        (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        assert result.returncode == expected, (name, result.returncode, result.stderr.decode(errors="replace"))
        return result

    api, cli = repo / "target/release/examples/invalid", repo / "target/release/author_content"
    run("api", [api, packs, case_file, args.scenario.resolve(), output / "api"], 0)
    for case in cases:
        name = case["name"]
        expected = json.loads((output / "api" / f"{name}.json").read_text())
        for attempt in range(2):
            result = run(f"{name}_{attempt}", [cli, "validate", packs / name, packs / name / "selection.json"], 1)
            assert not result.stdout
            assert json.loads(result.stderr) == expected
        diagnostic = expected["diagnostic"]
        assert diagnostic["phase"] == case["phase"] and diagnostic["code"] == case["code"], (name, diagnostic)
        assert diagnostic["source_file"] == case["source"] and diagnostic["pack_id"] == "official", (name, diagnostic)
        if name in ("reference", "owner_mismatch"):
            assert diagnostic["object_id"] == "paradise_lost_judgement"
            assert diagnostic["field_path"].endswith(".resource" if name == "reference" else ".owner")
        print(json.dumps({"case": name, "passed": True}), flush=True)
    assert hashes(packs) == before and hashes(original) == source_hashes
    result = dict(passed=True, cases=cases, commands=commands, source_files=source_hashes,
                  binary_sha256={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in (api, cli)},
                  files=hashes(output))
    (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"passed": True, "cases": len(cases)}), flush=True)


if __name__ == "__main__":
    main()
