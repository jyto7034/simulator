#!/usr/bin/env python3
"""Static inventory only. Never builds Rust, launches tests, or changes run settings."""
import argparse
import collections
import csv
import fnmatch
import hashlib
import io
import json
from pathlib import Path
import re
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
ROOTS = [f"{crate}/{kind}" for crate in ("core_enhanced", "game_server")
         for kind in ("src", "tests", "examples", "benches")]
TEST = re.compile(r"#\[\s*(?:test|(?:tokio|actix_rt|actix_web)::test)\s*(?:\([^\]]*\))?\]")
FUNCTION = re.compile(r"\b(?:async\s+)?fn\s+(\w+)\s*\(")
FIELDS = ["source", "line", "test_name", "package", "target_hint", "feature_hints",
          "lane", "resource_groups", "tags", "rule_ids", "source_sha256", "parallel_validated"]


def validate_policy(policy):
    ids = [r["id"] for r in policy["rules"]]
    if len(ids) != len(set(ids)):
        raise ValueError("Duplicate rule IDs")
    for rule in policy["rules"] + policy.get("non_harness_jobs", []):
        if rule["lane"] not in policy["lane_priority"]:
            raise ValueError(f"Unknown lane: {rule}")
        if set(rule.get("resources", [])) - policy["resources"].keys():
            raise ValueError(f"Unknown resource: {rule}")
        if not rule.get("reason", "").strip():
            raise ValueError(f"Missing classification reason: {rule}")
    for rule in policy["rules"]:
        if not rule.get("paths"):
            raise ValueError(f"Empty rule paths: {rule['id']}")


def execution_constraints(row, policy):
    """Keep orthogonal constraints even when the display lane has higher priority."""
    rules = {r["id"]: r for r in policy["rules"]}
    lanes = {row["lane"]}
    for key in row["rule_ids"].split(";"):
        if key != "P_DEFAULT":
            lanes.add(rules[key]["lane"])
    return {
        "fresh_process": "isolated_process" in lanes,
        "quiet_host": "quiet_host" in lanes,
        "resources": sorted(filter(None, row["resource_groups"].split(";"))),
        "parallel_validated": row["parallel_validated"] == "true",
    }


def declaration_changes(previous, current):
    # Lines can move without adding/removing a declaration. Preserve multiplicity.
    def counts(rows):
        return collections.Counter((r["source"], r["test_name"]) for r in rows)
    old, new = counts(previous), counts(current)
    def expand(values):
        return [{"source": source, "test_name": name, "count": count}
                for (source, name), count in sorted(values.items())]
    return {"added_declarations": expand(new - old),
            "removed_declarations": expand(old - new)}


def require_review(changed, reviewed, policy_changed, reviewed_policy):
    missing = set(changed) - set(reviewed)
    extra = set(reviewed) - set(changed)
    if missing or extra or (policy_changed and not reviewed_policy):
        raise ValueError("Snapshot update requires exact reviewed sources and policy acknowledgement: "
                         + json.dumps({"unreviewed_sources": sorted(missing),
                                       "unexpected_reviewed_sources": sorted(extra),
                                       "unreviewed_policy": policy_changed and not reviewed_policy}))


def render_table(rows, policy):
    def cell(value):
        return str(value).replace("|", "\\|").replace("\n", " ")
    lines = ["# 테스트 분류 기준과 등록표", "",
             "자동 생성: `policy.json` + 검토된 `inventory.csv`. 직접 수정하지 않는다.", "",
             "현재 작업 트리의 승인이나 병렬 실행 통과를 뜻하지 않는다. 실행 전 `check_inventory.py`가 통과해야 한다.", "",
             "## 분류 기준", "", "| 표시 분류 | 의미 |", "| --- | --- |"]
    for lane in policy["lane_priority"]:
        lines.append(f"| `{lane}` | {cell(policy['defaults'][lane])} |")
    lines += ["", "표시 분류와 별개로 새 프로세스·호스트 독점·모든 자원 제약을 함께 적용한다.",
              "설정·플랫폼·완전한 시험 이름은 구성별 하네스 대조에서 확정한다. 빌드/fixture 선행 조건은 실행 묶음이 소유한다.",
              "", "## 공유 자원", "", "| 자원 키 | 적용 범위 | 근거 |", "| --- | --- | --- |"]
    for key, value in policy["resources"].items():
        lines.append(f"| `{key}` | {cell(value['scope'])} | {cell(value['reason'])} |")
    lines += ["", "## 등록표 — 소스와 조건별 묶음", "",
              "개별 선언은 `inventory.csv`에서 확인한다. 새 테스트는 규칙을 상속하되 소스 변경 검토 전에는 실행 승격되지 않는다.",
              "", "| 소스 | 수 | 표시 분류 | 새 프로세스 | 조용한 호스트 | 자원 키 |", "| --- | ---: | --- | --- | --- | --- |"]
    groups = collections.Counter()
    for row in rows:
        c = execution_constraints(row, policy)
        groups[(row['source'], row['lane'], c['fresh_process'], c['quiet_host'],
                "; ".join(c['resources']))] += 1
    for (source, lane, fresh, quiet, resources), count in sorted(groups.items()):
        lines.append(f"| `{source}` | {count} | {lane} | {fresh} | {quiet} | {resources or '—'} |")
    lines += ["", "## 하네스 밖의 등록된 측정 작업", "",
              "| 소스 | 분류 | 자원 | 근거 |", "| --- | --- | --- | --- |"]
    for job in policy.get('non_harness_jobs', []):
        lines.append(f"| `{job['source']}` | {job['lane']} | {'; '.join(job['resources'])} | {cell(job['reason'])} |")
    return "\n".join(lines) + "\n"


