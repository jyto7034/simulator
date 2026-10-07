# 별도 배포 경로 적용 안내

**상태: 미적용. 이 안내 자체는 저장소 변경·구현·배포 승인이 아니다.**

## 1. 배치 대상

원본 로드맵·정식 P2G/GEX 산출물을 보존하기 위해 다음과 같은 **새 연구 폴더**를 권고한다.

```text
core_enhanced/docs/roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/
  README.ko.md
  report/
  evidence/
  roadmap_v2.2-review/
  ROADMAP_STATUS_ADDENDUM.ko.md
  ...
```

같은 경로가 이미 존재하면 내용을 합치거나 덮어쓰지 말고, 기존 목적·내용을 확인한 뒤 충돌하지 않는 새 이름을 정한다. 본 검토는 이 경로가 현재 비어 있다고 확인하지 않았고 실제로 만들지도 않았다. 압축본의 최상위 폴더 안 내용을 새 폴더에 배치하면 문서의 상대 링크가 유지된다.

[create-only diff](CREATE_ONLY.patch)는 이 배포 파일들을 위 새 경로에 추가하는 검토용 패치다. **기존 v2.1 파일을 수정하는 diff가 아니다.** 패치 자신과 해시 manifest는 패치에서 제외했다. 실제 적용 전에 작업 트리와 대상 경로 충돌을 확인해야 하며, 적용·검사 명령을 이번 연결된 저장소에서 실행하지 않았다.

## 2. 보존할 원본

`ROADMAP_STATUS.md` v1.9의 기록 0010과 이전 이력, NEXT-0009, CA00 요구/COV, `WORKING_POLICY.ko.md`, 기존 Review·Decision·Freeze·Phase Plan·Receipt·Goal Set·Handoff·감사, AGENTS/P2G/GEX/검증기, production 코드·RON·저장은 그대로 둔다.

이 배포본은 새 정식 Topic, successor, Change Declaration, Goal activation 또는 실행 기록을 만들지 않는다. 원래 미커밋 변경을 reset/stash/삭제/재포맷하는 정리 단계도 없다.

## 3. 현황에 연결하는 방법

[현황 추가 기록안](ROADMAP_STATUS_ADDENDUM.ko.md)은 **문서 채택 뒤 덧붙일 내용의 제안**이다. 기존 최근 기록의 번호를 미리 소비하지 않는다. 실제 현황 owner가 변경된 최신 기록 번호를 확인하고, 새 연구 위치·적용 여부를 연결한다. NEXT-0009를 지우거나 자동으로 다른 실행 카드로 교체하지 않는다.

정식 최소 Goal의 순서는 기존 `PP-CAMEC-P2G40-002`가 소유한다. 본 연구의 R01은 그것을 유지하는 A 권고다. 후속 R 순서는 별도 로드맵 권고이므로, 아직 존재하지 않는 정식 Phase/Goal의 확정 실행 순서로 표시하지 않는다.

## 4. 후속 정식 절차와 책임

최소 Goal의 실행 여부는 새로운 명시적 실행 요청과 GEX 인수에서 결정한다. 이 패키지를 배치하거나 읽는 것만으로 실행이 승인되지 않는다.

이후 실제 기능을 위한 정식 작업을 선택할 때는 필요한 최소 계약을 각 owner에서 인수한다. 새 gameplay·수명·미정 정책은 P2G-10, 실제 source/consumer 근거·구현 delta는 P2G-20, Phase·동시 전환·검증 owner는 P2G-40, Goal의 표현/누락은 P2G-50의 책임이다. 기존 불변 artifact를 직접 덮어쓰지 않는다. 본 연구는 그 절차를 수행하지 않았다.

개정 `GOALS.json`은 형식이 다른 연구용 객체이므로 기존 배열을 기대하는 프로그램에 그대로 전달하지 않는다. 별도 위치에서 읽거나 후속 문서 owner가 명시적으로 소비자를 조정한다. 기존 정식 검증기를 우회·변조할 근거로 쓰지 않는다.

## 5. 적용 전 내용 식별

읽기 기준은 simulator의 branch `cleanup/simulator-dirty-worktree-20260803`, short HEAD `f318414`, dirty=true였다. 시작과 마지막의 현황 선택은 같았지만, dirty 전체의 원자적 해시/사본은 없다. 각 판단의 영향 source는 [SOURCES](evidence/SOURCES.ko.md)의 범위·심볼로 재대조한다. 관련 부분이 바뀌었으면 그 판단만 갱신하고 과거 값으로 되돌리지 않는다.

로컬 검사:

```bash
python tools/validate_package.py
```

이 명령의 통과는 배포 파일 내부 정합성만 확인한다. 게임 동작, 실제 팩 로딩, 저장 복원, 독립 제작, GEX 자격 또는 서버/Unity 호환성을 보장하지 않는다.
