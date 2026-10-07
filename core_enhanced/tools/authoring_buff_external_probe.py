#!/usr/bin/env python3
"""Verify actual Buff sources and unchanged public gameplay on the resource fixture."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--prototype-pack", type=Path, required=True)
    parser.add_argument("--scenario", type=Path, required=True)
    parser.add_argument("--previous-resource-result", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repository.resolve(strict=True)
    previous = args.previous_resource_result.resolve(strict=True)
    output = args.output_dir.absolute()
    command = [sys.executable, str(Path(__file__).with_name("authoring_resource_external_probe.py")),
               "--repository", str(repo), "--prototype-pack", str(args.prototype_pack),
               "--scenario", str(args.scenario), "--output-dir", str(output)]
    subprocess.run(command, check=True)
    current_fixture = json.loads((output / "FIXTURE.json").read_text())
    prior_fixture = json.loads((previous / "FIXTURE.json").read_text())
    assert current_fixture["files"] == prior_fixture["files"]
    assert (output / "scenario.json").read_bytes() == (previous / "scenario.json").read_bytes()
    buff_site = {"action": {"definition_id": "paradise_lost_judgement", "action_id": "judgement_strike"},
                 "effect_id": "effect_02"}
    damage_site = {"action": buff_site["action"], "effect_id": "effect_01"}
    compared = ["schema_version", "kind", "index", "phase", "input_sha256", "content", "request",
                "resolved_input", "public_result", "assertion", "lifecycle", "initial", "player_profile"]
    summaries = {}
    for generation in ("a", "b"):
        trace = [json.loads(line) for line in
                 (output / f"{generation}_on/terminal_trace.ndjson").read_text().splitlines()]
        buffs = [row for row in trace if row["change"]["channel"] == "buff"]
        damage = [row for row in trace if row["change"]["channel"] == "hp" and row["source"]
                  and row["source"]["authored"]["site"] == damage_site and row["change"]["after"] > 0]
        assert len(buffs) == len(damage) == 5
        for buff, hit in zip(buffs, damage):
            assert buff["source"]["authored"]["site"] == buff_site
            assert buff["source"]["authored"]["field_path"] == "skills[21].steps[0].effects[1]"
            assert buff["source"]["authored"]["source_file"] == "official/data/skills/base.ron"
            for field in ("time_ms", "source_unit", "target_unit"):
                assert buff[field] == hit[field], field
            for field in ("cast_seq", "ability_event_seq", "cause", "proc_origin", "caster_instance_id"):
                assert buff["source"][field] == hit["source"][field], field
            assert buff["change"]["kind"] == "Silence"
            assert buff["change"]["after"] == {"stacks": 1, "expires_at_ms": buff["time_ms"] + 700,
                                               "next_tick_ms": None}
        assert buffs[0]["change"]["before"] is None
        current_rows = [json.loads(line)["payload"] for line in
                        (output / f"{generation}_off/receipts.ndjson").read_text().splitlines()]
        old_rows = [json.loads(line)["payload"] for line in
                    (previous / f"{generation}_off/receipts.ndjson").read_text().splitlines()]
        assert len(current_rows) == len(old_rows) == 15
        steps = 0
        for new, old in zip(current_rows, old_rows):
            for field in compared:
                assert (field in new) == (field in old), field
                if field in new:
                    assert new[field] == old[field], (generation, new.get("index"), field)
            steps += int("public_result" in new)
        assert steps == 13
        summaries[generation] = {"buff_applications": len(buffs), "first_buff": buffs[0],
                                 "last_buff": buffs[-1], "compared_public_steps": steps}
    result = {"passed": True, "resource_result": "RESULT.json", "api_replays": 6, "cli_simulations": 2,
              "cli_explanations": 2, "same_pack_hashes": True, "previous_result": str(previous),
              "previous_result_sha256": hashlib.sha256((previous / "RESULT.json").read_bytes()).hexdigest(),
              "compared_payload_fields": compared, "summaries": summaries}
    (output / "BUFF_RESULT.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result), flush=True)


if __name__ == "__main__":
    main()
