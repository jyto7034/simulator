#!/usr/bin/env python3
"""Create explicit prototype combat A/B packs; never modify official resources."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--official-pack", type=Path, required=True)
    parser.add_argument("--scenario", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    out = args.output_dir.absolute()
    out.mkdir()
    original = args.official_pack.resolve(strict=True)
    for name in ("a", "b"):
        pack = out / name / "official"
        shutil.copytree(original, pack)
        equipment = pack / "data/equipments/base.ron"
        text = equipment.read_text()
        marker = 'id: "standard_training_sword",'
        assert text.count(marker) == 1
        # Shared prototype activation: existing engine vocabulary and authored
        # skill, deterministic proc, no production content-specific code.
        text = text.replace(marker, marker + '''
            ability_activations: [(
                ability_id: "paradise_lost_judgement",
                activation: TriggerProc(trigger: OnAttack, proc_chance_percent: 100, internal_cooldown_ms: 1500),
            )],''')
        equipment.write_text(text)
    path = Path("data/skills/base.ron")
    skill = out / "b/official" / path
    text = skill.read_text()
    marker = 'id: "paradise_lost_judgement",'
    assert text.count(marker) == 1
    start = text.index(marker)
    end = text.index("SkillDef(", start)
    section = text[start:end]
    before = '(id: "effect_01", op: Damage(amount: 18, damage_type: Magic))'
    after = '(id: "effect_01", op: Damage(amount: 36, damage_type: Magic))'
    assert section.count(before) == 1
    skill.write_text(text[:start] + section.replace(before, after) + text[end:])
    differences = []
    files = []
    for a in sorted((out / "a/official").rglob("*")):
        if not a.is_file():
            continue
        relative = a.relative_to(out / "a/official")
        b = out / "b/official" / relative
        ah, bh = hashlib.sha256(a.read_bytes()).hexdigest(), hashlib.sha256(b.read_bytes()).hexdigest()
        if ah != bh:
            differences.append(str(relative))
        files.append({"path": str(relative), "a_sha256": ah, "b_sha256": bh})
    assert differences == [str(path)]
    assert (out / "a/official" / path).read_text() == text
    # Verify the exact selected replacement, rather than a broad regex edit.
    assert skill.read_text() == text[:start] + section.replace(before, after) + text[end:]
    scenario = json.loads(args.scenario.read_text())
    binding = next(binding for binding in scenario["steps"][0]["bindings"] if binding["to"] == "/command/position")
    assert binding["from"]["pointer"].endswith("/cells/0")
    binding["from"]["pointer"] = binding["from"]["pointer"][:-1] + "3"
    (out / "discovery.json").write_text(json.dumps(scenario, separators=(",", ":")))
    (out / "selection.json").write_text('{"packs":["official"],"winners":[]}')
    receipt = {"prototype_only": True, "original": str(original), "shared_change": "training sword OnAttack proc of existing paradise_lost_judgement, 100%, 1500ms",
               "ab_change": {"definition": "paradise_lost_judgement", "action": "judgement_strike", "effect": "effect_01", "field": "amount", "a": 18, "b": 36},
               "different_files": differences, "files": files}
    (out / "FIXTURE.json").write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
