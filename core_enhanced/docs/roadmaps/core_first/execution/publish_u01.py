"""Record the post-audit status-only successor; preserve audited inputs."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tarfile

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent / "u01_closure_20261004_001"
B = ROOT / "core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/corrections/f01_action_flow_20261004_001"
A = ROOT / "core_enhanced/docs/goals/content_authoring_r08_r12_runtime/post_audit/20261004_f01_001"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    assert (A / "AUDIT_CLOSURE.json").is_file()
    assert "Complete / High" in (A / "COMPLETION_AUDIT.md").read_text()
    HERE.mkdir(exist_ok=False)
    spec = importlib.util.spec_from_file_location("u01_runner", B / "run.py")
    runner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(runner)
    before = runner.identity()
    status = ROOT / "core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md"
    roadmap = ROOT / "core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md"
    paths = [status, roadmap]
    hashes_before = {str(p.relative_to(ROOT)): sha(p) for p in paths}
    with tarfile.open(HERE / "DOCUMENTS_BEFORE.tar.gz", "w:gz") as tar:
        for p in paths:
            tar.add(p, arcname=str(p.relative_to(ROOT)))
    text = status.read_text()
    start = text.index("2026-10-04 · v1.126")
    end = text.index("아래 R 기록은", start)
    text = text[:start] + """2026-10-04 · v1.127 · 현재 개발 주작업: **U00 인수·U01 F01 교정 및 독립 감사 완료 → U02 정책·코드 검토**.
[활성 로드맵](../core_first/ROADMAP.ko.md), [연속 실행 위임](../core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md),
[U01 독립 감사 Complete / High](../../goals/content_authoring_r08_r12_runtime/post_audit/20261004_f01_001/COMPLETION_AUDIT.md),
[감사 종료 후 현황 기록](../core_first/execution/u01_closure_20261004_001/RESULT.ko.md),
[U02 작업 문서](../../goals/core_runtime_safety_observability/WORKING_POLICY.ko.md)를 따른다.
U01의 새 target은 F01·55조건 및 R06/R07 현재 통합 적용성을 통과했다. 과거 Not Complete 원본은 보존한다.
U02는 P2G-00/10 완료, P2G-20 진행 중이며 아직 제품 구현 완료가 아니다. R13~R16 요구는 새 owner U07–10 등에 인수한다.

""" + text[end:]
    text = text.replace("## 현재 작업\n\n", """## 현재 작업

U01 target `f01_action_flow_001`은 독립 감사 Complete / High다. 공개 취소 oracle 5/5와
default/test-fixtures 전체 각각1684 PASS를 확인했다. R05 raw exit101의 기존 승인 concurrent peak 경고
343,587 bytes 초과와 OBS001은 유지한다. 20분 시험은 제외한다.
다음 범위 U02의 trusted profile·bounded 관측에 필요한 코드 검토와 정식 Goal 인계를 진행한다.
실제 server baseline의 공개 API 불일치는 U02에서 필요한 host 연결 범위로 기록했다.

### 이전 F01 감사 상태 — 2026-10-03 target 이력

""", 1)
    status.write_text(text)
    text = roadmap.read_text()
    text = text.replace("**상태: 2026-10-04 연속 실행 요청 인수 / U00 인수·U01 F01 교정 착수 / 제품 완료·감사 통과는 별도 근거**",
        "**상태: 2026-10-04 U00 인수·U01 F01 교정 및 독립 감사 Complete / High / U02 P2G-20 코드 검토 진행**", 1)
    needle = "아래 v4 작성 당시의 ‘제안·실행 승인 아님’ 표기는 당시 문서 작성의 증거 수준이며, 최신 명시적 실행 요청을 취소하지 않는다."
    assert needle in text
    text = text.replace(needle, needle + "\n[U01 완료·검증·남은 경고](execution/u01_closure_20261004_001/RESULT.ko.md)와\n[U02 정책 작업](../../goals/core_runtime_safety_observability/WORKING_POLICY.ko.md)에 실제 진행을 기록한다.", 1)
    roadmap.write_text(text)
    after = runner.identity()
    (HERE / "PUBLICATION_RECORD.json").write_text(json.dumps({
        "kind": "post-audit status-only successor", "audit": str(A.relative_to(ROOT)),
        "audit_closure_sha256": sha(A / "AUDIT_CLOSURE.json"),
        "audit_report_sha256": sha(A / "COMPLETION_AUDIT.md"),
        "target_manifest_sha256": sha(B / "target/f01_action_flow_001/MANIFEST.json"),
        "documents_before": hashes_before,
        "documents_after": {str(p.relative_to(ROOT)): sha(p) for p in paths},
        "source_before": before, "source_after": after,
        "impact": "Only ROADMAP_STATUS is inside target source identity; active ROADMAP is a reference document. Status/link updates do not change product behavior, policy scope, frozen target bytes or audit verdict.",
        "next": "core_runtime_safety_observability WR-CRSO-002 / P2G-20 source review; no U02 product mutation yet",
    }, ensure_ascii=False, indent=2) + "\n")
    (HERE / "RESULT.ko.md").write_text("""# U00/U01 완료 인수와 다음 작업

2026-10-04 · U01 독립 판정 **Complete / High**. [독립 감사](../../../../goals/content_authoring_r08_r12_runtime/post_audit/20261004_f01_001/COMPLETION_AUDIT.md)와
[종료 currentness](../../../../goals/content_authoring_r08_r12_runtime/post_audit/20261004_f01_001/AUDIT_CLOSURE.json)가 판정 owner다.

U00 문서·현재성 인수를 완료했다. U01은 정확한 action의 미방출 Flow와 예약 제어 이벤트를 취소하고
기방출 projectile/area 및 다른 action을 보존한다. 신규7개 회귀·6개 fault rollback 지점, 공개5개 oracle,
기존55조건과 R06/R07 통합 적용성을 확인했다. default/test-fixtures 각각1684 PASS,
release CLI9팩45명령·trace parity·PC36 비교 및 관련 format/check/clippy가 통과했다.

R05 raw exit101은 승인된 concurrent peak 단독 경고(9,373,775 대9,030,188 bytes)로 유지하며
OBS001 회복 preview 배열 순서 관찰도 보존한다. 20분 시험은 수행하지 않았고 후속 필수 조건에도 넣지 않는다.
과거 감사·동결 target·시험 실패/중단 원본을 수정하지 않았다.

감사 종료 후 현황 view 두 파일만 갱신했다. [변경 전 archive](DOCUMENTS_BEFORE.tar.gz)와
[적용성·전후 identity 기록](PUBLICATION_RECORD.json)을 남겼다. 이후 현황 bytes가 감사 종료 source와
다르다는 사실을 숨기지 않으며, 코드/데이터가 같다는 영향 근거로 이 문서 successor를 인수한다.

다음은 [U02 실행 한도·관측](../../../../goals/core_runtime_safety_observability/WORKING_POLICY.ko.md)이다.
P2G-00/10을 완료했고 P2G-20에서 실제 host/authoring/compile/runtime 경계를 조사한다.
서버 baseline API 연결 불일치는 원자료로 보존했으며 필요한 공개 adapter 교정을 포함해 정식 계획을 작성한다.
U02 제품 구현·검증·독립 감사는 아직 미완료다. 원 연속 요청에 따라 재승인 없이 이어간다.
""")
    print("U01 Complete / High published; 2 status documents updated; U02 P2G-20 active")


if __name__ == "__main__":
    main()
