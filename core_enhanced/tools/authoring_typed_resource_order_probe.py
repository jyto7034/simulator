#!/usr/bin/env python3
"""Public resource pack/container order and later-cost refusal, preserving fixtures."""
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
    for name in ("repository", "fixture", "output-dir"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    repo, source = args.repository.resolve(), args.fixture.resolve()
    output = args.output_dir.absolute()
    output.mkdir()
    before = hashes(source)
    commands = []
    binary = repo / "target/release"

    def run(name, command, expected=0):
        command = list(map(str, command))
        started = time.perf_counter()
        result = subprocess.run(command, cwd=repo, capture_output=True, timeout=300)
        for suffix, data in (("stdout", result.stdout), ("stderr", result.stderr)):
            (output / f"{name}.{suffix}").write_bytes(data)
        commands.append(dict(name=name, command=command, exit_code=result.returncode,
                             elapsed_seconds=time.perf_counter() - started))
        (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
        assert result.returncode == expected, (name, result.stderr.decode(errors="replace"))
        return result

    packs = output / "packs"
    shutil.copytree(source / "packs", packs)
    for name, resource, owner in (("left", "employee_meter", "Employee"), ("right", "copy_meter", "EquipmentCopy")):
        pack = packs / name
        pack.mkdir()
        (pack / "resources.ron").write_text(f'(resources:[(id:"{resource}",owner:{owner},lifetime:Run,initial:7,cap:15)])')
        (pack / "pack.ron").write_text(f'''(pack_id:"{name}",pack_version:"1",namespace:Global,
contract:(pack_format:1,catalog_schema:3,authoring:1,execution:2,fingerprint:1),
required_capabilities:["typed_resources_v1"],dependencies:["official"],
files:[(kind:Resources,path:"resources.ron")],
operations:[(target:(kind:Resource,id:"{resource}"),operation:Replace,after:["official"])])''')
    scenario = source / "scenario.json"
    previous = None
    previous_receipts = None
    for index, order in enumerate((["official", "left", "right"], ["right", "official", "left"])):
        selection = output / f"selection_{index}.json"
        selection.write_text(json.dumps(dict(packs=order, winners=[])))
        native_file = output / f"native_{index}.json"
        run(f"native_{index}", [binary / "examples/typed_resources", packs, selection, scenario, native_file])
        native = json.loads(native_file.read_text())
        assert native["failure"] is None and len(native["final"]["run"]["entries"]) == 2
        if previous is not None:
            assert native == previous
        previous = native
        run(f"trial_{index}", [binary / "examples/combat", packs, selection, scenario, output / f"trial_{index}", "replay", "off"])
        receipts = (output / f"trial_{index}/receipts.ndjson").read_bytes()
        assert run(f"cli_{index}", [binary / "author_content", "simulate", packs, selection, scenario]).stdout == receipts
        if previous_receipts is not None:
            assert receipts == previous_receipts
        previous_receipts = receipts

    # Declaration insertion order changes source bytes, not owner/gameplay state.
    reversed_packs = output / "reversed"
    shutil.copytree(source / "packs", reversed_packs)
    (reversed_packs / "official/data/resources/base.ron").write_text(
        '(resources:[(id:"copy_meter",owner:EquipmentCopy,lifetime:Run,initial:7,cap:15),'
        '(id:"employee_meter",owner:Employee,lifetime:Run,initial:7,cap:15)])')
    run("native_original", [binary / "examples/typed_resources", source / "packs", source / "selection.json", scenario, output / "native_original.json"])
    run("native_reversed", [binary / "examples/typed_resources", reversed_packs, source / "selection.json", scenario, output / "native_reversed.json"])
    original, reversed_value = [json.loads((output / name).read_text()) for name in ("native_original.json", "native_reversed.json")]
    assert original["failure"] is None and reversed_value["failure"] is None
    # The resolver deliberately includes declaration slots in content identity.
    # Reordering declarations is a new content input; gameplay must still agree.
    assert original["identity"]["contract"] == reversed_value["identity"]["contract"]
    assert original["identity"]["semantic_sha256"] != reversed_value["identity"]["semantic_sha256"]
    for key in ("final", "outputs", "terminal_facts"):
        assert original[key] == reversed_value[key], key

    shortage = output / "shortage"
    shutil.copytree(source / "packs", shortage)
    skill = shortage / "official/data/skills/base.ron"
    text = skill.read_text()
    old = 'resource:"copy_meter",owner:SourceEquipment,operation:Consume(2)'
    assert text.count(old) == 1
    skill.write_text(text.replace(old, old.replace("Consume(2)", "Consume(99)")))
    run("shortage_api", [binary / "examples/typed_refusal", shortage, source / "selection.json", scenario, output / "shortage_api.json"])
    refused = json.loads((output / "shortage_api.json").read_text())
    assert refused["passed"]
    cli = run("shortage_cli", [binary / "author_content", "simulate", shortage, source / "selection.json", scenario], 1)
    assert json.loads(cli.stderr) == refused["failures"][0]
    assert cli.stdout == "".join(row + "\n" for row in refused["accepted_before"]["receipts"]).encode()
    run("shortage_native", [binary / "examples/typed_resources", shortage, source / "selection.json", scenario, output / "shortage_native.json"])
    native = json.loads((output / "shortage_native.json").read_text())
    assert native["failure"]["code"] == "resource_insufficient"
    assert native["failure"]["before"] == native["failure"]["after"]
    assert hashes(source) == before
    result = dict(passed=True, pack_orders=2, reversed_declaration=True, atomic_shortage=True,
                  source_files=before, commands=commands,
                  binaries={name: hashlib.sha256((binary / name).read_bytes()).hexdigest()
                            for name in ("author_content", "examples/typed_resources", "examples/typed_refusal", "examples/combat")},
                  files=hashes(output))
    (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"passed": True, "pack_orders": 2, "reversed_declaration": True, "atomic_shortage": True}), flush=True)


if __name__ == "__main__":
    main()
