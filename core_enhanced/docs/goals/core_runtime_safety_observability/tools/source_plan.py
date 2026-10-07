#!/usr/bin/env python3
import json
from initialize import HERE, ROOT, artifact, write

GROUPS = [
    ("RQ01", [1, 2, 3], "기존 기본값/ceiling·trusted parser·effective/raw identity",
     ["core_enhanced/src/game/battle/runtime_control.rs", "core_enhanced/src/authoring/control.rs", "core_enhanced/src/game/value_expr/mod.rs", "core_enhanced/src/game/value_expr/parse.rs", "core_enhanced/src/authoring/trial.rs", "core_enhanced/src/game/battle/event_log.rs", "core_enhanced/Cargo.toml", "Cargo.lock"],
     "기존 값·유효 범위·TLS nested scope를 확인하고 RON 단일 기본값/안전 parser/typed profile 경계를 설계한다."),
    ("RQ02", [3, 4, 5, 11], "실제 host→application→첫 spawn·restore·종료",
     ["game_server/src/main.rs", "game_server/src/env.rs", "game_server/src/lib.rs", "game_server/src/game/load_balance_actor/handlers.rs", "game_server/src/game/load_balance_actor/mod.rs", "game_server/src/game/load_balance_actor/messages.rs", "game_server/src/game/player_game_actor/mod.rs", "game_server/src/game/player_game_actor/handlers.rs", "core_enhanced/src/application.rs", "core_enhanced/src/game/world.rs", "core_enhanced/src/game/world/application.rs", "core_enhanced/src/game/world/active_run.rs", "core_enhanced/src/game/world/combat.rs"],
     "実際のbootstrap/caller/command/restore consumerを追跡し、現在APIとの不一致は新規実装済とせず必要adapter deltaとして記録する。"),
    ("RQ03", [3, 5, 6], "authoring load→parse→compile→catalog→trial/API/CLI/preview/explain",
     ["core_enhanced/src/authoring/mod.rs", "core_enhanced/src/authoring/trial.rs", "core_enhanced/src/bin/author_content.rs", "core_enhanced/src/game/data.rs", "core_enhanced/src/game/data/live_ron.rs", "core_enhanced/src/game/value_expr/catalog.rs", "core_enhanced/src/game/value_expr/compile.rs", "core_enhanced/src/game/value_expr/eval.rs"],
     "profile settingsが渡される全段階とcache identity/公開parameterを追跡し、既存合法limits/式意味/無変更queryを保つ。"),
    ("RQ04", [4, 7], "history 생성·deserialize·import·cursor·snapshot",
     ["core_enhanced/src/game/battle/event_log.rs", "core_enhanced/src/game/battle/core/mod.rs", "core_enhanced/src/game/world/state.rs", "core_enhanced/src/game/world/snapshot.rs", "core_enhanced/src/game/world/combat.rs"],
     "全構築経路、serde既定値、保持窓のeviction/floor/resyncと結果統計の別ownerを確認する。"),
    ("RQ05", [8, 10, 11], "bounded operation summary·peak·rollback·commit·cleanup",
     ["core_enhanced/src/game/battle/runtime_control.rs", "core_enhanced/src/game/battle/core/runtime_facts.rs", "core_enhanced/src/game/battle/core/queue.rs", "core_enhanced/src/game/battle/core/sim.rs", "core_enhanced/src/game/world/application.rs", "core_enhanced/src/authoring/control.rs"],
     "現行検査中の費用/extentと公開operation境界を追跡し、exporter全owner走査なしの観測seamとroll-back外の失敗統計を設計する。"),
    ("RQ06", [9, 10, 11], "metrics/exporter依存・host task・Prometheus/Grafana設定・boundedness",
     ["Cargo.toml", "Cargo.lock", "core_enhanced/Cargo.toml", "game_server/Cargo.toml", "game_server/src/main.rs", "game_server/src/env.rs"],
     "実装有無をmanifestから確認し、採用versionの公式source/docでstorage/maintenance/label/cardinalityと実環境起動を検証する。"),
    ("RQ07", [12], "既存U01/R05/R06/R07と製品回帰のcurrent basis",
     ["core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/corrections/f01_action_flow_20261004_001/AUDIT_HANDOFF.md", "core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/corrections/f01_action_flow_20261004_001/target/f01_action_flow_001/MANIFEST.json"],
     "既存frozen evidenceは原本保持し、新規変更のimpactに応じた再実行/再利用を区別する。独立audit確定前のPASS前提は禁止。"),
]


def main():
    source = json.loads((HERE / "P2G_10_POLICY_001.json").read_text())["artifacts"]
    working = next(a for a in source if a["ref"]["Artifact Type"] == "Working Revision")
    rows = []
    for key, elements, claim, paths, validation in GROUPS:
        rows.append({"Requirement ID": key,
            "Exact Policy Element Revision Ref Set": [working["payload"]["Current Policy Element Revision Map"][f"PE-CRSO-{n:02}"] for n in elements],
            "Required claim/boundary": claim,
            "Source type/role": "Workspace Actual; exact frozen local evidence; official dependency evidence for RQ06",
            "Required identity/location constraint": {"repository": str(ROOT), "locations": paths,
                "dirty_basis": "preserve current workspace; hash and archive actually reviewed sources"},
            "Applicability/Necessity": "required", "Applicability condition": None,
            "Policy/Scope Basis Reference": {"Working Revision Ref": working["ref"], "Scope selectors": ["I01", "I02", "I03"]},
            "Validation requirement": validation})
    plan = artifact("Review Scope/Source Requirement Plan", "RSP-CRSO-001", {
        "Bound Input Working Revision Ref": working["ref"], "Requirement Set": rows,
        "Currentness Boundary": "Workspace only; no deployed/remote compatibility claim. Actual local host/bootstrap execution required by GEX.",
        "Candidate Publication": "non-current until complete P2G-20 batch"})
    write("P2G_20_SOURCE_PLAN_001.json", {"artifacts": [plan]})
    print("P2G-20 source plan: 7 required groups, all 12 decided elements covered; review pending")


if __name__ == "__main__":
    main()
