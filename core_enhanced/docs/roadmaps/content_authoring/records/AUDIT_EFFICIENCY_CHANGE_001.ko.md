# 감사 효율 문서 개정 — 적용·인수 안내

2026-09-12 · 문서 개정 `audit-efficiency-1` · GEX gex-r5 / AUTORUN single-r-v3 / 운영 규칙 v1.6

이 파일은 사용자 요청에 따른 절차 문서 개정과 인수 안내다. GEX 감사, 새 정책/Goal/Phase의 확정, 게임 시험
결과 또는 R05 완료 판정이 아니다. 배포본 작성 환경에는 저장소 쓰기 도구가 없었으며 실제 적용 여부·시점·
검사는 적용 담당자가 별도 인수 결과에 기록한다. 파일을 복사한 것만으로 실행 중 agent에 새 지침이 전달되지는 않는다.

## 1. 변경한 범위와 바꾸지 않는 기준

| 문서 | 개정 내용 |
| --- | --- |
| [GEX](../../../goal_execution_pipeline%281%29.md) | 조건별 영향 재감사, 기존 판단/시험의 검증된 재사용, 기계 검사 공유, 현재 요약·고정 증거 구분, reviewer 문맥 재사용, Master 적용 |
| [AUTORUN](../AUTORUN_REQUEST.ko.md) | 위 규칙을 단일 R 실행에 연결. 전체 R 자동 진행·무단 선행 감사·정책 대리 결정 범위를 늘리지 않음 |
| [운영 규칙](../OPERATING_RULES.md) | 최신 result·교정·target을 바로 찾는 요약과 원문 참조. 같은 사실의 여러 원본·반복 서술 방지 |
| [R05 재개 안내](R05_AUDIT_RESUME_GUIDE_001.ko.md) | 이미 고정된 계획/증거를 덮어쓰지 않는 현행 successor 안내와 다음 감사의 구체 입력/검토 범위 |
| [현재 현황](../ROADMAP_STATUS.md) | 문서 개정 안내와 R05 최신 locator 연결. 과거 실행·검증/감사 결과를 소급 변경하지 않음 |

모든 applicable 완료 조건·차원·실제 negative/edge 검증·필수 독립 reviewer·구현/감사 분리·실패 보존·
시작/종료 currentness는 유지한다. 재감사할 조건을 빠뜨리는 축약, 긍정 결론 재사용, 테스트 완화/삭제,
해시 검사 생략, 개인 데이터 수정은 승인하지 않는다. P2G의 통상적 정책 위임/사용자 의논 대기와 단일 R 종료도 유지한다.

P2G 본문/템플릿/검증기, 기존 정식 Goal Set·Freeze·Phase Plan·Handoff·감사 verdict·고정 target은 변경하지 않는다.
새 machine 필드·validator version을 강제하지 않는다. 기존 Goal의 독립성 조건을 없애는 요청도 아니다.

## 2. 적용할 때 반드시 구분할 것

1. 쓰기 작업/감사가 실제로 진행 중인지 확인한다. 진행 중 agent의 지침을 중간에 바꾸지 않고 안전한 경계에서
   적용한다. 이 문서나 수정 배포본이 실제로 프로세스를 pause했다는 뜻은 아니다.
2. 수정 전 GEX/AUTORUN/운영 규칙의 byte identity를 배포 preimage와 비교한다. 새 수정이 있으면 강제 덮어쓰지
   않고 현재본에 관련 변경만 재대조한다. 허용된 수정·새 안내 이외의 dirty와 모든 원본을 보존한다.
3. 이번 변경은 문서 편집으로 기록한다. 실제 적용 경로·전/후 identity·문서 검사·보존 범위를 남기고 종료한다.
   별도 재개 요청 없이 게임 구현·테스트·GEX-20/30·새 reviewer 배정은 시작하지 않는다.
4. 향후 재개를 승인받으면 적용 owner가 old/new 지침·현재 요청·exact 감사 대상을 연결한다. 기존 target에 지침
   파일의 old hash가 들어 있으면 mismatch를 사실대로 기록한다. 동결된 old target에 새 hash를 덮어쓰거나
   governance 파일을 몰래 제외하여 unchanged로 만들지 않는다.
5. 필요한 새 pass/target은 **기존 target + 이번 문서 delta + 그 외 실제 delta + 사용 근거**로 재식별한다.
   runtime/source/input/원래 요구/검증이 불변이면 그 근거를 검증해 재사용한다. 문서 revision 하나 바뀌었다는
   이유로 이미 유효한 게임 시험과 전체 과거 감사를 처음부터 다시 수행하지 않는다. 예상 밖 변경은 따로 판정한다.
6. 새 절차가 기존 필수 Goal 기준을 바꾸지 않는지 적용 담당자가 확인한다. 조건/범위 의미의 실제 변경이 필요하면
   기존 P2G owner로 반환한다. 이 지침 개정 자체로 정식 acceptance를 재작성하거나 전체 프로젝트 완료를 발행하지 않는다.

문서 변경과 대상 전환은 한 번의 공통 인수 근거로 공유하고 각 pass는 그 적용성을 확인한다. 매번 같은 승인/변경
이력을 장문으로 반복하지 않는다. 그러나 raw mismatch, 미실행, 이전 negative finding, 새 source 변경을 숨기지 않는다.

## 3. 재사용 판단의 예

- 동일한 의미·입력·소비자·환경을 유지한 조건: 원문과 근거의 현재성을 확인하고 기존 판단을 exact ref로 연결한다.
- 생성자 panic을 typed 오류로 고침: 개선이지만 오류 계약에 영향을 준 변경이다. 해당 실패·취소·게시 조건을 새로 검토한다.
- private 보조 필드를 제거함: private/serde-skip만으로 무영향을 단정하지 않는다. equality·관찰·size·clone·예산·실제 consumer를 확인한다.
- 이전 전체 감사가 Not Complete: 기존에 검토한 개별 조건은 재사용 가능성을 판단하되 open finding은 새 증거와
  독립 판단으로 닫는다. 부정 verdict나 실패 실행을 과거 PASS로 바꾸지 않는다.
- 새 보고서/현황 링크만 추가함: 실제 수용 의미·입력·evidence에 영향이 있는지 확인한다. 보고서 ID만 바뀌었다고
  모든 선행/후행 감사를 재귀 재실행하지 않는다.

## 4. 이번 개정의 관찰 기준

현재 연결은 simulator, branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414`, dirty=true였다.
GEX r4 1,256줄, AUTORUN single-r-v2 164줄, 운영 v1.5 212줄과 현황 v1.77/기록0077을 읽었다.
R05의 PREDECESSOR_IMPACT_002와 기존 감사/교정/result·target 관련 구간을 대조했다. 이 관찰은 전체 dirty의
원자적 snapshot이나 새로운 게임 실행·독립 감사가 아니다. 적용 시점 currentness는 적용 담당자가 확인한다.