def mask_non_code(text):
    """Keep offsets/newlines while hiding comments and Rust string/char literals.

    This is a declaration scanner, not Rust macro expansion or cfg evaluation.
    """
    result = list(text)
    i = 0
    while i < len(text):
        end = None
        if text.startswith("//", i):
            end = text.find("\n", i)
            if end < 0:
                end = len(text)
        elif text.startswith("/*", i):
            depth, end = 1, i + 2
            while end < len(text) and depth:
                if text.startswith("/*", end):
                    depth, end = depth + 1, end + 2
                elif text.startswith("*/", end):
                    depth, end = depth - 1, end + 2
                else:
                    end += 1
        else:
            raw = re.match(r'(?:br|cr|r)(#*)"', text[i:i + 40])
            if raw:
                close = '"' + raw[1]
                found = text.find(close, i + raw.end())
                end = len(text) if found < 0 else found + len(close)
            elif text[i] == '"':
                end = i + 1
                while end < len(text):
                    if text[end] == "\\":
                        end += 2
                    elif text[end] == '"':
                        end += 1
                        break
                    else:
                        end += 1
            elif text[i] == "'":
                char = re.match(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'", text[i:])
                if char:
                    end = i + char.end()
        if end is None:
            i += 1
        else:
            for j in range(i, min(end, len(text))):
                if text[j] != "\n":
                    result[j] = " "
            i = end
    return "".join(result)


def source_paths():
    paths = {p for root in ROOTS for p in (ROOT / root).rglob("*.rs")}
    paths.update((ROOT / ".cargo").glob("*.toml"))
    paths.update(p for p in [ROOT / "Cargo.toml", ROOT / "Cargo.lock",
                            ROOT / "core_enhanced/Cargo.toml", ROOT / "game_server/Cargo.toml",
                            ROOT / "core_enhanced/build.rs", ROOT / "game_server/build.rs"] if p.exists())
    return sorted(paths)


def inventory(policy):
    rows, hashes, used = [], {}, collections.Counter()
    for path in source_paths():
        source = path.relative_to(ROOT).as_posix()
        data = path.read_bytes()
        hashes[source] = hashlib.sha256(data).hexdigest()
        if path.suffix != ".rs":
            continue
        text = data.decode("utf-8")
        masked = mask_non_code(text)
        for attr in TEST.finditer(masked):
            function = FUNCTION.search(masked, attr.end())
            if function is None or function.start() - attr.end() > 4096:
                raise ValueError(f"Unresolved test declaration: {source}:{attr.start()}")
            between = masked[attr.end():function.start()]
            if TEST.search(between) or ";" in between:
                raise ValueError(f"Ambiguous test declaration: {source}:{attr.start()}")
            name = function[1]
            matches = []
            for rule in policy["rules"]:
                if (any(fnmatch.fnmatchcase(source, pattern) for pattern in rule["paths"])
                    and fnmatch.fnmatchcase(name, rule.get("test", "*"))
                    and ("contains" not in rule or re.search(rule["contains"], masked))):
                    matches.append(rule)
                    used[rule["id"]] += 1
            lane = max((r["lane"] for r in matches), key=policy["lane_priority"].get,
                       default="parallel_candidate")
            resources = sorted({s for r in matches for s in r.get("resources", [])})
            tags = sorted({s for r in matches for s in r.get("tags", [])})
            if "#[ignore" in text[attr.end():function.start()]:
                tags.append("ignored")
            features = sorted(set(re.findall(r'feature\s*=\s*"([\w-]+)"', text)))
            if "/runtime_observation" in source or "/observability/" in source:
                features.append("runtime-observation")
            if "/runtime_measurement/" in source:
                features.append("runtime-measurement")
            parts = Path(source).parts
            target = "lib"
            if parts[1] == "tests":
                target = "test:" + ("skill_test_suite" if parts[2] == "skill_test" else Path(parts[2]).stem)
            elif parts[1] != "src":
                target = "needs-harness-mapping"
            rows.append(dict(zip(FIELDS, [source, text.count("\n", 0, function.start()) + 1,
                name, "game_core" if parts[0] == "core_enhanced" else "game_server", target,
                ";".join(sorted(set(features))), lane, ";".join(resources), ";".join(sorted(set(tags))),
                ";".join(r["id"] for r in matches) or "P_DEFAULT", hashes[source], "false"])))
    ids = [(r["source"], r["line"], r["test_name"]) for r in rows]
    if len(ids) != len(set(ids)):
        raise ValueError("Duplicate source declarations")
    unused = [r["id"] for r in policy["rules"] if not used[r["id"]]]
    if unused:
        raise ValueError(f"Rules without matches: {unused}")
    return rows, hashes


def encode_csv(rows):
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=FIELDS, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="Explicitly replace this directory's static snapshot only")
    parser.add_argument("--render-table", action="store_true", help="Render the reviewed inventory without accepting source changes")
    parser.add_argument("--reviewed-source", action="append", default=[], help="Reviewed changed source path, repeat per changed file")
    parser.add_argument("--reviewed-policy", action="store_true", help="Acknowledge review of changed classification rules")
    args = parser.parse_args()
    if args.render_table and args.write:
        parser.error("--render-table and --write are mutually exclusive")
    if not args.write and (args.reviewed_source or args.reviewed_policy):
        parser.error("Review acknowledgements require --write")
    policy_bytes = (HERE / "policy.json").read_bytes()
    policy = json.loads(policy_bytes)
    validate_policy(policy)
    stored = json.loads((HERE / "source_snapshot.json").read_text())
    previous = list(csv.DictReader(io.StringIO((HERE / "inventory.csv").read_text())))
    policy_changed = stored['policy_sha256'] != hashlib.sha256(policy_bytes).hexdigest()
    table_path = HERE / "CLASSIFICATION.ko.md"
    if args.render_table:
        if policy_changed:
            raise ValueError("Review and update the policy snapshot before rendering")
        table_path.write_text(render_table(previous, policy), encoding="utf-8")
        print(json.dumps({"table_written": True, "source_snapshot_updated": False,
                          "declarations": len(previous), "rust_builds_or_tests_started": 0}))
        return 0
    rows, hashes = inventory(policy)
    counts = dict(sorted(collections.Counter(r["lane"] for r in rows).items()))
    snapshot = {"schema_version": 1, "status": "static_classification_only",
                "parallel_validated": False, "source_declarations": len(rows),
                "counts": counts, "test_source_files": len({r["source"] for r in rows}),
                "policy_sha256": hashlib.sha256(policy_bytes).hexdigest(), "sources": hashes}
    csv_text = encode_csv(rows)
    changed = sorted(k for k in hashes.keys() | stored["sources"].keys()
                     if hashes.get(k) != stored["sources"].get(k))
    if args.write:
        require_review(changed, args.reviewed_source, policy_changed, args.reviewed_policy)
        # Reject a moving input before replacing any reviewed registration files.
        end_hashes = {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                      for p in source_paths()}
        if end_hashes != hashes or (HERE / 'policy.json').read_bytes() != policy_bytes:
            raise ValueError("Inputs changed during inventory; retry at a stable editing boundary")
        (HERE / "inventory.csv").write_text(csv_text, encoding="utf-8")
        (HERE / "source_snapshot.json").write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
        table_path.write_text(render_table(rows, policy), encoding="utf-8")
        print(json.dumps({"snapshot_written": True, "declarations": len(rows), "counts": counts}))
        return 0
    same_csv = (HERE / "inventory.csv").read_text() == csv_text
    same_table = (not policy_changed and table_path.exists()
                  and table_path.read_text() == render_table(previous, policy))
    ok = stored == snapshot and same_csv and same_table
    print(json.dumps({"ok": ok, "declarations": len(rows), "counts": counts,
                      "changed_sources": changed, "inventory_matches": same_csv,
                      **declaration_changes(previous, rows),
                      "policy_matches": not policy_changed, "table_matches": same_table,
                      "source_changes_require_review": bool(changed),
                      "rust_builds_or_tests_started": 0}, indent=2))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
