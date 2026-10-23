# Goal Execution Pipeline

Status: `ready`

Document Revision: `2026-09-13-gex-r6`

이 개정은 기존 GEX-00/10/20/30과 정책·구현·감사 권한을 유지하면서 교정 재진입, 재개, 문서 역할 매핑,
상세 설계와 검증 근거 재사용을 명확히 한다. `ready`는 지침의 사용 상태이며 실제 Goal 실행·검증기·게임
테스트의 통과를 뜻하지 않는다. 기존 Goal·Handoff·감사 verdict를 새 버전의 결과로 소급 변경하지 않는다.

gex-r4는 2026-09-11 사용자 합의의 통상적 정책 판단·위임 기준을 P2G §8.6에서 인수한다. 정책 owner로의
반환과 사용자 답변 대기를 구분하며, 기존 질문 전달·대기·실행 중인 독립 작업 조건은 유지한다.
Goal·검증기·완료 기준은 이번 문서 편집으로 변경하지 않는다.

**gex-r5 변경:** 완료 조건 전체의 추적은 유지하면서 영향 기반 재감사, 조건별 근거 재사용, 공통 기계 검사와
현재 요약의 역할을 명확히 한다. 기존 GEX 단계·verdict·필수 독립성·정책 위임/대기·실행 권한은 유지한다.
읽기 절차의 소유자는 §5, 시험/기계 증거 재사용은 §6.4, reviewer 운용은 §6.7, 실제 판정은 GEX-20/30이다.
이미 고정된 대상에는 지침 파일 교체만으로 적용하지 않는다. 적용 owner가 구·신 revision, exact 대상·권한,
바뀐 검토 절차와 보존할 요구·증거를 기록한 후 새 pass에서 사용한다. 과거 target/verdict는 수정하지 않는다.

**gex-r6 변경:** 기본 구현 요청은 전체 구현·자체 검증 후 감사 인계에서 종료한다. 명시적으로 요청한 경우에만
빌드 가능한 중간 checkpoint의 단계검토를 추가한다. 적용 owner는 새 실행 PLAN에 구·신 revision과 이 변경의
영향을 기록한다. 기존 동결 Goal·P2G artifact·audit target/verdict는 편집하거나 새 revision으로 소급 승인하지 않는다.

Scope: P2G-60 Handoff 이후, 사용자가 선택한 Single/Phase Goal의 구현·검증·완료 감사와 multi-phase Goal Set의
Master 최종 통합 감사까지

Upstream contract: [Policy-To-Goal Pipeline](./policy_to_goal_pipeline_revised_v2.md)

위 링크는 현재 저장소 배치 경로다. 실제 accepted Handoff가 가리키는 P2G/Goal의 exact basis와 적용 지침 revision을
확인한다. P2G r2의 설계 인계·검증 재사용은 해당 Goal에 적용되는 내용만 소비하며, GEX 파일 교체만으로
P2G 본문·Goal Quality Contract·검증기·상위 AGENTS/runner의 규칙까지 바뀌었다고 가정하지 않는다.

## Reader Contract

이 문서를 사용하는 AI는 먼저 요청을 다음 중 하나로 구분한다.

- `execute`: Goal 실행 파이프라인의 한 단계 또는 terminal result까지의 분리된 단계 sequence를 수행한다.
- `inspect`: Goal, 실행 기록, 코드 또는 감사 결과를 읽고 설명한다.
- `status`: 현재 실행 단계와 충족·미충족 조건을 보고한다.
- `pipeline-audit`: 이 파이프라인 문서나 실행 계약의 논리를 감사한다.
- `document-edit`: 사용자가 지정한 문서를 수정한다.

`execute`일 때만 GEX 단계 routing을 수행한다. `execute` 요청은 다음 두 모드로 구분한다.

- `stage-run`: 사용자가 지정한 GEX 단계 하나의 직접 결과만 수행한다.
- `goal-run`: 사용자가 지정한 Goal 또는 Goal Set을 이 절에서 정한 terminal result까지 수행한다.

`GEX-20을 설명해줘`, `GEX-30 계약을 검토해줘` 같은 요청의 GEX ID는 조회 대상을 뜻할 뿐 실행 승인이 아니다.
반대로 `Phase 완료 감사를 수행해줘`처럼 고유 결과를 직접 요청하면 GEX ID가 없어도 `stage-run` 요청으로 볼 수
있다. 다음과 같이 exact Goal을 실행·완료 대상으로 지정한 요청은 별도 제한이 없으면 `goal-run`으로 해석한다.

- `<Goal 문서>를 수행해줘`
- `<Goal ID>를 구현하고 완료해줘`
- `이 Goal을 끝까지 진행해줘`

GEX 단계 ID 하나를 명시적으로 요청하면 기본적으로 `stage-run`이다. `GEX-00부터 Goal 완료까지 진행해줘`처럼
전체 실행 의도가 명시되면 `goal-run`이다. 아래는 감사까지 승인된 경우의 전체 terminal result 범위다.
일반 구현 요청에는 다음 절의 기본 감사 인계 checkpoint를 적용하므로 이 표만으로 감사가 승인되지 않는다.

| 요청 대상 | `goal-run` 범위와 terminal result |
| --- | --- |
| Single Goal | `GEX-00 -> GEX-10 -> GEX-20`; `Complete / High` 또는 중단 조건 |
| Phase Goal member | 해당 Phase의 `GEX-00 -> GEX-10 -> GEX-20`; `Complete / High` 또는 중단 조건 |
| multi-phase Goal Set 또는 Master Goal | 모든 Phase의 `GEX-00 -> GEX-10 -> GEX-20`과 마지막 `GEX-30`; Master `Complete / High` 또는 중단 조건 |

### 기본 구현 인계와 선택적 단계검토

이 규칙은 모든 Goal에 적용하며 특정 로드맵·모델·파일명에 한정하지 않는다. 기존 `stage-run | goal-run`과
GEX 단계는 유지한다. 아래 checkpoint는 사용자에게 제어권을 돌려주는 실행 경계이지 새 formal state/승인 artifact가 아니다.

- **기본값:** 선택된 Single/Phase Goal의 내부 구현 단계와 자체 검증·허용된 기술 교정을 중간 검토 대기 없이
  수행한다. GEX-10 exit를 충족하고 구현 후 Audit Target Record를 고정하면 `구현·자체 검증 종료 / 감사 요청 대기`를
  보고하고 최종 응답으로 종료한다. 새 감사 요청 전에는 GEX-20/30을 시작하지 않는다. 구현 종료는 Goal `Complete`가 아니다.
- **명시적 opt-in:** 사용자가 “단계검토 과정을 포함해줘”, “빌드 가능한 단계마다 멈추고 검토받게 해줘”처럼
  중간 검토 중단을 실제로 지시하면 §6.4의 단계검토 checkpoint를 추가한다. 설명·인용·예시·질문 속 단순 언급,
  저비용/고성능 모델 사용 계획, 위험도 또는 문서에 옵션이 존재한다는 사실만으로 활성화하지 않는다.
- PLAN에 선택 여부·근거 요청·적용 Goal/범위와 종료점을 기록한다. 명시적으로 선택한 옵션은 같은 Goal의
  재개·교정에도 유지하되 새 Goal에 자동 전파하지 않는다. 뒤늦게 요청해도 이미 끝난 구현을 재실행하지 않고
  현재 baseline에서 남은 작업의 checkpoint만 정한다. 사용자가 옵션을 변경하면 근거와 남은 경계를 갱신한다.
- 기본 최종 감사 인계 경계는 “이 Goal을 **감사까지** 끝까지 수행해줘”처럼 감사까지 포함한 명시적 위임으로
  해제할 수 있다. 단순 “구현하고 완료해줘/끝까지”만으로는 해제하지 않는다. 별도로 켠 중간 단계검토 경계는
  감사까지의 위임만으로 해제되지 않는다. 명시적 `stage-run`과 더 좁은 사용자 제한도 유지한다.
- checkpoint에 도달하면 mutation·후속 구현·자동 감사·새 위임을 멈추고 최종 응답으로 반환한다. 고정 범위를
  쓰는 진행 중 작업도 안전하게 정리한 뒤 고정한다. 모델 변경·자동 재개·시간 경과·검토 PASS는 진행 승인이 아니다.
  사용자는 검토, 필요한 교정, 다음 구현을 각각 명시적으로 요청한다. 검토만 요청받은 AI는 코드를 수정하지 않으며
  PASS여도 다음 구현을 시작하지 않는다. 검토에서 수정 요구가 나오면 승인된 교정·재검토 후 다음 구현 요청을 기다린다.
- multi-phase Set의 기본 흐름은 다음 합법적인 Phase 구현 인계에서 멈춘다. Set 전체를 먼저 구현하려고 필수
  선행 Phase 감사를 생략하지 않는다. checkpoint는 Atomic Group·Phase prerequisite·정책·격리·독립성 조건을 완화하지 않는다.

이 절의 중단 경계는 아래 재개표·자동 correction/re-audit·독립 Phase 진행보다 우선한다. 감사 권한이 없으면
필요한 exact audit와 다음 요청을 보고하고 멈춘다. 모델은 사용자가 선택하며 문서/AI가 실제 모델 전환을 했다고 주장하지 않는다.
검토 인계 checkpoint는 예정된 요청 종료점이지 기술적 `Blocked`나 정식 Goal `Complete` 판정이 아니다.
자동 실행 runner를 쓰면 인수 시 이번 요청의 종료점을 해당 checkpoint로 연결하고 지원하는 정상 종료 절차를 따른다.
문서 편집만으로 기존 시스템 Goal·runner가 바뀌었다고 가정하지 않으며 상위 실행 계약과 충돌하면 그 경계에서 보고한다.

### execute mode routing

| 단계 | 요청한 직접 결과 |
| --- | --- |
| `GEX-00` | P2G Handoff 인수, 실행할 Single/Phase Goal 선택, 현재 기준선과 실행 문서 초기화 |
| `GEX-10` | 선택된 Single/Phase Goal의 코드·데이터·문서 구현과 검증 |
| `GEX-20` | 구현이 끝난 Single/Phase Goal의 독립 완료 감사 |
| `GEX-30` | 모든 Phase가 완료된 multi-phase Goal Set의 Master 최종 통합 감사 |

다음 규칙을 항상 적용한다.

1. 하나의 GEX pass에서는 GEX 단계 하나만 수행한다. `goal-run`도 여러 단계를 하나의 pass로 합치지 않고
   단계별 진입 조건, 권한, 산출물과 종료 조건을 각각 충족한다.
2. `stage-run`은 요청한 단계가 끝나면 멈춘다. 여러 단계의 결과를 한꺼번에 요청했지만 전체 Goal 실행 의도가
   명확하지 않으면, 아직 완료되지 않은 가장 이른 단계 하나만 수행한다.
3. `goal-run`은 하나의 사용자 요청 아래에서 section 6.6의 자동 전환 계약에 따라 여러 GEX pass와 Phase를
   순서대로 수행할 수 있다. 단, 위 기본 감사 인계·선택적 단계검토 경계를 먼저 적용한다.
4. 선택 단계의 prerequisite가 없으면 뒤 단계로 건너뛰지 않는다. `stage-run`은 그 단계에 허용된 인수·
   제한적 기록 보완까지만 수행하고, 다른 단계가 필요하면 owner와 이유를 보고한다. `goal-run`은 section 5의
   재개 판정과 section 6.6이 허용한 in-scope correction·re-audit를 분리된 pass로 수행할 수 있다.
   새 구현의 선행 조건과 이미 승인된 변경의 제한적 교정 조건을 혼동하지 않는다.
5. 사용자가 단계나 Goal을 지정하지 않았고 직접 결과와 current artifact로도 실행 대상 또는 실행 모드를
   정할 수 없을 때만 사용자에게 묻는다.
6. `GEX-10`의 직접 `stage-run` 요청 또는 active `goal-run` authorization은 선택된 Goal 범위의 구현·테스트와
   필요한 코드/데이터/문서 수정을 승인한다. `GEX-00` 완료, P2G-60 Handoff 또는 Goal 작성만으로 구현이
   승인되지는 않는다.
7. `GEX-20`과 `GEX-30` pass는 감사와 감사 증거 작성을 승인하지만 구현 수정을 승인하지 않는다. active
   `goal-run`도 이 read-only 경계를 바꾸지 않는다.
8. `goal-run`의 자동 전환 권한은 exact target과 그 Goal의 기술적 correction에만 적용된다. 정책 의미 변경,
   Goal/Phase 구조 변경, 범위 확대, 배포 또는 외부 환경 변경은 별도 사용자 결정이나 권한 없이는 수행하지 않는다.
9. 재개 권한은 현재 사용자 요청과 확인 가능한 허용 범위에서 판정한다. 과거 PLAN의 더 넓은 root target이나
   현황 문서 갱신만으로 현재 요청을 다른 Goal/Topic으로 확대하지 않는다. 현황 문서가 exact 대상·허용 변경·
   종료점을 명확히 가리키면 그 문서를 지정한 실행 요청으로 판정할 수 있지만 파일 자체가 권한을 만들지는 않는다.
10. 인수 결과가 이미 current `Complete / High`이면 재구현하지 않는다. 구현 종료·감사 대기·교정·stale을
    구분해 section 6.6의 필요한 pass부터 재개한다. 단순 단계 변경·첫 패치·해결 가능한 실패만으로는
    active goal-run을 중단하지 않되, 사용자 checkpoint·독립 reviewer 요구·실제 정책/권한 경계는 유지한다.
11. 정책을 발견하면 section 6.5에 따라 통상적 정책·위임 범위를 먼저 판정한다. 사용자 의논이 필요한 경우에만
    section 6.6에 따라 주 작업은 일반 채팅의 최종 답변으로 질문 후
    대기한다. 이미 실행 중이며 독립성이 확인된 작업·하위 agent만 기존 범위까지 완료할 수 있다. 정책 의존성이
    있거나 불분명하면 중단하며, 답변 전 새 작업·위임·후속 전환을 시작하거나 질문 카드만 띄운 채 진행하지 않는다.

### 선택적 읽기와 중단 후 재개

처음에는 Reader Contract, 현재 단계, 직접 사용하는 공통 계약을 읽는다. 기존 실행 기록은 section 5의 최소
선독 규칙을 따른다. 관련 source·설정·지침·환경이 그대로면 이미 확인한 내용과 유효한 근거를 재사용하며,
매 pass·명령마다 전체 지침·전체 과거 이력·전체 저장소를 다시 읽지 않는다.

같은 reviewer가 이미 이해한 불변 요구·공통 규칙은 exact ref와 변경 여부를 확인해 재사용한다. 새 reviewer는
현재 Goal의 요구를 직접 이해해야 하지만 구현 일지 전체를 처음부터 읽을 의무는 없다. §5의 현재 locator와
조건별 영향·근거에서 시작하고, 의미가 불명확하거나 누락·모순·예상 밖 변경이 있을 때만 관련 원문을 확장한다.

도구/세션 한계, 취소 또는 의미 있는 진전이 없는 반복으로 더 진행할 수 없으면 마지막 유효 결과·실제 실패·
필요한 다음 pass와 미수행 범위를 남긴다. 완료나 백그라운드 실행을 약속하지 않는다. 재개 요청에서는 이 기록과
현재 actual·권한을 대조하고, 이미 끝난 구현을 자동으로 반복하지 않는다.

## 1. 목적

P2G에서 작성한 Goal을 실제 프로젝트에 구현하고, 구현자의 완료 주장과 분리된 감사로 완료 여부를 판정한다.
multi-phase Goal Set은 각 Phase를 개별 구현·감사한 뒤 마지막에 Phase 사이의 통합 계약을 다시 감사한다.

이 문서가 강하게 제한하는 것은 다음 경계다.

- 정확히 어떤 Goal과 Phase를 실행하는가
- 누가 정책 의미와 Phase/Goal 구조를 변경할 수 있는가
- 어떤 요청이 구현 수정을 승인하는가
- 구현과 완료 감사를 어디에서 분리하는가
- 어떤 증거가 있어야 Phase와 Master를 완료로 판정하는가
- 어떤 실행 authorization이 다음 단계와 다음 Phase를 시작할 수 있는가

구체적인 조사 방법, 명령, 테스트 순서, 파일 탐색 순서와 내부 작업 표현은 AI의 판단에 맡긴다.

## 2. 범위

### 포함

- P2G-60 Handoff와 exact upstream Goal 인수
- 한 pass에서 실행할 Single/Phase Goal 선택과 goal-run 전체 대상 관리
- 현재 코드·데이터·문서와 dirty work 기준선 확인
- 기존 실행 문서의 역할 매핑과 근거가 있는 제한적 보완, 실제 상태에 따른 재개
- 선택된 Goal 범위의 코드 기반 상세 설계·구현과 검증
- 구현 과정과 실패·재검증 증거 기록
- Single/Phase Goal 완료 감사
- multi-phase Goal Set의 최종 Master 통합 감사
- 기술 결함의 동일 Goal 재작업 또는 올바른 P2G owner 단계로의 반환

### 제외

- P2G-10 owner를 거치지 않는 새로운 material policy 확정 또는 위임되지 않은 정책의 대리 결정
- Final Policy Freeze, Work Package, Atomic Group, Phase 또는 Goal 구조의 임의 변경
- active goal-run 범위 밖의 다음 GEX 단계나 다음 Phase 자동 실행
- 배포, release, 운영 환경 변경과 production rollout
- 범용 동시성 제어, crash recovery, CAS와 publication protocol
- Campaign, Role-ID, Basis-ID 또는 별도 activation state machine

배포나 운영 변경이 Goal 범위에 실제로 포함되어 있더라도, 이 문서만으로 해당 외부 변경 권한이 생기지는 않는다.
그 작업에 필요한 사용자 승인과 환경별 안전 절차를 별도로 따른다.

## 3. 전체 흐름

```text
P2G-60 Handoff
  -> 사용자 요청: stage-run 또는 goal-run 시작
  -> GEX-00 실행 인수와 현재 상태 확인
       -> 이미 current 완료: 같은 대상의 완료 근거 보고
       -> 구현 종료/재감사 대기: 해당 fresh audit
       -> 신규 또는 허용된 교정: 아래 GEX-10
       -> 모든 Phase current 완료/미완료 Master 감사: GEX-30
  -> GEX-10 선택된 Goal 상세 설계·구현과 검증 (명시적 opt-in이면 중간 checkpoint에서 종료)
  -> 구현 후 target 고정·종료 / 감사 요청 대기 (감사까지 명시 위임한 경우에만 자동 전환)
  -> 감사 권한 확인 후 fresh GEX-20 Single/Phase 완료 감사
       -> in-scope 기술 결함: correction GEX-10 -> fresh GEX-20
       -> Phase-local blocker: dependent Phase와 GEX-30 차단, 독립 eligible Phase는 조건부 진행
       -> shared basis/policy/구조/외부 권한 blocker: run 중단, 해당 owner로 반환
       -> Complete / High:
            Single/Phase target: target 실행 완료
            Goal Set target + 미완료 eligible Phase 존재: 다음 Phase의 GEX-00
            Goal Set target + 모든 Phase 완료: fresh GEX-30
  -> fresh GEX-30 Master 최종 통합 감사
       -> in-scope cross-Phase 기술 결함: 영향 Goal의 GEX-10/GEX-20 -> fresh GEX-30
       -> policy/구조/외부 권한 blocker: run 중단, 해당 owner로 반환
       -> Complete / High: Goal Set 실행 완료
```

`stage-run`은 위 흐름에서 요청된 한 단계 뒤 멈춘다. `goal-run`도 Reader Contract의 checkpoint가 없고
해당 후속 작업의 권한이 있을 때만 section 6.6의 전환을 자동으로 이어간다.

multi-phase에서 Master Goal은 조정과 전체 완료 조건만 소유한다. Master 자체가 Phase 구현 작업을 수행하지
않는다. 하나의 GEX pass는 Single/Phase Goal 하나만 실행 대상으로 삼는다. Goal Set `goal-run`은 여러 Phase
pass를 순서대로 이어갈 수 있지만 서로 다른 Phase 구현을 한 pass에 합치지 않는다. Phase Plan이 병렬 실행을
허용하는 것은 병렬 실행 의무가 아니며, 이 파이프라인의 goal-run은 Phase를 직렬로 진행할 수 있다.

### P2G-60 인수 계약

P2G-60 산출물과 GEX consumer의 관계는 다음과 같다. 이 표는 새 artifact를 만들지 않고 두 파이프라인의 인수
관계만 고정한다.

| P2G-60 Handoff 내용 | GEX에서 사용하는 곳 | 의미 |
| --- | --- | --- |
| Handoff Ref와 exact Result Composite Basis | 모든 GEX 단계 | accepted upstream policy/plan/Goal identity |
| Final Policy Freeze Ref | GEX-10/20/30 | 승인된 policy meaning과 Observable Policy Acceptance source |
| Phase Plan Ref | GEX-00/20/30 | Phase dependency, assigned execution node, 순차·병렬, integration boundary와 Integration Validation Ownership Map |
| Goal Set Ref와 `(Goal Set Ref, Goal ID)` | GEX-00/10/20/30 | exact Single/Phase execution target, Master coordination relation, Goal Quality Contract/Validation Dimension Matrix와 integration validation mapping |
| Goal member의 코드 기반 설계 인계 | GEX-00/10/20 | 승인 의미와 권장 구조, 구현자가 닫을 상세 선택·준비 점검·검증 기준 |
| Goal Set의 applicable Shared Validation Baseline Reuse Contract | GEX-00/10/20/30 | initial/final baseline owner, input-to-seam 영향, 근거 재사용과 재실행 범위 |
| Remaining Gap/Blocker/Limitation/Out-of-Scope Summary | GEX-00과 각 exit 판단 | 숨기거나 pass로 바꾸면 안 되는 실행 전제 |
| P2G Scope/Authorization Boundary | GEX-00/10 | P2G는 구현을 승인하지 않았으며 direct GEX-10 또는 exact goal-run authorization이 별도로 필요함 |

GEX는 위 upstream payload를 복제하거나 수정하지 않는다. GEX 실행 문서는 accepted Ref, current actual과 실행
evidence를 연결한다.

## 4. 권한과 판단 경계

### 사용자 소유

- material gameplay, balance, UX, public contract, persistence loss/lifetime, compatibility, migration과
  rollout의 정책 의미 중 P2G §8.6의 통상적 정책 기준·위임 범위로 결정할 수 없는 선택
- `stage-run` 또는 `goal-run`의 시작, exact target과 authorization 범위
- goal-run 중단 지점에서 필요한 정책·범위·외부 권한 결정
- 별도 승인이 필요한 production, 배포 또는 외부 환경 변경

통상적 정책의 정의·다섯 조건·위임과 기록의 원본은
[P2G §8.6](./policy_to_goal_pipeline_revised_v2.md#86-blocker와-중단-조건)이다. 사용자 선택이 필요한 정책만
의논 대상으로 삼는다. 위임된 통상적 정책이라도 정식 의미·acceptance의 확정과 변경은 section 6.5의
P2G owner 반환을 따른다. 이 문서나 통상적 판정이 P2G 실행·범위 확대 권한을 새로 만들지 않는다.

### AI가 선택된 GEX-10 안에서 결정할 수 있는 것

- 승인된 정책과 Goal 범위 안의 module, type, function, file과 private representation
- 조사 방법과 순서
- 테스트, probe, fixture와 validation seam
- Goal이 허용한 범위 안의 기술적 구현 순서와 작은 trial-and-error
- 동일한 정책 의미를 보존하는 기술적 correction

### AI가 임의로 결정할 수 없는 것

- 열린 정책 질문의 답
- Observable Policy Acceptance의 의미 변경
- Goal scope 확대 또는 out-of-scope 항목 편입
- Work Package/Atomic Group/Phase dependency와 Goal mapping 변경
- 증거가 없는 compatibility, consumer 부재 또는 완료 주장
- 완료 감사를 구현 수정과 같은 pass에서 통과시키는 것
- active goal-run 범위에 포함되지 않은 다음 Phase의 선택·초기화·구현

## 5. 실행 문서와 증거

Goal 실행 문서는 진행 중 작업의 기억장치와 실행 증거다. durable 정책이나 P2G artifact를 대신하지 않는다.
기본 Goal 디렉터리는 다음 네 문서를 사용한다. 기존 Goal이 동등한 구조를 이미 가지고 있으면 중복 문서를 만들지
않고 그 문서를 이어서 사용한다.

| 문서 | 역할 | 주요 owner 단계 |
| --- | --- | --- |
| `PLAN.md` | exact Goal, 실행 범위, current baseline, 상세 설계·준비 점검, validation·완료·중단 조건 | `GEX-00`, 필요 시 `GEX-10`의 current evidence 보완 |
| `EXPERIMENTS.md` | 시간순 조사, 시도, 구현, 명령, 실패, 결과와 재검증 기록 | `GEX-10` |
| `EXPERIMENT_NOTES.md` | 현재 기술 판단, evidence gap, 정책 질문, blocker와 후속 후보 요약 | `GEX-00`·`GEX-10`·감사 결과의 요약 |
| `COMPLETION_AUDIT.md` | 구현과 분리된 Single/Phase 또는 Master 완료 감사와 verdict | `GEX-20` 또는 `GEX-30` |

multi-phase Master 디렉터리도 같은 문서명을 사용할 수 있다. 이때 Master `PLAN.md`와 `EXPERIMENTS.md`는 Phase
관계와 감사 결과를 요약할 뿐 Phase별 구현 세부사항을 복제하거나 소유하지 않는다.

새 registry, digest, snapshot 또는 중간 ID는 실제 owner와 consumer가 생길 때만 추가한다. 다음 항목은 별도
artifact가 없어도 위 문서에 exact reference와 evidence로 기록할 수 있다.

- P2G-60 Handoff, Goal Set과 `(Goal Set Ref, Goal ID)`
- direct GEX-10 요청 또는 exact goal-run authorization과 승인된 mutation 범위
- 담당 Phase ID와 assigned Final Execution Node
- exact execution-document directory와 문서별 normalized path
- current repository revision과 scoped dirty baseline
- 실행·검증 결과와 completion-condition evidence
- section 5의 Audit Target Record가 식별한 exact current tree, external actual과 evidence set
- verdict와 다음 필요한 owner 단계

### 현재 요약과 고정 증거의 구분

현재 상태의 원본 locator는 PLAN 또는 동등한 현재 요약 한 곳이 소유한다. latest 구현/교정 결과, exact
input/target, 미해결 finding, 검증/감사 evidence와 다음 필요한 pass를 짧게 연결한다. 다른 문서에는 같은
경위·건수·해시를 복사하지 말고 exact ref/절과 해당 문서의 판단만 쓴다. 날짜나 큰 record 번호만으로 최신을 정하지 않는다.

구현/교정 종료 후 감사 대상으로 고정하기 전에 현재 요약을 맞춘다. 이미 동결된 PLAN/NOTES를 고치면 안 되는
상황에는 허용된 비대상 현황/인수 기록에서 최신 successor를 가리키고, 이전 요약을 당시 snapshot으로 구분한다.
동결 문서·원시 로그·실패·기존 verdict는 그대로 보존한다. 위치 안내는 승인·PASS·대상 identity의 대체물이 아니다.
현재 요약이 뒤처졌다는 이유로 전체 EXPERIMENTS를 재독하지 말고 직접 연결된 최신 result/correction부터 확인한다.

### Execution-document directory

각 Single/Phase Goal member와 multi-phase Master는 정확히 하나의 execution-document directory를 사용한다.
GEX-00은 문서를 만들거나 갱신하기 전에 다음 순서로 directory를 정한다.

1. current Goal artifact, Handoff 또는 repository convention이 exact path를 지정하면 그것을 사용한다.
2. exact `(Goal Set Ref, Goal ID)`를 이미 기록한 기존 execution document set이 정확히 하나면 그 directory를
   이어서 사용한다.
3. 기존 path가 없으면 root Goal 문서와 repository의 기존 Goal layout을 기준으로 member별 path를 하나 정한다.
   multi-phase Master와 각 Phase member는 서로 다른 directory를 사용한다.
4. 같은 exact Goal member를 가리키는 후보가 둘 이상이거나 existing history를 보존할 canonical path를
   판정할 수 없으면 새 문서를 만들지 않고 중단한다.

선택한 directory와 역할별 실제 normalized path/section locator는 GEX-00 산출물과 PLAN 역할의 current 절에
기록한다. 위 네 파일명은 기본 역할명이다. 기존의 동등한 파일명·한 파일의 여러 절·연결된 이력 분할을 허용하며,
이후의 `PLAN.md` 등 표기는 이 역할 매핑을 따른다. 여러 locator가 같은 역할의 독립 current 원본이 되게 하지
않는다. 별도 mapping registry는 만들지 않고 기존 인수 기록에 한 번 적는다. 도구가 고정 파일명을 실제로
요구하면 해당 도구 계약을 확인하며, 이를 맞춘다는 이유로 기록을 무단 이동·복제하지 않는다.

이후 GEX pass와 correction/re-audit는 이 path와 locator를 재사용한다. execution-document path는 upstream
Goal identity를 대체하는 authority가 아니다.

한 파일의 여러 절을 사용할 때도 구현 target과 감사 evidence의 경계는 식별 가능해야 한다. 감사 기록 추가를
이유로 구현·승인·검증 원문 변경을 숨기지 않는다. 구간별로 target identity를 안정적으로 비교할 수 없다면
그 target 파일은 그대로 두고 별도의 감사 evidence 위치를 사용한다. 감사 시작 후 target을 임의로 축소하지 않는다.

### 기존 Goal 실행 문서 재개 선독 Gate

exact `(Goal Set Ref, Goal ID)`에 대응하는 실행 기록이 있으면 GEX-00은 새로운 구현 조사·테스트·production
mutation보다 먼저 current 실행 상태를 인수한다. 이 gate는 전체 history 재독이 아니라 최소 current context와
직접 dependency 확인이다. 경로·역할·중단 원인을 확인하기 위한 제한적 파일/기록 검색과 VCS 이력 조회는
허용한다. 필요한 기록을 찾지 못한 상태를 곧바로 신규 실행으로 간주하지 않는다.

먼저 PLAN·EXPERIMENTS·EXPERIMENT_NOTES 세 역할을 실제 파일/절에 매핑하고, 감사가 필요한 경우 AUDIT 역할을
함께 확인한다. 파일 개수가 아니라 필요한 사실의 존재·해석 가능성을 기준으로 다음과 같이 판정한다.

| 확인한 상태 | 처리 |
| --- | --- |
| 세 역할과 exact Goal의 다른 실행/audit history가 없다는 근거가 있음 | 신규 GEX-00 초기화. 단순 검색 실패만으로 history 부재를 확정하지 않음 |
| 세 역할의 current 내용이 동등한 구조에 존재함 | 이름·파일 수와 무관하게 기존 기록에서 재개. 중복 파일을 생성하지 않음 |
| 초기화 중단이 확인되며 실제 실행 전이고 일부 역할만 작성됨 | GEX-00에서 남은 역할을 작성. 미수행 항목은 미수행으로 기록하며 완료 이력을 만들지 않음 |
| 필요한 사실이 exact upstream·남아 있는 실행/감사 원본에 온전히 존재하지만 요약/locator가 누락됨 | 원문을 보존하고 누락된 구조·요약만 GEX-00에서 제한적으로 보완. 복구 위치·근거·한계 기록 |
| 실행 사실·승인·시험 결과가 유실됐거나 current record/권한을 유일하게 식별할 수 없음 | `Blocked`. 사실을 추정해 빈 기록이나 성공 이력으로 대체하지 않음 |

제한적 보완은 현재 요청이 해당 GEX-00 인수·기록 작성을 허용할 때만 수행한다. 실제 수행 여부가 불명확하면
`실행 기록 없음`을 `실행하지 않았음`으로 바꾸지 않는다. 현재 코드가 존재한다는 사실만으로 과거 구현자·명령·
테스트 결과·승인을 재구성하지 않는다. 초기가 아니라 실행 이력 손실일 가능성을 배제할 수 없으면 보완 범위를
늘리지 않고 필요한 근거를 보고한다. 별도 crash-recovery 프로토콜이나 승인 장부를 만들지 않는다.

감사만 허용된 GEX-20/30에서는 문서 복구·source 교정을 함께 수행하지 않는다. audit target을 바꾸는 기록 보완은
해당 인수/기록 owner에서 먼저 수행하고 변경 영향을 다시 판정한다. history·원본 감사 verdict는 보존한다.

`COMPLETION_AUDIT.md` 역할은 구현 전부터 필수는 아니다. 완료 재사용, correction, re-audit 또는 predecessor
completion을 인수할 때는 latest applicable verdict·Audit Target Record·currentness가 직접 dependency이므로
필요한 절을 읽는다. 있어야 하는 audit를 찾지 못하면 완료로 추정하지 않는다.

#### 최소 선독 범위

문서마다 먼저 heading, 목차, exact Ref, `current`·`latest`, current pass/result와 직접 dependency locator를
찾고 다음 범위만 읽는다.

- `PLAN.md`: exact Goal과 Phase, run mode/root target/authorization, current pass, in/out scope, 직접
  predecessor/dependency, current baseline, validation·completion·stop condition을 담은 current section
- `EXPERIMENT_NOTES.md`: current technical summary, unresolved evidence gap·policy question·blocker, 현재 유효한
  결정과 current `PLAN.md`가 직접 가리키는 note
- `EXPERIMENTS.md`: current baseline 이후 아직 유효한 latest implementation/correction/validation pass와
  `PLAN.md` 또는 `EXPERIMENT_NOTES.md`의 current 판단이 직접 의존하는 기록
- conditional `COMPLETION_AUDIT.md`: 재개 eligibility나 dependency에 직접 쓰이는 latest applicable verdict,
  Audit Target Record/currentness, blocking finding과 필요한 correction 범위

완료된 과거 pass, superseded baseline, 실패 뒤 폐기된 접근, 독립 sibling Goal과 current 판단이 참조하지 않는
history는 기본 선독 범위가 아니다. 검색 hit만 읽고 문맥을 추측하지 않으며, 선택한 항목의 의미를 판정할 수 있는
최소 연속 section과 그 section이 명시적으로 참조하는 exact record까지 읽는다.

#### Phase Goal의 직접 dependency 선독

선택 대상이 multi-phase Goal Set의 Phase Goal member이면 다음 직접 dependency만 추가로 읽는다.

- Master Goal에서 현재 member, 담당 Phase, cross-Phase invariant와 applicable integration acceptance를 설명하는 부분
- Phase Plan과 Goal Set이 선언한 직접 predecessor Phase의 latest applicable 감사와 currentness. 신규 착수에는
  current `Complete / High`가 필요하며, section 6.6의 제한적 교정에서는 현재 무효화된 조건과 과거 정상 인수
  근거를 함께 읽되 과거 결과를 current 통과로 사용하지 않음
- 현재 member가 직접 의존하는 predecessor 또는 correction Goal의 current 결정

Master 전체 history, 독립 sibling Phase Goal의 구현 기록과 현재 member에 적용되지 않는 integration row는 읽지
않는다. 직접 predecessor가 없으면 Phase Plan/Goal Set의 dependency relation으로 그 사실을 확인해 `PLAN.md`에
`direct predecessor dependency: none`으로 기록하며 새 predecessor나 completion evidence를 만들지 않는다.

#### 선독 범위 확대 조건

다음 중 하나일 때만 필요한 방향으로 선독 범위를 한 단계씩 넓힌다.

- current/latest marker 또는 exact locator가 없어 current section을 식별할 수 없음
- current `PLAN.md`, notes, experiment 또는 audit 사이의 Ref·result·dependency가 모순됨
- current section이 요약하지 않은 이전 결정이나 evidence를 completion/blocker 근거로 직접 참조함
- 승인 scope, authorization, dependency eligibility, stop condition 또는 audit currentness를 현재 범위에서
  판정할 수 없음

먼저 명시된 exact reference와 인접한 최소 section을 읽고, 그래도 위 판정이 불가능할 때만 더 오래된 관련
history로 확장한다. 문서 전체를 처음부터 끝까지 읽는 것은 구조나 locator가 없어 그 문서 전체가 current 판단에
실제로 필요하다는 근거가 있을 때의 마지막 수단이다. 확대 이유와 current 판단에 사용한 section/ref는
`PLAN.md` 또는 `EXPERIMENT_NOTES.md`에 기록한다.

선독 뒤의 재개 disposition은 새 Status 체계를 만들지 않고 GEX pass result와 audit currentness를 따른다.
기록상 완료 주장과 current actual을 필요한 범위에서 대조한 뒤 section 6.6의 재개표로 다음 pass를 정한다.
조사로 해소 가능한 불명확함은 관련 근거를 확인하되, 필수 result/dependency를 끝내 판정할 수 없으면 중단한다.
이 gate는 실행 문서를 runtime·live data·wire 또는 정책 meaning의 새 source of truth로 승격하지 않는다.

### Audit Target Record와 currentness

완료 verdict는 모호한 "현재 tree"가 아니라 재식별 가능한 Audit Target Record에 귀속된다. 별도 registry나
repository 전체 snapshot은 요구하지 않으며, GEX-10이 Phase 감사 대상을 고정할 때 또는 GEX-30이 Master 감사
활동을 시작하기 전에 기존 실행 문서에 다음 applicable identity를 기록한다.

- repository/worktree locator, VCS revision과 applicable submodule/dependency revision
- audit impact surface의 tracked/clean, tracked/dirty, untracked와 generated file manifest
- manifest member별 path, kind와 content identity. clean tracked content는 revision과 path로 식별할 수 있고,
  dirty/untracked/generated content는 digest 또는 동등하게 다시 비교할 수 있는 identity를 사용함
- runtime/data/schema/store/transport/consumer가 applicable하면 environment/endpoint, schema·data·consumer
  version, selector/query, capture 시점·transaction 또는 exact evidence ref와 known mutability
- validation command, test selector, configuration, fixture/input과 result identity
- explicit exclusion, inaccessible/unknown surface와 완료 주장에 미치는 영향
- implementation audit target에서 제외되는 `COMPLETION_AUDIT.md`, 감사 evidence와 audit-local temporary output

새 target은 해당 완료 판단의 실제 기준·현재 source/설정/소비자·사용하는 evidence와 필요한 부재/목록 검사로
범위를 정한다. 모든 과거 target의 전체 파일 목록을 관성적으로 재귀 전개하지 않는다. 공통 immutable manifest는
exact identity와 member 집합을 참조하여 공유할 수 있지만 실제 파일·전이적 입력의 현재성 검사를 생략할 수 없다.
기존에 고정한 target의 범위나 실패 기록을 비용 절감 명목으로 사후 삭제·축소하지 않는다. 이 지침을 적용하는
새 target/운영 보완은 이전 대상·delta·포함/제외 이유를 추적하고, 필수 입력 누락을 간결성으로 정당화하지 않는다.

content identity의 형식이나 hash algorithm은 고정하지 않는다. 그러나 다른 pass가 동일 여부를 다시 판정할 수
없으면 direct evidence가 아니다. mutable external actual을 완전히 고정할 수 없으면 capture identity와 재검증
방법을 기록하고, completion-blocking unknown이 남으면 `High`를 허용하지 않는다.

verdict는 해당 Audit Target Record에 대한 immutable historical result다. 후속 변경이 이전 verdict를 소급
수정하지는 않지만, 그 감사를 prerequisite나 current completion evidence로 재사용할 수 있는지는 각 GEX-00,
GEX-20과 GEX-30 consumer pass에서 다시 판정한다.

다음 상태는 아래 순서로 평가하며 정확히 하나만 판정한다.

- `current / unchanged`: 모든 applicable target identity가 동일하고 applicable external actual과 evidence가 여전히
  유효함
- `current / proven non-impacting delta`: 하나 이상의 applicable target identity가 변경됐지만 changed member와
  영향 경로를 식별했고, audited Goal의 scope, assigned execution node, contract-visible result, completion
  condition, validation evidence와 producer-consumer behavior에 영향을 주지 않는다는 current direct evidence가
  있으며 나머지 applicable external actual과 evidence도 유효함
- `stale`: 앞선 두 조건을 충족하지 못함. external actual/evidence 만료, 변경 영향의 unknown 또는 필요한
  non-impact evidence 부재를 포함함

`current / proven non-impacting delta` 판정에는 changed member, 영향 경로, 확인 방법과 evidence를 기록한다.
파일 경로가 manifest 밖이라는 사실만으로 non-impacting을 주장하지 않는다. `stale` audit은 신규 Phase의
prerequisite나 GEX-30의 current Phase audit으로 사용할 수 없으며 fresh audit가 필요하다. section 6.6의
제한적 교정은 stale을 통과로 바꾸는 예외가 아니라, 무효화 원인을 수리한 뒤 필요한 감사를 복원하는 진입 경로다.

fresh re-audit은 implementation을 수정하지 않고 current actual에서 새 Audit Target Record를 고정할 수 있다.
이 경로는 upstream Goal 의미와 completion condition이 그대로이고 source/data correction이 필요하지 않으며,
다음 중 하나의 근거가 있을 때만 허용된다. 구현 delta가 있으면 이미 완료된 GEX-10 또는 Phase Plan이 승인한
successor Phase mutation에 정확히 귀속한다. 구현은 그대로이고 evidence 만료·추가 확인만 필요하면 기존 구현
종료 근거와 새 관찰·검증의 identity/적용 범위를 확인한다. 이 경우 source delta owner는 근거 있는 N/A이며
없는 mutation을 발명하지 않는다. 새 record는 predecessor Audit Target Record, 변경된 구현 또는 evidence의
identity와 실제 owner/관찰 경로를 가리키고, 이전 record/verdict를 덮어쓰지 않는다. defect가 있으면 책임 변경과 exact 교정 Goal을 먼저 식별하여 GEX-10으로
반환한다. delta owner가 unknown이면 제한적 read-only 근거 확인을 하고, 끝내 식별하지 못하면 근거 부족으로
중단한다. 소유자가 없는 교정 Goal을 발명하지 않는다. Goal/Phase 경계가 바뀌어야 하면 section 8의 P2G owner로
반환한다. 교정된 current target은 영향받은 audit를 새로 통과해야 하며, 기존 verdict는 historical로 남긴다.

후속 변경의 audit 영향은 **changed input → 실제 producer/consumer → 완료 조건·검증 근거**로 판정한다.
predecessor의 assigned execution node, contract-visible output, schema/store/transport/consumer, legacy
처분, validation seam/fixture 또는 completion evidence에 실제 영향이 있거나 영향이 불명확하면 `stale`이다.
Phase Plan이 예상한 변경이나 결함을 고친 변경도 무영향은 아니다. 반대로 공유 파일의 수정·단계 번호·문서 날짜·
감사 보고서 ID 변경만으로 모든 선행 Goal을 일괄 stale 처리하지 않는다. 앞선 무영향 직접 증거 요건은 그대로다.

#### 조건별 영향과 재감사 범위

기존 실행/감사 기록의 표나 동등한 추적 구조에서 다음을 구분한다. 새 stage, verdict enum 또는 필수 registry가 아니다.

| 조건의 상태 | 필요한 검토와 기록 |
| --- | --- |
| 변경의 영향 있음 | 실제 변경·소비 경로·무효화된 근거, 현재 source와 필요한 새 검증을 직접 대조한다. |
| 무영향을 입증함 | 조건/가정과 관련 입력·소비자·환경·oracle의 유효 근거, 재사용할 exact 이전 판단/증거를 연결한다. |
| 영향 미확인·근거 만료/부족·이전 미해결 | 필요한 원문/코드/검증으로 확장한다. 해소 전에는 무영향이나 PASS로 처리하지 않는다. |

재감사는 원래 요구·완료 조건·applicable dimensions를 빠짐없이 추적하되, 이미 유효하게 검토된 불변 부분을
처음부터 재조사·재서술하지 않는다. 이전 target과 현재 target 사이의 **아직 평가하지 않은 전체 delta**를 기준으로
검토하며 마지막 수정만 보고 누적 미검토 변경을 누락하지 않는다. 변경 파일 수나 높은 시험 통과율은 무영향 증거가 아니다.

기존 판단의 재사용은 이전 결론을 믿으라는 뜻이 아니다. 감사자가 해당 조건의 원문 의미, 실제 source/소비 관계와
증거의 적용 범위·현재성을 독립적으로 확인하고 자신의 재사용 판단을 기록해야 한다. 불변 근거는 exact ref로 연결할
수 있고 의심·모순·불완전성에는 관련 원문을 확장한다. initial 감사에도 유효한 시험은 재사용할 수 있으나, 존재하지
않는 이전 의미 판단을 인수한 것으로 기록하지 않는다. 이전 Not Complete의 개별 검토 결과는 이 조건에서 재사용할
수 있지만 미해결 finding과 전체 부정 verdict를 성공으로 바꾸지 않는다. 변경 의미/범위 자체가 달라졌으면 P2G로 반환한다.

모든 필수 조건이 현재 직접 검토 또는 검증된 재사용 근거로 충족되어야만 새 `Complete / High`를 발행한다.
재사용한 이전 기록·결과 시점은 그대로 두고 이번 판단만 추가한다. 재감사 순서는 영향받은 실제 의존과 유효한
선행 결과로 정하며 R/Phase 번호만으로 반복 체인을 만들지 않는다. 필요한 cross-Goal 감사의 권한은 별도로 확인한다.

### 문서별 최소 내용

`PLAN.md`는 다음을 식별할 수 있어야 한다.

- exact Handoff/Goal Set/Goal member와 담당 Phase
- `stage-run | goal-run`, root execution target, current pass와 authorization 범위
- 기본 최종 감사 인계 또는 명시적 감사까지 위임, 단계검토 opt-in 여부·요청 근거·적용 범위
- opt-in이면 checkpoint별 포함 작업·선행/남은 의존·빌드/검증 명령·통과 조건·전후 snapshot과 검토 기록 locator
- 현재 검토 대기/교정/재검토/다음 구현 대기, 실제 받은 검토·교정·진행 요청. PASS와 진행 승인을 구분
- exact execution-document directory와 역할별 path/section locator
- 적용하는 GEX 지침 revision과 현재 요청의 근거. 과거 authorization과 이번 범위를 구분
- objective, contract-visible result, in-scope와 out-of-scope
- predecessor/dependency와 current execution prerequisite
- current behavior, target behavior와 implementation boundary
- 인계받은 권장 설계·가정, 코드 기반 상세 설계·준비 점검의 적용 범위와 근거 또는 연결 위치
- applicable baseline reuse contract, initial/final owner, changed input-to-seam 판정과 reusable/재실행 근거
- 재개할 pass와 이유. 제한적 교정이면 exact owner·귀속 delta·해당 선행 조건·재감사 복원 순서
- current dirty baseline과 보존할 unrelated work
- applicable Goal Quality Contract version, Validation Dimension Matrix, validation 계획, completion conditions와
  stop conditions

`EXPERIMENTS.md`는 완료 주장에 사용한 사실을 재검토할 수 있어야 한다. 중요한 명령은 대상, 실제 결과,
실행 수 또는 relevant output을 기록한다. 실패한 접근과 수정·재검증도 삭제하지 않고 남긴다. 같은 범위의
사소한 명령/시험 오류는 명령·원인·수정·재검증 결과와 기존 범위 ref면 충분하다. 정책·권한·격리·target의
실제 변화가 없으면 전체 목적·이력·감사 계획을 매 시도마다 다시 쓰지 않는다.

`EXPERIMENT_NOTES.md`는 current summary이지 별도 정책 authority가 아니다. 새 정책은 통상적 판단·위임
근거 또는 사용자 선택이 필요한 이유, 영향 범위와 P2G owner를 기록한다. 위임된 AI 결정·사용자 직접 결정·
기술 결정을 구분하며, 이 요약으로 정식 정책 기록을 대신하지 않는다.

`COMPLETION_AUDIT.md`는 최소한 다음을 포함한다.

- exact audit target과 upstream Goal/Phase
- Audit Target Record와 audit 시작 시 currentness comparison 결과
- 감사 방법과 구현 pass와의 분리 관계
- 새로 실행한 검증과 유효성을 확인해 재사용한 근거의 구분, 적용되는 baseline contract와 영향 판단
- completion condition별 current implementation evidence, validation evidence와 판정
- applicable Goal Quality Contract, Validation Dimension별 evidence/N/A 재판정과 Implementation Quality
  Guardrails source review 결과
- applicable한 runtime/data/schema/store/transport/consumer/legacy/compatibility 검토
- omitted, inaccessible 또는 unknown evidence와 완료 주장에 미치는 영향
- scoped diff와 unrelated dirty work 보존 판정
- primary verdict와 다음 필요한 단계

형식은 고정하지 않는다. 위 사실을 모호하지 않게 연결할 수 있으면 표, 목록 또는 서술을 자유롭게 선택한다.
재감사는 이전 verdict를 삭제하거나 소급 수정하지 않는다. 같은 파일에 predecessor를 밝힌 새 audit pass를
추가하거나 별도 감사 파일로 보존할 수 있으며, 어느 방식을 쓰든 current verdict와 audit target을 구분한다.

## 6. 공통 실행 규칙

### 6.1 Exact upstream basis

최초 GEX-00은 current P2G-60 Handoff를 인수한다. 이후 모든 GEX 단계는 그 accepted Handoff가 선택한 exact
Final Policy Freeze, Phase Plan과 Goal Set을 계속 가리킨다. Handoff나 upstream artifact가 승인된 실행 delta
외의 이유로 authoritative하게 stale 처리됐거나 서로 다른 basis를 가리키면 구현을 시작하거나 완료하지 않는다.

최초 GEX-00은 current Handoff에서 시작한다. 정상 신규/후속 실행의 basis는 같은 Handoff와 section 5의
currentness 판정을 통과한 `Complete / High` predecessor Phase audit의 조합으로 읽는다. 이미 정상 인수한
변경의 회귀를 교정하는 경우에만 section 6.6의 제한적 재진입 조건을 적용할 수 있다. 이때도 P2G upstream은
유효해야 하며, 무효화된 감사는 수리 대상이지 current prerequisite가 아니다. Goal이 승인한 implementation delta와
완료된 predecessor Phase가 만든 예상 코드·데이터 변화는 그 자체로 Handoff를 재작성하거나 P2G로 돌아갈
사유가 아니다. 반대로 실제 변화가 Final Freeze의 의미, reviewed implementation boundary, Phase dependency나
Goal scope의 전제를 깨면 해당 P2G owner로 반환한다.

Goal 문서는 upstream 내용을 복제해 새로운 authority가 되지 않는다. 필요한 의미는 exact upstream reference로
연결하고, 실행 중 확인한 actual code/data evidence만 Goal 실행 문서에 기록한다.

Goal Set의 `Activation Status: Not Activated`와 `Implementation Authorized: No`, Handoff의 authorization
boundary는 P2G가 작성·인계 시점에 실행을 승인하지 않았다는 immutable 기록이다. GEX가 그 upstream artifact를
수정해 상태를 바꾸지는 않는다. 이후 사용자의 direct GEX-10 요청 또는 exact goal-run authorization이 selected
Goal 범위의 mutation을 새로 승인하며, 그 요청과 범위를 실행 기록에 남긴다.

### 6.2 Current tree와 dirty work

구현 전에는 선택된 Goal에 필요한 범위에서 repository status, 관련 파일과 current behavior를 확인한다.
기존 dirty work는 사용자 작업으로 간주하고 되돌리거나 현재 Goal 성과로 귀속하지 않는다. overlapping change를
안전하게 분리할 수 없으면 중단한다.

과거 Goal, audit, 오래된 test output과 debug artifact는 조사 단서일 수 있지만 current actual의 증거를
대신하지 않는다.

### 6.3 조사 방법의 재량

section 5의 기존 Goal 재개 선독 Gate와 선택된 pass의 prerequisite·권한 경계를 먼저 적용한 뒤, AI는 Goal과
위험에 맞춰 실제 상태 조사 방법을 선택한다. call graph, reference search, producer-consumer 추적, data flow,
state transition, persistence/version chain, DTO/transport mapping, test inventory와 before/after comparison은
유용한 권장 기법이다. 특정 명령, 알고리즘, 도구 또는 파일 순서를 강제하지 않는다.

검색 hit, 파일명, 오래된 문구 또는 테스트 하나만으로 authority, consumer 부재나 결함을 확정하지 않는다.
완료 주장에 필요한 실제 producer, mutation owner, consumer와 observable behavior를 적절한 범위에서 확인한다.

#### 코드 기반 상세 설계와 준비 점검

GEX-10은 인계된 권장 구조를 actual이나 확정 정책으로 오인하지 않는다. 필요한 코드를 조사한 뒤 구현 전에
적용되는 타입·입출력·상태 key/수명·사건/완료 순서·실패/취소/복원·데이터 전환·변경 심볼·시험 기대값을
상세화한다. 생산자와 소비자가 같은 계약을 사용하는지, 종료 경로가 완결되는지, 선행 계약이 실제로 있는지,
미정 정책을 임의 기본값으로 채우지 않았는지 점검한다.

간단한 내부 변경에는 짧은 근거면 충분하다. 상태·공유 인터페이스·저장 전환은 영향에 맞게 깊게 작성한다.
기존 PLAN의 절 또는 실제 설계 문서를 연결하며 모든 Goal에 별도 DESIGN 파일·새 Gate·추가 reviewer를 강제하지
않는다. 요구된 독립 reviewer나 사용자 checkpoint는 생략하지 않는다. 자기 점검을 독립 감사로 보고하지 않는다.

정책·공유 계약을 보존하는 구현 선택은 AI가 근거와 시험을 남겨 결정한다. reviewed premise, consumer-visible
의미, Phase/Goal 경계가 바뀌면 section 8의 owner로 반환한다. 시험 구현이 필요하면 승인 범위의 GEX-10에서
수행하고 probe와 production 완료를 구분한다. 상세 설계·준비 점검은 완료 증거를 대체하지 않는다.

### 6.4 구현과 검증

- 작은 변경 단위마다 필요한 focused validation을 수행하고 실패 원인과 재검증을 기록한다.
- 완료 전에는 Goal 위험과 변경 표면에 비례한 broader validation coverage를 확인한다. 아래 재사용 계약에
  따라 유효한 기존 근거를 사용할 수 있으며, 불충분하거나 무효화된 coverage는 실제 재검증한다.
- zero test, skipped test, 실행되지 않은 command와 unknown result는 pass가 아니다.
- 기존 baseline failure와 이번 변경이 만든 failure를 구분한다.
- validation을 생략하면 이유와 완료 주장에 미치는 영향을 기록한다. 필수 조건을 침범하는 생략은 완료를
  허용하지 않는다.
- 임시 fallback, dual path, shim 또는 compatibility layer는 Final Freeze/Goal이 허용하고 범위·검증·제거
  조건이 충분한 경우에만 구현한다.
- 구현으로 바뀐 durable contract, schema, wire 문서나 fixture가 있다면 같은 Goal 범위에서 정합성을 맞춘다.

#### 검증 근거 재사용과 initial/final baseline

Goal Set에 `Shared Validation Baseline Reuse Contract`가 있으면 GEX-00에서 exact source, initial/final command
set·owning executable Goal·시점, input-to-seam 영향 map과 현재 baseline evidence를 인수한다. 원본 계약을 복제해
새 authority로 만들지 않는다. 계약에 지정된 initial baseline이 필요한 신규 실행이면 mutation 전에 해당 owner가
수행한다. 이미 유효한 실행 근거가 있으면 최초 실행인 것처럼 다시 만들지 않는다.

재사용 판단에는 command/selector, fixture/input, 관련 source/data/test/config, toolchain·lock/dependency,
환경과 result의 실제 identity 및 근거의 유효 범위를 사용한다. Phase·감사 pass 시작이나 문서의 날짜 변경만으로
전체 테스트를 반복하지 않는다. 새 감사 판단과 검증 명령 재실행은 서로 다른 활동이다.

- 관련 입력·근거 계약이 유지되거나 변경이 해당 seam에 무영향임을 직접 확인할 수 있으면 재사용한다.
- production source/data/config가 바뀌면 test 파일이 같아도 영향받은 seam을 재실행한다. test/fixture/probe가
  바뀌어도 해당 seam을 current target에서 non-zero로 다시 실행한다.
- shared schema/harness/persistence/transport/dependency의 영향 분리가 불가능하면 근거 없이 재사용하지 않고
  필요한 넓은 범위를 다시 검증한다. 불확실성을 이유로 무조건 모든 무관한 테스트를 반복하지도 않는다.
- 실패·zero/불완전 실행·검증 뒤 relevant input 변경은 성공 baseline으로 세지 않는다. 반복 횟수 목표를
  맞추기 위해 필요한 재검증을 생략하지 않는다.
- final full baseline은 계약상 마지막 owning executable Single/Phase Goal이 모든 구현·focused 재검증 후의
  integrated target에서 수행한다. Single이면 같은 Goal이 initial/final 역할을 갖는다. Master는 ownership을
  대신하지 않고 그 근거를 감사한다. 적용 계약상 final owner가 없거나 소유 변경이 필요하면 해당 P2G owner로 반환한다.
- 후속 correction이 final baseline을 invalid 처리하면 그 owner의 검증 전용 GEX-10과 필요한 fresh audit를
  정당한 진입 순서에서 수행한다. 영향받은 선행 감사는 먼저 복원하며 아직 성립하지 않는 선행을 current로
  취급해 final owner를 실행하지 않는다. 검증 전용 pass에 새 기능을 끼워 넣지 않는다. owner가 현재 root 밖이면
  자동 실행하지 않고 필요한 exact 대상·권한을 보고한다.
- mutation 전 initial baseline이 필수였으나 실제로 누락됐다면 mutation 후 실행을 과거 initial로 소급하지
  않는다. exact 과거 target의 유효한 기존 근거가 있는지 확인하고, 없으면 계약 위반·누락과 완료 영향 및 필요한
  owner를 기록한다. 새 기록 생성이나 current 테스트 통과만으로 누락된 과거 조건을 충족시켰다고 하지 않는다.

정상적인 initial 1회/final 1회는 적용되는 계약의 실행 계획이지 오류 복구·새 입력 재검증의 상한이 아니다.
해당 계약이 없는 Goal에는 횟수·owner 규칙을 새로 강제하지 않고, 원래 완료 조건과 위험에 비례한 검증을 적용한다.
재사용한 실행은 원래 시점·결과를 유지하고 이번 pass에는 currentness 판단만 기록한다. 유효한 기존 근거의
재사용과 아직 실행하지 않은 테스트를 실행했다고 꾸미는 것은 다르다.

#### 공통 기계 검사와 의미 판단

해시·목록·부재·건수·exit·원시 출력 비교는 실제 도구로 집계한다. 도구의 범위/옵션·입력/결과 identity·실패와
누락·미실행을 보존하고, reviewer에게는 변경·불일치·실패 목록과 원문 locator 중심의 출력을 제공한다.
거대한 manifest와 모든 성공 로그를 매번 본문에 펼치거나 같은 숫자를 각 문서에서 독립 원본으로 재집계하지 않는다.

동일 exact 입력 집합의 공통 검사 결과는 §6.4의 현재성 요건을 만족하면 여러 감사 pass가 참조할 수 있다.
reviewer는 검사 방법·대상 coverage·환경·결과의 적용성을 독립 확인하며 도구의 `passed`를 의미적 PASS로
대체하지 않는다. 기존 구현자 집계도 검증 가능한 근거일 뿐 무조건 신뢰하지 않는다. 불일치·변경·근거 부족 또는
명시적 독립 재실행 요구가 있으면 관련 검사를 다시 수행한다. 원시 결과와 모든 실패는 계속 열람 가능해야 한다.

감사 시작·종료의 target currentness 확인은 유지한다. 직전 다른 감사의 entry 확인만으로 이후 시간의 무변경을
추정하지 않는다. 공통 immutable 근거는 검증된 결과를 참조하되 mutable source/환경·새 파일 membership·부재 등
달라질 수 있는 범위는 다시 비교한다. 완전한 영향 분리가 불가능하면 해당 고정 target의 전체 비교를 수행한다.
기계 검사 비용과 AI 입력 토큰은 구별하며, 출력 축약을 실제 검사 생략이나 해시 일치의 조작으로 구현하지 않는다.

#### 로컬 명령과 감사 부작용

명령 이름이 test/build라는 이유만으로 안전하다고 가정하지 않는다. 처음 실행하거나 관련 환경이 바뀌면 필요한
범위에서 fixture·DB/서비스·네트워크·출력 경로·주요 side effect·예산을 확인한다. 승인된 격리 환경에서 같은
명령의 실행·교정·재검증을 반복할 때는 이 확인과 사용자 허락을 의식처럼 반복하지 않는다.

감사의 read-only는 파일을 전혀 만들 수 없다는 뜻이 아니다. 분리된 build output·일회성 probe·감사 evidence는
허용하되 감사 대상, 미승인 사용자 데이터·외부 상태를 수정하지 않는다. source를 재생성·포맷·golden 갱신하는
명령은 감사에서 실행하지 않고 mutation owner로 반환한다. 정리는 자신이 만든 임시 부산물만 대상으로 한다.

#### Implementation Quality Guardrails

다음 방식은 test가 통과하더라도 승인된 구현이나 완료 증거로 인정하지 않는다.

- 특정 test name, fixture identity, mock/probe 여부 또는 test 전용 값에 따라 production behavior가 달라지는
  분기와 fixture/mock에만 필요한 production special case
- invalid data/state/command를 명시적 계약 없이 조용히 보정하거나, broad catch-all·silent recovery로 contract
  failure를 성공처럼 바꾸는 처리
- authoritative Core/data contract를 consumer나 presentation 계층이 중복 재추론하거나 별도 authority로
  유지하는 구현
- 현재 승인 scope의 live producer/consumer가 요구하지 않는 미래 전용 abstraction, registry, compatibility
  layer 또는 extension point
- test expectation에 맞추기 위해 사용자-visible 계약, strict validation, 원자성, ordering 또는 오류 의미를
  약화하는 수정
- arbitrary timestamp, sequence, retry count 또는 delay를 증가시켜 identity/ordering/state-transition 결함을
  우회하는 수정
- 의미와 source-of-truth를 확인하지 않은 snapshot/golden 갱신, 실패 test 비활성화·ignored 처리 또는 assertion
  삭제
- superseded runtime/data/schema/DTO/mock/test/document path, duplicate authority와 승인되지 않은 fallback을
  current path와 함께 남겨 completion을 주장하는 것

Test helper나 fixture 내부의 test-only 처리는 production behavior를 바꾸지 않고 실제 계약을 충실히 관찰하는
경우 허용된다. Testability를 위한 production seam이 필요하면 current live path에도 유효한 명시적 경계여야
하며, test 전용 우회가 아니라는 source-level 근거와 consumer를 기록한다.

공통 콘텐츠 작성 계약·descriptor·팩 로더·headless 검증 포트 자체가 승인된 Goal의 결과라면 최종 UI가 없다는
이유만으로 미래 전용 abstraction으로 배제하지 않는다. 이번 범위의 실제 producer/consumer 또는 최소 실제
RON·공개 시험 포트가 같은 production 계약을 관찰하고, 직접적인 완료 조건과 검증 근거가 있어야 한다.
아직 없는 후속 API를 현재 consumer로 꾸미거나 mock/fixture 전용 실행 분기를 두는 것은 여전히 금지한다.

#### Validation Dimension 적용

Goal Quality Contract Version `2`인 Goal은 P2G-50의 Validation Dimension Matrix를 실행 계약으로 사용한다.

- 각 `required` dimension은 current source/data/consumer evidence와 non-zero validation 또는 durable read-only
  seam에 연결한다. 모든 dimension을 별도 test command로 나눌 필요는 없다.
- `N/A`는 그대로 신뢰하지 않고 current actual에서 applicability를 다시 확인한다. 실제로 적용되는데 근거가
  없거나 happy path만으로 대체됐다면 GEX-10을 종료하지 않는다.
- success test 하나나 broad suite 이름만으로 invalid input, rollback, retry/idempotency, ordering,
  persistence, real producer-consumer 또는 legacy absence를 증명하지 않는다.
- 실행 조사에서 새로 applicable해진 dimension이 승인된 Goal 의미 안의 기술 세부사항이면 execution
  `PLAN.md`와 evidence map을 보완한다. 새 사용자-visible 의미가 필요하면 P2G-10, durable seam owner나 Phase
  경계가 필요하면 P2G-40, Goal mapping만 누락됐으면 P2G-50으로 반환한다.

Version `2` 이전의 accepted Goal Set에도 Implementation Quality Guardrails는 적용한다. 다만 immutable
historical Goal member에 새 matrix를 소급 작성하지 않고, 기존 completion condition과 current actual에서
applicable dimension을 execution `PLAN.md`에 기록한다.

#### 명시적으로 요청한 단계검토

Reader Contract의 opt-in일 때만 아래 중간 검토를 적용한다. 기본 실행의 내부 작업 분할·수시 빌드·자체 테스트는
계속 수행하되 그것을 사용자 검토 대기로 바꾸지 않는다. 실행 PLAN·EXPERIMENTS·NOTES의 기존 역할을 사용하며
새 전역 mode, P2G artifact schema, 필수 checkpoint 수 또는 WP별 독립 컴파일 조건을 만들지 않는다.

1. 구현 전에 PLAN에 각 checkpoint의 포함 작업·변경 경계, 선행/남은 의존, 빌드·관련 검증 명령과 통과 조건을
   정한다. WP 번호가 아니라 actual producer/consumer·schema/store·공유 타입의 결합을 따른다. WP 일부를 묶거나
   여러 WP를 한 checkpoint로 묶을 수 있지만 Atomic Group을 독립 Phase로 쪼개거나 필수 선행 감사를 생략하지 않는다.
   Goal 내부 구현 순서만 바꾸면 기술 계획으로 기록하고, 정식 Phase/Goal·정책 경계 변경은 P2G owner로 반환한다.
2. checkpoint는 해당 target이 실제 빌드 가능하고 관련 검증을 통과하며, 아직 미구현인 동작과 잔여 의존을
   명시한 지점이다. 컴파일이 없는 문서/데이터 작업은 그 성격에 맞는 구조·정합성 검증을 사용한다. 단순 문법
   검사나 파일 존재만으로 실행 검증을 대신하지 않는다. 컴파일 실패·필수 테스트 실패를 `검토 준비 완료`로 표시하지 않는다.
   해결 가능한 실패는 승인 범위에서 교정·재검증하고 실제 blocker이면 실패 상태와 마지막 유효 기준선을 보고한다.
3. 안전한 중간 경계가 없으면 임의의 stub·가짜 PASS·임시 기능 무력화로 만들지 않는다. 이미 승인된 전환 전략의
   호환 계층은 종료/제거 조건과 검증을 갖춘 경우에만 사용한다. 최소 결합 범위를 묶은 이유를 기록하며, 그 범위가
   Goal 전체라면 중간 checkpoint를 만들 수 없음을 알리고 최종 구현 인계만 둔다. 독립 완료 가능하다고 꾸미지 않는다.
4. mutation 전과 검토 인계 후의 identity는 아래 **구현 전후 고정**을 따른다. 검토 패키지에는 exact Goal·checkpoint,
   검토 대상 요구/설계 locator, 직전 승인 snapshot 대비 delta와 최초 baseline 대비 누적 delta, 테스트 명령·exit·출력,
   미완료·제약·미해결 finding·다음 후보/진입 조건을 연결한다. 첫 checkpoint의 비교 기준은 최초 구현 baseline이다.
   검토가 없는 초기 baseline을 승인된 구현으로 부르지 않는다.
5. 해당 snapshot을 고정하고 `부분 구현·검증 종료 / 단계검토 요청 대기`로 최종 응답 후 종료한다. 전체 exit를
   충족하기 전에는 GEX-10 전체 종료나 Goal `Complete`를 선언하지 않는다. 자동 재개와 독립 작업으로 경계를 우회하지 않는다.
6. 별도 요청을 받은 reviewer는 고정된 요구·전후 delta·관련 코드를 읽기 전용으로 검토하고 실제 시험 근거를
   확인한다. 진입/종료 시 source identity를 확인하고, 대상 변경이 있으면 범위와 영향에 따라 재고정·재검토한다.
   단계별 검토 기록은 exact snapshot에 귀속하고 감사 target 밖의 evidence에 쓴다. 부분 범위 PASS/수정 필요/판정 불가와
   findings를 남기되 GEX-20의 `Complete / High`를 발행하지 않는다. 모델 성능만으로 독립성이 충족된 것으로 보지 않는다.
7. 수정 요구가 있으면 별도 승인된 GEX-10 교정에서 새 snapshot·검증을 만들고 재검토 요청을 기다린다. 과거 snapshot과
   검토 기록을 덮어쓰지 않는다. 검토가 끝나도 다음 구현은 새 사용자 요청을 기다리며, 재개 시 current tree·미해결 finding·
   선행·권한을 확인한다. 뒤 단계 변경이 과거 검토/시험 근거를 무효화하면 영향 범위를 명시하고 다시 검증·검토한다.
8. 최종 구현 뒤에는 전체 조건의 Audit Target Record를 고정하고 기본 감사 인계 경계를 따른다. 중간 검토의 유효한
   증거를 재사용할 수 있어도 최종 GEX-20, 필요한 Phase 선행 감사와 GEX-30을 면제하지 않는다. 최종 독립성은 §6.7을 따른다.

#### 구현 전후 고정

기본 실행과 opt-in 모두, 최초 mutation 전의 구현 baseline과 최종 구현 후 target을 구분하여 보존한다.
계획 종료 baseline은 비교 입력이며 실제 mutation 전 baseline을 대신하지 않는다. opt-in은 각 구간의 전후도 고정한다.
§5의 Audit Target Record/currentness 방식을 재사용하되 부분 snapshot을 Goal 전체의 완료 감사 대상으로 오인하지 않는다.

- repository/worktree·branch·HEAD와 관련 dependency revision, scoped tracked/dirty/untracked/generated 및 추가·삭제·이동을 기록한다.
- 영향 source·data·설계/계약·테스트·설정의 경로별 SHA-256 또는 동등한 identity와 포함/제외 범위를 남긴다.
  clean tracked는 복원 가능한 commit/path로 식별할 수 있지만 dirty·untracked는 HEAD만으로 고정되지 않는다.
- 해시는 내용 복원 수단이 아니다. 실제 원본 snapshot 또는 접근 가능한 Git 객체와 적용 가능한 binary 포함 diff로
  전후 내용을 재구성할 수 있어야 한다. 미추적·삭제 파일·기존 사용자 변경도 손실 없이 구분해 보존한다.
  commit 생성은 필수가 아니며 기존 사용자 변경을 자동 commit하거나 reset하지 않는다.
- 명령·exit·출력·fixture/oracle·도구/환경·실행/재사용 여부를 해당 target에 연결한다. 근거 파일도 identity를 남기고
  source target 밖에 둔다. manifest 자신이나 후속 review 결과를 자기 해시 대상에 넣는 순환 참조를 만들지 않는다.
- 재개·검토 전후의 drift를 대조한다. 예상된 구현 delta와 외부 변경을 구분하고 영향받은 근거만 갱신한다.
  복원 자료/필수 검증이 없으면 인계 준비 완료를 주장하지 않고 정확한 누락과 필요한 다음 작업을 보고한다.

### 6.5 새 질문과 결함

GEX-10 안의 일반적인 기술 실패는 선택된 Goal 의미와 범위를 유지하는 한 AI가 조사·수정·재검증할 수 있다.
하지만 다음 경계를 만나면 임의로 해결하지 않는다.

- 새로운 material policy 또는 acceptance 의미가 필요함
- current code evidence가 Final Freeze나 implementation boundary를 무효화함
- Work Package, Atomic Group, Phase dependency 또는 Goal mapping을 바꿔야 함
- required external consumer/evidence가 없어 완료 조건을 증명할 수 없음
- Goal 밖의 독립 작업을 포함해야만 구현 가능함

이 경우 발견 사실, 이미 수행한 안전한 작업, 영향 범위와 필요한 owner 단계를 기록하고 현재 GEX 단계를
완료하지 않는다. **owner 반환과 사용자 답변 대기는 별개다.** 새 정책이면
[P2G §8.6](./policy_to_goal_pipeline_revised_v2.md#86-blocker와-중단-조건)의 다섯 조건과 실제 위임을 확인한다.
통상적 정책은 기존 기록에 판단 근거를 남기고, 상위 실행 요청이 허용하는 P2G-10 pass에서 exact 결정을
반영한다. 필요한 successor·검토·동결·인계를 마친 뒤 current basis로 GEX를 재개한다. P2G를 포함하지 않는
실행 요청이면 필요한 좁은 권한을 요청하며, GEX 자체에서 동결된 정책·Goal을 고치지 않는다. 사용자 선택이
필요한 정책은 section 6.6에 따라 질문 후 대기한다. 분류에 필요한 사실이 부족하면 현재 권한 안에서 먼저
조사하고, 해소 불가능한 필수 근거 문제는 `evidence_gap`으로 남긴다.

### 6.6 Goal-run 자동 전환과 중단

Reader Contract의 기본 감사 인계·선택적 단계검토 checkpoint와 현재 권한을 모든 재개·교정·자동 전환보다
먼저 확인한다. 아래 표는 그 경계를 통과할 권한이 있을 때만 적용한다. 검토 대기 중에는 독립 Phase도 시작하지 않는다.

active `goal-run`은 각 GEX pass의 결과·근거를 기록한 뒤 필요한 다음 pass를 선택한다. 성공 경로는 해당 exit를
충족해야 하며, 실패 뒤 correction은 아래 명시된 조건을 따라 실패 기록을 보존한 별도 pass로 수행한다.
이 절은 Phase Plan을 변경하거나 새로운 Goal을 승인하지 않는다.

#### 정책 의논 시 주 작업 대기와 실행 중인 독립 작업

section 6.5의 판정으로 사용자와 의논이 필요한 정책이 발생하면 **주 작업 담당자는 진행을 임시중단하고, 일반 채팅의 최종 답변으로
질문한 뒤 사용자 답변을 기다린다.** 이미 실행 중인 작업은 아래 조건에 따라 처리한다. 이 규칙은 연속 실행·
자동 전환·독립 작업 계속 진행 조항보다 우선하며, 독립 작업의 존재가 주 작업의 계속 진행 근거가 되지 않는다.

| 대상 | 답변 대기 중 처리 |
| --- | --- |
| 주 작업 담당자 | 추가 조사·계획·구현·빌드·테스트·검증·감사와 후속 진행을 멈추고 질문 후 대기 |
| 정책 답변에 의존하는 실행 중 작업·하위 agent·백그라운드 작업 | 안전하게 임시중단하고 마지막 유효 상태 보존 |
| 정책 의존성이 불분명한 실행 중 작업 | 독립으로 추정하지 않고 안전하게 임시중단 |
| 이미 실행 중이며 독립성이 확인된 작업·하위 agent·백그라운드 작업 | 질문 전에 배정받아 실행 중이던 범위까지만 완료 가능 |
| 새 작업 시작·새 위임·후속 단계 또는 다른 Goal/Topic/로드맵으로 전환 | 독립 여부와 관계없이 사용자 답변 전에는 수행하지 않음 |

독립성이 확인되려면 **어느 정책 답변이 선택돼도 해당 작업의 요구사항·완료 기준·결과의 유효성이 유지되고,
보류 작업과 파일·공유 상태의 충돌이 없어야 한다.** 이미 확보한 근거와 배정 범위로 판단하고, 확인할 수 없으면
의존성 불분명으로 처리한다. 이미 배정됐어도 아직 실행하지 않은 작업은 새로 시작하지 않는다. 계속 실행하던
작업에서도 정책 의존성·충돌 또는 그 불확실성이 드러나면 안전하게 멈춘다.

- 주 작업 담당자는 안전한 중단·기존 작업의 범위 제한 전달·마지막 유효 상태와 재개 지점 기록·이미 확보한
  근거로 질문을 작성하는 최소 정리만 한다. 질문 준비나 독립성 확인을 이유로 새 조사·실험을 시작하지 않는다.
- 질문은 **일반 채팅의 최종 답변 본문**에 정책·의논이 필요한 이유·근거·선택지와 영향·권고를 명시하고,
  주 작업이 대기함과 계속 실행하는 기존 독립 작업이 있으면 그 범위를 알린 뒤 응답을 끝낸다. 질문 카드나
  비동기 입력 도구만 띄운 채 주 작업을 계속하지 않는다. 독립 작업의 완료를 기다리느라 질문 전달을 늦추지 않는다.
- 기존 독립 작업은 배정 범위의 결과를 보존하고 종료한다. 주 작업 담당자는 완료 통지를 받아도 결과 보존 외의
  추가 검토·통합·후속 작업 배정을 하지 않고 계속 대기한다. 완료 통지는 재개 근거가 아니다.
- 무응답·기본 선택·경과 시간은 승인이 아니다. 답변이 정책을 해소하지 못하면 의논과 대기 상태를 유지한다.
  정책이 해소되면 실제 policy owner에 반영하고 기존 승인 범위에서 재개한다.
- 중단한 실행을 성공으로 기록하거나 미완료 Goal을 `complete`로 표시하지 않는다. 실행 시스템에서 실제로
  적용하지 않은 `paused` 상태를 주장하지 않는다.

#### 재개 시 다음 pass

GEX-00은 section 5의 기존 기록을 인수한 뒤 필요한 current actual을 확인하고 다음을 판정한다.

| 확인한 현재 상태 | 같은 승인 대상 안의 다음 동작 |
| --- | --- |
| root Single/Phase 또는 Master가 current `Complete / High`이고 필요한 basis도 유효함 | 중복 구현·감사 없이 현재 완료 근거 보고. 사용자가 fresh audit를 명시하면 그 요청을 따름 |
| 신규 Goal 또는 미완료 구현, 정상 선행 조건 충족 | selected Goal의 GEX-10 |
| 구현 종료 조건과 안정된 target이 확인되고 감사만 남음 | GEX-10을 다시 하지 않고 fresh GEX-20 |
| 이전 audit만 stale/근거 보충 필요, source correction은 불필요하고 section 5의 재감사 조건 충족 | current target의 fresh GEX-20; Master이면 GEX-30의 실제 진입 조건도 확인 |
| 이미 수행한 변경의 기술 결함과 교정 owner가 명확함 | 정상 진입 또는 아래 제한적 교정 조건으로 GEX-10 |
| Goal Set의 모든 Phase audit가 current이고 Master 감사만 남음 | 새 implementation target을 만들지 않고 fresh GEX-30 |
| 일부 Phase만 완료됨 | 재구현하지 않고 dependency와 현재 basis상 eligible한 member를 선택 |
| 기록·actual·권한을 일치시킬 수 없음 | 필요한 제한적 확인 후에도 불명확하면 중단 |

`stage-run`의 GEX-00은 이 판정과 다음 pass를 보고할 뿐 다른 단계를 실행하지 않는다. 위 표를 적용했다고
새 감사 verdict를 만든 것은 아니다. 이미 완료된 목표에 대한 현황 보고에도 currentness 확인 근거를 남긴다.

#### 이미 승인된 변경의 제한적 교정 재진입

**새로운 Phase 착수와, 이미 정당하게 인수·구현한 변경의 회귀 교정은 다르다.** 다음 예에서 선행 audit를
무조건 current로 요구하면 수리가 막힐 수 있다.

```text
A가 독립적으로 완료·감사됨 → B가 정상 prerequisite로 인수되어 변경됨
→ 그 변경이 A의 감사/검증을 무효화하거나 A 재감사에서 회귀가 확인됨
→ 책임 변경이 B에 있으므로 B를 교정 → A와 B의 영향받은 검증·감사를 복원
```

아래 조건을 **모두** 확인하면, 무효화된 선행 audit 때문에 정상 재진입할 수 없는 경우에도 기존 owner의
교정 GEX-10을 시작할 수 있다. 이는 새 stage·Goal·approval ID가 아니라 해당 pass의 제한된 진입 근거다.

1. accepted Handoff·Final Freeze·Phase Plan·Goal의 exact 의미와 경계는 여전히 유효하다. 신규 기능·Phase 순서·
   dependency·완료 조건을 바꾸는 작업이 아니다.
2. 교정 owner는 이미 정상 GEX-00을 거쳐 승인 범위에서 mutation을 수행한 exact Goal이다. 정상 인수 당시의
   선행 감사와 scope, 책임 delta·실패/무효화 조건을 직접 근거로 연결할 수 있다. 처음부터 누락된 선행이나
   원래 성립하지 않는 Phase 분할을 이 예외로 통과시키지 않는다.
3. 교정할 정확한 범위와 영향받은 audit를 식별하고 다른 미해결 선행·정책·권한·환경·dirty 문제와 분리할 수
   있다. 역사적 감사가 있다는 이유만으로 현재의 다른 결함을 무시하지 않는다. 귀속·영향이 unknown이면 중단한다.
4. 현재 요청이 교정 owner의 mutation을 허용한다. Goal Set `goal-run`은 그 안의 owner만 포함하며,
   Single/Phase 요청은 다른 member까지 확대되지 않는다. 교정과 필수 후속 재감사 중 root 밖의 대상은 별도
   명시적 권한을 확인한다. 승인 없는 외부 Goal까지 실행하지 않는다.
5. 교정은 승인된 동작·불변식을 복원하는 최소 범위이며, 고장 난 선행 동작을 안전한 것으로 믿고 비가역적인
   데이터/환경 변경을 수행하지 않는다. 작업·dirty·실행 환경을 안전하게 격리할 수 있어야 한다.
6. PLAN 또는 동등한 기존 기록에 책임 Goal, 귀속 delta, 현재 쓸 수 없는 선행 감사/조건, 다른 선행의 유효 근거,
   허용 교정 범위, 재검증·재감사 대상과 복원 순서를 기록한다. 별도 예외 승인 장부는 만들지 않는다.

이 진입은 해당 교정만 허용한다. stale 또는 `Not Complete` audit는 계속 미충족이며 신규 dependent Phase와
Master 완료는 보류한다. 필요한 인수/기록 재개도 같은 제한 아래 GEX-00에서 할 수 있지만, `정상 선행 충족`으로
기록하지 않는다. 직전 감사가 실패한 A라고 해서 A 코드를 고치는 것이 아니라 실제 책임 delta의 owner를 고친다.

교정 뒤에는 §5의 조건별 영향 판정에 따라 영향받은 focused validation, 필요한 baseline과 모든 relevant
audit를 복원한다. 단지 predecessor 보고서가 새로 발행됐다는 이유로 무관한 successor 전체를 재감사하지 않는다.
관련 보고서 갱신이 실제 선행 판정·수용 근거를 바꿨는지는 별도로 확인하며, 기존 미해결/실패를 건너뛰지 않는다. 구현을 멈춘
안정된 target에서 선행 의미를 확인하는 fresh audit부터 실제 dependency 순서로 재검토한다. 다른 affected
후행 audit가 아직 미복원이라는 이유만으로 그 복원을 위한 선행 read-only audit를 막지 않는다. 반대로 독립적인
필수 선행 조건이 실패한 audit를 긍정으로 닫지는 않는다. 검증 중 새 source defect를 찾으면 다시 mutation을
분리하여 owner의 GEX-10으로 반환한다. 모든 required predecessor·selected Goal·baseline이 current가 된 뒤에만
정상 진행/완료로 복귀한다. 기존 audit record/verdict는 수정하거나 성공으로 덮어쓰지 않는다.

`stage-run`도 사용자가 해당 owner의 correction GEX-10을 명시적으로 요청하면 위 조건을 사용할 수 있지만,
그 요청은 재감사까지 자동 승인하지 않는다. 현재 root가 필요한 수리·재감사 전체를 포함하지 않는다면 허용된
안전한 결과까지만 기록하고 미승인 범위는 질문한다. 동시 전환이 본래 필수였다면 P2G-40으로 반환한다.

#### Finding 영향과 진행 우선순위

먼저 정책·upstream 무효화·권한·격리·진짜 Phase 경계 문제인지 판정한다. 이는 아래 교정으로 우회할 수 없다.
그 다음 normal in-scope correction 또는 위 제한적 교정이 가능한 **이미 귀속된 기술 회귀**를 처리한다.
교정 중에는 무효화된 completion을 사용한 정상 후속 진행을 하지 않는다. 이를 단지 여러 audit가 영향받았다는
이유로 무조건 수리 불가능한 전역 blocker로 바꾸지 않는다.

안전하게 수리할 수 없어 남은 `Not Complete`/`Blocked` finding은 다음과 같이 분류한다.

- `Phase-local`: selected Phase와 그 전이적 dependent 범위만 침범한다. 그 밖의 독립 Phase의 계약·환경·권한·
  dirty 격리·검증에 영향이 없음을 직접 확인한다. A와 A에 의존하는 B가 함께 막혔다는 사실만으로 전역은 아니다.
- `run-global`: 위 의존 범위를 넘어 공통 basis·실행 환경·권한·격리 또는 독립 Phase의 prerequisite/완료에도
  영향을 주거나, 영향 범위를 신뢰성 있게 제한할 수 없다. 여러 Phase라는 숫자가 아니라 실제 영향 경계로 판단한다.

영향이 unknown이면 `run-global`로 취급한다. 분류와 근거는 기존 notes/audit에 기록하며 새 blocker registry를
만들지 않는다. 안전하게 진행할 수 있는 independent Phase는 Goal Set `goal-run` 안에서만 아래 조건으로 선택한다.

#### 자동 전환

서로 겹치는 경우 정책/권한/격리의 중단 조건을 먼저, 허용된 기술 교정을 그 다음, 일반 성공·독립 진행을 마지막에
평가한다. 일부 검증이 실패했다는 이유만으로 첫 수정 시도 전에 무조건 중단하지 않는다.

| current pass 결과 | goal-run의 다음 동작 |
| --- | --- |
| opt-in 중간 checkpoint 충족 | snapshot·검증·남은 작업 고정 후 최종 응답으로 종료, 별도 단계검토 요청 대기 |
| GEX-10 종료 조건 충족, 기본 최종 인계 경계 적용 | Audit Target Record 고정 후 종료, 별도 감사 요청 대기 |
| GEX-00 인수 종료 | 위 재개표의 필요한 pass 또는 current 완료 보고. 항상 GEX-10으로 보내지 않음 |
| GEX-10 종료 조건 충족, 감사까지 명시 위임·미충족 checkpoint 없음 | mutation을 멈추고 필요한 predecessor fresh audit·검증 복원 후 selected Goal의 fresh GEX-20 |
| 책임 owner가 확정된 승인 범위 기술 결함, 정상 또는 제한적 교정 조건 충족 | 발견 pass를 닫고 exact owner의 correction GEX-10 → 영향 검증/감사 복원 |
| expected delta로 predecessor audit만 stale, source correction 불필요 | root/명시 권한 안의 fresh GEX-20 → 중단됐던 pass 재개 |
| GEX-20 current Complete / High, Single/Phase target | 그 target terminal completion 보고 |
| GEX-20 current Complete / High, Goal Set에 eligible 미완료 Phase 존재 | 그 Phase의 GEX-00 |
| Goal Set의 모든 Phase current 완료, required final baseline 준비됨 | fresh GEX-30 |
| GEX-30 Not Complete, exact owner 범위의 기술 결함 | owner의 correction GEX-10 → relevant GEX-20·final baseline 복원 → fresh GEX-30 |
| 수리 불가 Phase-local finding, Goal Set에 독립 eligible Phase 존재 | 현재 Phase와 dependent 보류 → 독립 Phase GEX-00 |
| Single/Phase target 또는 진행할 독립 Phase 없음 | 실제 Not Complete/Blocked terminal result 보고 |
| run-global Blocked, P2G owner 또는 미승인 권한 필요 | current run 중단, 영향과 필요한 owner/사용자 결정 보고 |
| GEX-30 current Complete / High | Goal Set terminal completion 보고 |

독립 eligible Phase가 여러 개면 Phase Plan의 명시 순서를, 없으면 선언 순서를 직렬 scheduling tie-break로
사용한다. 순서가 observable result·ownership을 바꾸거나 dirty collision을 만들면 임의 선택하지 않는다.
독립 Phase를 새로 시작하거나 후속 pass로 전환하려면 사용자 정책 답변 대기 상태가 아니어야 한다. 대기 중
이미 실행 중인 독립 작업의 기존 범위 완료는 위 정책 의논 규칙만을 따른다. 어느 경우든 원래 dependency상
eligible하며 blocked mutation/evidence와 격리되고 shared basis와 predecessor audit가 유효해야 한다.
독립 진행은 blocked Phase를 완료하거나 Master prerequisite를 충족시키지 않는다.

selected Phase가 애초에 dependent Phase의 mutation 없이는 유효한 중간 상태와 독립 완료를 만들 수 없었다면
Phase/Atomic Group 경계 문제다. 미래 Phase를 조기 실행하거나 교정 예외로 숨기지 않고 P2G-40으로 반환한다.

다음 상황에서는 active goal-run도 자동 진행을 중단한다.

- section 6.5의 정책 owner 반환이 필요하거나 scope/Phase/Goal 구조를 바꿔야 함. 통상적 정책은 상위 요청이
  허용한 owner pass에서 결정·수리 후 재개하며, 사용자 선택 또는 미승인 범위·권한이 필요한 경우만 답변 대기
- exact P2G upstream basis가 stale/모순됨. 위 교정 대상인 실행 audit 무효화와 구분함
- required external 근거·배포/운영 권한이 없거나, 안전하게 분리할 수 없는 환경·기존 dirty 문제가 있음
- 책임 delta, root target 또는 허용 execution target을 신뢰성 있게 정할 수 없음
- 기본 감사 인계 또는 명시적 사용자 checkpoint에 도달함, 또는 필수 독립 reviewer를 확보할 수 없음
- 같은 실패를 반복할 뿐 안전하고 의미 있는 진전이 없거나 현재 실행 자원이 소진됨

중단 시 마지막 유효 결과·실패 원인·affected scope·필요한 owner와 입력을 기록한다. 권한을 확대하거나
다른 pipeline의 owner를 대신하지 않는다. 승인 범위 안의 owner 처리는 별도 pass로 수행하며, 새 사용자
결정·권한이 필요한 경우에는 실제 입력을 받은 뒤 범위·basis를 확인해 필요한 지점부터 재개한다.

### 6.7 감사 분리

GEX-20과 GEX-30은 구현이 끝나고 감사 대상이 안정된 뒤 시작하는 fresh pass다. `stage-run`에서는 새 사용자
요청이 필요하고, active `goal-run`에서는 감사까지 명시 위임되고 Reader Contract의 대기 경계가 없을 때만
같은 상위 요청 아래 자동 시작할 수 있다. 중간 단계검토는 §6.4의 부분 검토이며 GEX-20/30을 대체하지 않는다. fresh pass는 새 사용자
요청 자체가 아니라 구현 pass와 작업 문맥, mutation 권한, 산출물과 감사 기록이 분리된 별도 검토 pass를
뜻한다. 감사자는 Goal 문서의 완료 주장을 그대로 신뢰하지 않고 current code/data/document와 validation
evidence를 다시 확인한다. fresh review는 모든 명령의 재실행과 동의어가 아니다. section 6.4에 따라 근거의
유효성을 독립적으로 판단하고, 영향받았거나 부족한 검증만 필요한 깊이로 실제 재실행한다.

감사 pass에서는 source, production code/data, audit target 또는 candidate를 수정하지 않는다. 테스트나
read-only probe가 생성한 임시 산출물은 증거로 분류하고 자신이 만든 임시 부산물만 정리할 수 있다. 결함을
발견하면 verdict와 반환 단계를 기록하고 수정은 후속 GEX-10 pass에서 수행한다.

`COMPLETION_AUDIT.md`와 감사 evidence 자체를 작성하는 것은 허용된다. 이 감사 문서 변경은 implementation
audit target과 분리해 귀속하며, 감사 중 발견한 source/data 결함을 함께 고치는 수단으로 사용하지 않는다.
PLAN·notes·ROADMAP_STATUS 같은 요약 문서가 audit target에 포함되어 있다면 감사 중 직접 수정하지 않는다.
감사 결과는 허용된 audit evidence에 먼저 기록하고, target에 영향을 주는 요약 갱신은 별도 허용된 기록 작업에서
수행한 뒤 currentness 영향을 판정한다. 현황 표시의 갱신을 구현 완료나 새 권한으로 간주하지 않는다.

다른 사람이나 별도 agent가 항상 필요한 것은 아니다. 그러나 Goal, 사용자 또는 risk 근거가 독립 reviewer를
요구하면 구현에 참여하지 않은 reviewer가 감사해야 한다. 같은 실행자가 감사할 수 있는 경우에도 구현 pass와
시간·작업 문맥·산출물이 분리된 fresh review여야 한다.

동일한 구현 비참여 reviewer가 여러 관련 Goal 또는 재감사를 맡을 수 있다. 매번 새 agent·빈 문맥을 만드는 것은
기본 의무가 아니다. 단, 사용자/각 Goal의 독립성·위험·blind review 요건에 어긋나면 이 방법을 쓰지 않는다.
구현 방법 선택·코드 교정에 참여한 보조자는 해당 구현의 독립 감사자가 될 수 없다. 감사 finding과 재검증 요구를
남기는 본래 감사 활동만으로 구현자가 되지는 않지만, 직접 설계/수리에 관여했는지는 구체적으로 판정한다.

각 pass의 exact Goal/target·권한·진입/종료·finding/verdict는 계속 분리한다. 공통 요구·도구 결과와 locator는
재사용하되 다른 Goal의 결론으로 현재 Goal을 승인하지 않는다. 새 reviewer에게는 해당 Goal의 원문 기준,
누적 delta·조건별 영향, 최신 유효 증거·미해결 finding·현재성 비교를 제공한다. 공통 요약은 탐색 입구이지
source of truth가 아니며 추가 원문 검토를 금지하거나 긍정 결론을 미리 정해 주지 않는다.

### 6.8 완료 verdict

긍정 완료 verdict는 `Complete / High`다.

- `Complete`: 모든 applicable completion condition과 승인된 policy/Goal 의미가 구현됨
- `High`: 각 완료 주장에 current direct evidence가 있고 completion-blocking unknown이 없음

고정된 점수표나 다단계 Fit enum을 요구하지 않는다. 하나라도 필수 조건이 실패하면 `Not Complete`, 필요한
권한·정책·외부 증거를 얻을 수 없으면 `Blocked`로 기록한다. `Not Complete`나 `Blocked`는 해당 Phase의
dependent Phase 또는 Master 완료를 허용하지 않는다. 다만 active goal-run의 in-scope 기술 결함은 section
6.6에 따라 correction pass로 반환할 수 있고, shared basis가 보존된 독립 eligible Phase는 같은 section의
조건에 따라 진행할 수 있다.

기술 부채나 비필수 개선점이 남아도 현재 Goal의 correctness, authority, compatibility, consumer, validation
또는 유지해야 할 구조를 침범하지 않는다는 근거가 있을 때만 out-of-scope/follow-up으로 분리할 수 있다.

## 7. 단계별 계약

<!-- GEX-00 START -->
### GEX-00 — 실행 인수와 Single/Phase Goal 선택

#### 목적

P2G-60 Handoff와 현재 실행 기록을 인수하여 필요한 다음 pass를 정한다. 구현이 필요하면 Single/Phase Goal
하나를 선택한다. 이미 current 완료거나 모든 Phase가 끝나 Master 감사만 남은 재개에서는 구현 대상을 억지로
만들지 않는다. 현재 tree·dependency·dirty·검증 가능성을 확인하되 production implementation은 하지 않는다.

#### 입력

- accepted/current P2G-60 Handoff와 exact Final Policy Freeze, Phase Plan, Goal Set 및 Remaining Gap Summary
- 현재 요청의 `stage-run | goal-run`, root target, exact authorization·명시적 checkpoint
- 대상 member의 Phase/execution node, scope, dependency, quality/invariant, validation·completion·stop condition
- applicable 설계 인계, Shared Validation Baseline Reuse Contract와 initial/final owner·현재 evidence
- applicable Integration Validation Ownership Map과 Master relation
- 현재 workspace와 source/data/test/consumer 접근, 기존 실행 문서·역할 locator·직접 audit/history

#### 진입 조건

- 최초 실행은 current Handoff와 exact upstream chain을 요구함. 재개는 section 6.1의 동일한 accepted basis와
  현재 요청·적용 지침을 확인함. authoritative하게 stale인 정책/Plan/Goal을 단순 인수 기록으로 복원하지 않음
- 실행/감사할 member는 exact Goal Set에 존재하고 kind·담당 Phase·assigned execution node가 Phase Plan과
  일치해야 함. Master의 조정 대상을 implementation member로 바꾸지 않음
- 정상 신규/후속 구현 인수는 필요한 current `Complete / High` predecessor를 요구함
- 재개 판정을 위한 record/actual 확인은 predecessor가 stale인 경우에도 read-only로 가능함. 정상 구현 eligible을
  선언하지 않으며, 교정 인수는 section 6.6의 제한 조건을 모두 확인해야 함
- ambiguous identity, unsafe ownership 또는 필요한 기록 손실을 추측으로 덮지 않음

#### 주요 활동

1. current 요청의 root target과 exact Handoff/Goal Set을 확인한다. 기존 PLAN의 더 넓은 권한을 자동 승계하지 않는다.
2. section 5의 directory·역할 매핑·재개 선독을 먼저 적용한다. history가 없는 신규 set만 초기화하며, 동등한
   구조는 재사용하고 입증 가능한 초기화/요약 누락만 제한적으로 보완한다. 기록 손실은 성공으로 채우지 않는다.
3. 직접 predecessor·current audit·기존 pass 결과를 필요한 current actual과 대조한다. 관련 status/diff·환경·
   validation 접근을 확인하고 unrelated dirty를 보존한다. 완료/감사 대기/교정/신규를 구분한다.
4. 구현 target이 필요하면 exact Single/Phase 하나를 정한다. eligible 후보가 하나면 선택할 수 있다. 여러 개면
   `stage-run`은 필요한 선택을 묻고 Goal Set `goal-run`은 section 6.6 순서를 따른다. Master를 구현 owner로 쓰지 않는다.
5. 설계 인계·실제 선행 계약·integration seam/owner·baseline contract의 적용 여부와 준비 상태를 확인한다.
   없는 durable seam owner는 P2G-40, Plan에는 있지만 Goal mapping만 없는 것은 P2G-50으로 반환한다.
6. PLAN 역할의 current 절에 지침 revision, 요청·root/selected target, path/role, upstream, baseline·dirty,
   설계/검증·완료/중단, 다음 pass와 이유를 기록한다. 제한적 교정이면 귀속 delta·미충족 선행·허용 수리 범위와
   감사 복원 대상을 분리하고 정상 prerequisite 충족으로 표시하지 않는다.
7. `stage-run`은 인수 결과와 필요한 다음 pass를 보고하고 멈춘다. `goal-run`은 section 6.6의 재개표에 따라
   구현·감사·재감사·terminal 보고로 이어간다. 같은 구현을 무조건 다시 수행하지 않는다.

#### 산출물

- exact root target, 적용 지침·현재 요청 권한, 구현/감사할 member 또는 이미 완료된 대상
- execution directory와 역할별 locator, 인수·제한적 보완 근거
- current baseline·dirty·dependency/audit 판단, integration/baseline/design 인계 상태
- 다음 pass와 진입 근거 또는 필요한 blocker/owner. 새 감사 verdict가 아님

#### 종료 조건

- exact upstream과 대상·권한·다음 동작을 모호하지 않게 식별함
- 정상 구현, 제한적 교정, read-only 감사, current 완료 보고 중 해당 경로의 조건을 실제 근거로 판단함
- 필요한 quality/validation/N/A와 integration owner·설계 인계·baseline을 누락하거나 성공으로 추정하지 않음
- 기존 history를 보존하고 현재 상태를 복구 가능한 locator로 연결함
- 이 GEX-00 pass에서 production implementation이 발생하지 않음

#### 중단과 반환

Handoff/Goal/Phase 의미가 stale하거나 모순되면 section 8의 owner로 반환한다. 기록·dirty·runner·필수 근거 문제는
실제 영향 범위로 판정한다. 인수 보고 자체와 후속 실행 eligibility를 구분하고, 교정 예외가 성립하지 않는
prerequisite를 건너뛰지 않는다. 독립 Phase 진행은 Goal Set `goal-run`과 section 6.6의 조건에서만 가능하다.

<!-- GEX-00 END -->

<!-- GEX-10 START -->
### GEX-10 — 선택된 Goal 구현과 검증

#### 목적

GEX-00에서 선택한 Single/Phase Goal을 exact scope 안에서 end-to-end로 구현하고, 별도 완료 감사가 재검토할 수
있는 current implementation과 validation evidence를 만든다.

#### 입력

- 해당 exact Goal의 successful GEX-00 기록과 current PLAN. 교정 재개라면 기존 정상 인수 기록 및 현재
  section 6.6 교정 조건 판정; 매 correction마다 새 초기화 문서를 요구하지 않음
- exact Handoff, Goal Set, selected Goal member와 Phase
- current repository/workspace와 dirty baseline
- Goal의 implementation boundary, 권장 설계·상세화할 선택, validation·completion·stop conditions
- applicable Shared Validation Baseline Reuse Contract·owner와 현재 reusable/invalid evidence
- applicable Goal Quality Contract, Goal-local invariant/structural completion과 Validation Dimension Matrix
- 사용자의 direct GEX-10 `stage-run` 요청 또는 selected Goal을 포함하는 active `goal-run` authorization

#### 진입 조건

- selected Goal과 exact P2G upstream 의미가 유효하고 현재 요청이 이 owner의 mutation을 허용함
- 정상 신규/후속 실행은 Goal dependency와 execution prerequisite가 충족됨
- 제한적 correction은 section 6.6의 모든 조건과 허용 범위를 기록함. 해당 audit 무효화만 수리 대상으로 두며
  그 밖의 선행·정책·권한·환경·dirty 조건은 충족되어야 함
- Goal 범위와 기존 dirty work를 안전하게 구분할 수 있고, 교정 대상 밖의 필수 policy/evidence blocker가 없음

#### 주요 활동

1. mutation 전에 exact selected Goal·현재 요청·scope와 relevant prerequisite/dirty drift를 확인한다. correction이면
   실제 원인 delta와 허용 변경이 같은 owner에 귀속되는지 다시 확인한다.
2. current behavior, authority, producer-consumer, data/schema/store/transport와 validation seam을 필요한 만큼
   조사한다. section 6.3의 상세 설계와 준비 점검으로 실제 타입·상태·순서·취소/복원·전환·시험을 구체화한다.
3. 적용되는 initial baseline prerequisite를 mutation 전에 이행하거나 유효한 기존 evidence로 확인한다.
   불명확한 정책이나 필요한 독립 검토를 설계 완료 문구로 대체하지 않는다.
   §6.4의 복원 가능한 구현 전 baseline을 고정하고, 단계검토 opt-in이면 실제 의존성에 맞는 checkpoint 계획을 연결한다.
4. assigned execution node를 유효한 중간 상태가 깨지지 않도록 구현한다. correction에서는 승인된 동작을
   복원하는 범위만 수정하고, 검증 전용 pass에는 새로운 구현을 추가하지 않는다. Goal 밖 cleanup은 하지 않는다.
5. 필요한 test/data/fixture/probe와 canonical contract 문서를 같은 경계에서 정렬한다. 배정된 durable
   integration seam도 해당 owner가 구현한다. section 6.4의 guardrail을 source-level 완료 조건으로 적용한다.
6. required dimension·input-to-seam 영향 map을 실제 evidence에 연결한다. focused와 필요한 broader 검증을
   실행하고, 새 실행/재사용·실패/교정/재검증·생략을 구분해 EXPERIMENTS 역할에 기록한다.
7. 기술 선택·설계 수정·evidence gap·새 정책 질문·범위 밖 후보를 NOTES 역할에 요약한다. 해결 가능한 기술
   실패는 승인 범위에서 계속 수정하고, 의미 있는 진전이 없거나 권한 경계가 생기면 기록 후 중단한다.
   opt-in checkpoint에 도달하면 §6.4에 따라 부분 snapshot·검증·잔여 범위를 고정하고 종료한다. 부분 검토/교정/
   다음 구현의 별도 요청과 currentness를 확인한 뒤 남은 구간을 이어가며, 모든 구간이 끝나야 아래 전체 exit를 평가한다.
8. completion conditions를 current actual과 검증에 대조한다. 감사 verdict를 미리 쓰지 않는다.
9. 이번 delta와 predecessor/기존 successor audit·baseline의 영향 관계를 대조한다. stale 또는 새 실패가 있으면
   exact owner·영향 검증·필요 fresh audit와 복원 순서를 남긴다. 정책/Phase 결함과 기술 회귀를 구분한다.
10. 자신이 final baseline owner이고 요구 시점이 됐다면 current integrated target에서 이를 수행한다. correction
    뒤 다른 owner가 필요하면 그 선행·권한을 확인하여 후속 검증 pass로 연결하고 완료를 가장하지 않는다.
11. 감사할 exact target·scoped diff·외부 actual·evidence를 Audit Target Record로 고정하고 mutation을 멈춘다.
12. 기본적으로 실제 구현/검증 결과와 남은 audit를 보고하고 종료한다. `goal-run`도 감사까지 명시 위임되고
    남은 checkpoint가 없을 때만 필요한 baseline·predecessor audit 복원과 selected Goal의 fresh GEX-20을
    분리된 pass로 수행한다. 복원 대기 audit는 긍정으로 선반영하지 않는다.

#### 산출물

- selected Goal 범위의 implementation delta
- current test/data/document/probe와 필요한 legacy removal 결과
- 실패와 재검증을 포함한 execution/validation evidence
- exact Audit Target Record와 known omission/limitation
- predecessor audit currentness/continuity 결과와 필요한 re-audit prerequisite
- 코드 기반 상세 설계·준비 점검과 실제 변경·시험의 연결
- reusable/invalidated baseline, 새 실행·재사용 근거와 필요한 복원 owner
- updated PLAN·EXPERIMENTS·EXPERIMENT_NOTES 역할의 실제 문서/절

#### 종료 조건

- 모든 assigned execution node와 applicable implementation boundary가 구현됨
- completion condition마다 current implementation 또는 validation evidence를 제시할 수 있음
- Implementation Quality Guardrails 위반이 없고, applicable structural completion/legacy removal condition이
  source와 current consumer에서 충족됨
- 모든 required validation dimension이 direct evidence를 가지며, N/A disposition이 current actual에서
  타당하고 happy-path-only evidence가 applicable negative/edge dimension을 대신하지 않음
- required focused와 proportional broad validation coverage가 실제 실행 또는 유효한 재사용 근거로 충족되고,
  baseline/omission disposition이 완료 주장을 침범하지 않음. 적용되는 자신의 baseline 책임을 이행함
- unresolved material policy, required consumer/evidence gap 또는 unsafe dirty overlap이 없음
- Audit Target Record가 complete하고 감사 대상이 안정되어 있으며 이후 source/data/evidence 수정은 새
  GEX-10 pass 또는 audit currentness 재판정으로 구분할 수 있음
- 상세 설계·준비 점검이 변경의 위험과 공유 계약에 맞게 이행되었고 구현 결과와의 차이를 설명할 수 있음
- 제한적 correction에서는 해당 결함을 수리·검증하고 남은 영향 audit를 식별함. 감사 복원 전의 GEX-10 종료는
  구현 pass 종료일 뿐 Phase prerequisite·전체 완료가 아니며 신규 후속 Phase를 시작하지 않음
- `COMPLETION_AUDIT.md`의 긍정 verdict를 선반영하지 않음

#### 중단과 반환

같은 Goal 안의 기술 문제는 이 실행 pass에서 안전하게 수정·재검증할 수 있다. 종료 시점까지 해결하지 못하면
`Not Complete` 또는 `Blocked`로 보고한다. active goal-run은 안전하고 유의미한 진전이 가능한 in-scope 기술
correction을 새 GEX-10 pass로 이어갈 수 있다. correction 진전이 없거나 external blocker면 section 6.6의
영향 범위에 따라 독립 eligible Phase로 진행하거나 run을 중단한다. `stage-run`은 새 요청을 기다린다.
정책·Phase·Goal 경계 문제는 section 8의 P2G owner로 반환한다.

<!-- GEX-10 END -->

<!-- GEX-20 START -->
### GEX-20 — Single/Phase 완료 감사

#### 목적

GEX-10이 고정한 audit target을 fresh read-only pass로 재검토하고, selected Single/Phase Goal의 완료 여부를
독립적으로 판정한다.

#### 입력

- exact Handoff, Goal Set, selected Goal member와 Phase
- PLAN·EXPERIMENTS·EXPERIMENT_NOTES 역할의 실제 문서/절
- GEX-10의 exact Audit Target Record와 scoped diff 또는 section 5/6.6 재감사의 predecessor record·구현 delta
  owner/근거 갱신 경로. source delta가 없으면 근거 있는 N/A
- applicable 설계 인계·준비 점검, baseline reuse contract와 새 실행/재사용·invalidated 근거
- current code/data/document와 validation evidence
- applicable reviewer requirement
- 사용자의 direct GEX-20 `stage-run` 요청 또는 selected Goal의 감사까지 명시 위임된 active `goal-run` authorization (Reader Contract의 checkpoint 우선)

#### 진입 조건

- initial audit이면 selected Goal의 GEX-10 종료 조건과 안정된 target이 식별됨
- initial target은 `current / unchanged | current / proven non-impacting delta`여야 함. 허용된 fresh re-audit는
  section 5/6.6의 기존 구현 기록·귀속된 후속/correction delta 또는 구현 무변경의 evidence 갱신 근거와 current
  actual로 새 target을 고정할 수 있음. selected Goal의 코드를 다시 고치지 않았다면 audit record를 만들기 위해
  불필요한 GEX-10을 강제하지 않음
- 제한적 교정 후 선행 감사 복원은 아직 미복원된 후행 verdict를 요구하지 않지만, 해당 Goal의 실제 선행·
  승인 의미·검증 입력을 직접 확인하고 다른 미해결 결함을 긍정으로 덮지 않음
- 현재 요청이 이 exact 감사 대상을 허용하고 필요한 reviewer 조건을 충족하며 fresh read-only review가 가능함

#### 주요 활동

1. initial audit이면 exact Audit Target Record를 current actual과 다시 비교하고 currentness 판정, changed
   member와 non-impact evidence를 기록한다. old record가 `stale`이면 그것을 current target인 것처럼 통과시키지
   않는다. 허용된 fresh re-audit이면 current actual에서 새 Audit Target Record를 먼저 고정하고 predecessor
   record, delta identity와 owner 또는 구현 무변경·evidence 갱신 근거를 기록한다.
2. Goal의 objective, scope, assigned execution node, policy/decision/acceptance coverage, applicable Goal Quality
   Contract, invariant/structural completion, validation과 완료 조건을 원문 기준으로 확인한다. 이미 읽은 불변
   부분은 exact ref와 현재성으로 재사용하고 새/변경/불명확한 부분은 직접 읽는다. 전체 과거 이력은 기본 입력이 아니다.
3. §5에 따라 각 완료 조건의 영향과 근거를 대조한다. 영향받거나 미검토인 producer/consumer·상태·실패/취소/
   전환은 현재 source·scoped diff·검증을 직접 검토하고, 무영향 조건은 검증한 기존 판단·evidence를 exact ref로
   연결한다. 모든 조건의 추적·현재성 책임은 유지하며, 영향표나 구현자 요약만으로 의미 판정을 대신하지 않는다.
4. applicable한 runtime/state, data/schema/store, transport/DTO, producer-consumer, migration/compatibility,
   legacy/fallback, durable document와 dirty attribution을 확인한다.
5. selected Goal에 durable integration validation seam 구현이 배정됐으면 current target에 존재하고 Master가
   사용할 locator, invocation, fixture/input과 evidence contract가 completion condition을 충족하는지 확인한다.
6. scoped diff와 current production path를 직접 읽어 test/fixture-specific behavior, silent normalization,
   broad catch-all recovery, consumer-side 재추론, 미래 전용 abstraction, arbitrary timing/sequence workaround,
   의미 없는 snapshot 갱신, duplicate authority와 승인되지 않은 legacy/fallback이 없는지 확인한다. Test
   결과만으로 이 source-level 판정을 대신하지 않는다.
7. 각 Validation Dimension의 required evidence와 N/A 근거, applicable baseline contract를 current actual에
   대조한다. non-zero 실행과 input-to-seam 영향·유효성을 확인하고 새 실행과 재사용을 구분한다. 부족하거나
   invalid한 검증만 section 6.4의 안전 경계에서 재실행한다. fresh audit 자체를 전체 재실행 사유로 쓰지 않는다.
   final baseline의 owning GEX-10 책임을 audit-local probe로 대신하지 않는다.
8. omitted/inaccessible/unknown evidence와 baseline failure가 완료 주장을 침범하는지 판정한다.
9. `single_phase`이면 별도 GEX-30이 없으므로 Goal Set 전체 acceptance와 broad regression, 적용되는 final
   baseline까지 이 감사에서 확인한다. 새 source/fixture 수정이 필요하면 감사를 닫고 GEX-10으로 반환한다.
10. `COMPLETION_AUDIT.md`에 Audit Target Record, condition별 evidence와 `Complete / High | Not Complete |
   Blocked` verdict를 기록한다.
11. 결함이 있으면 수정하지 않고 exact finding·귀속 delta·실제 correction owner와 영향받는 감사/검증을
    기록한다. A 감사에서 B 회귀를 발견했다고 A를 임의 교정 대상으로 삼지 않는다.

#### 산출물

- `COMPLETION_AUDIT.md`
- Audit Target Record currentness 판정과 applicable non-impact evidence
- completion-condition evidence matrix 또는 동등한 추적 가능한 설명
- primary verdict와 remaining risk/out-of-scope
- multi-phase이면 dependency상 새로 eligible해질 수 있는 Phase 또는 GEX-30 prerequisite 보고

#### 종료 조건

긍정 종료는 다음을 모두 만족하는 `Complete / High`뿐이다.

- 모든 applicable completion condition이 current direct evidence로 pass함
- selected Goal의 scope와 정책 의미가 구현 결과와 일치함
- source-level Implementation Quality Guardrails와 structural completion/legacy disposition이 통과함
- applicable Validation Dimension이 current direct evidence를 가지며 근거 없는 N/A나 happy-path-only 대체가
  없음. 실제 재실행과 유효성 확인 후 재사용한 evidence를 구분하고 자신의 baseline 책임을 확인함
- required predecessor의 실제 의미와 applicable current 감사가 복원되어 있고, correction 진입 예외를
  긍정 completion이나 신규 Phase prerequisite의 대체로 사용하지 않음
- completion-blocking validation omission, unknown, consumer 또는 compatibility gap이 없음
- verdict가 complete Audit Target Record에 귀속되고 audit 종료 시까지 target currentness가 유지됨
- unrelated dirty work가 보존되고 Goal 성과로 잘못 귀속되지 않음
- 감사 중 audit target을 수정하지 않음

#### 다음 경계

- `single_phase`의 `Complete / High`는 해당 Goal Set 실행을 완료한다.
- Phase Goal member를 root target으로 한 `goal-run`은 해당 Phase의 `Complete / High`에서 완료한다. Goal Set의
  나머지 Phase나 GEX-30을 자동 실행하지 않는다.
- `stage-run`은 새로 eligible해진 Phase 또는 GEX-30 prerequisite를 보고하되 선택·초기화·실행하지 않는다.
- Goal Set `goal-run`은 미완료 eligible Phase가 있으면 section 6.6에 따라 다음 GEX-00 pass를 시작하고, 모든
  Phase가 `Complete / High`이면 fresh GEX-30 pass를 시작한다.
- `Not Complete` 또는 `Blocked`이면 dependent Phase와 GEX-30의 정상 진행을 허용하지 않는다. active goal-run의
  in-scope 기술 결함은 감사 종료 뒤 실제 owner의 correction GEX-10으로 반환한다. 선행 audit 무효화가 얽히면
  section 6.6의 제한적 진입과 복원 조건을 적용한다. 수리 불가 Phase-local finding 뒤에는 같은 절의 조건을
  충족하는 독립 eligible Phase만 진행할 수 있다.

<!-- GEX-20 END -->

<!-- GEX-30 START -->
### GEX-30 — Master 최종 통합 감사

#### 목적

multi-phase Goal Set의 모든 Phase 완료 뒤, Phase별 완료 주장을 그대로 신뢰하지 않고 final current tree에서
정책 coverage, cross-Phase seam, 통합 validation과 전체 완료 조건을 fresh read-only pass로 검토한다.

#### 입력

- accepted P2G-60 Handoff와 그 exact Final Policy Freeze, Phase Plan 및 multi-phase Goal Set
- Master Goal member와 모든 Phase Goal member
- 모든 Phase `Complete / High` 감사와 각 Audit Target Record/currentness evidence
- final current code/data/document와 integrated validation evidence
- applicable Shared Validation Baseline Reuse Contract와 actual final baseline owner/result·재사용/영향 판정
- cross-Phase invariant, integration acceptance, Integration Validation Ownership Map과 applicable reviewer requirement
- 사용자의 direct GEX-30 `stage-run` 요청 또는 Goal Set의 감사까지 명시 위임된 active `goal-run` authorization (Reader Contract의 checkpoint 우선)

#### 진입 조건

- Goal Set의 모든 Phase가 section 5의 currentness 판정을 통과한 `Complete / High` 감사를 가짐
- final audit target과 section 6.1의 execution basis가 유효하고 서로 일치함
- Phase 완료 이후 통합 의미를 바꾸는 미감사 수정이 없음
- 모든 integration acceptance에 existing/audit-local read-only seam 또는 구현·감사 완료된 owning Phase
  Goal/Final Execution Node가 있음
- applicable final baseline이 지정 executable owner의 current integrated target에서 충족됨. 없으면 GEX-30이
  ownership을 가져와 실행 완료로 만들지 않고 owning GEX-10과 필요한 audit부터 복원함
- 감사자가 구현/Phase 감사와 필요한 수준으로 분리된 fresh review를 수행할 수 있음

#### 주요 활동

1. final tree, external actual과 integrated evidence를 section 5의 Master Audit Target Record로 고정한다. 모든
   Phase audit의 target을 final actual과 비교해 currentness와 successor Phase delta의 영향 판정을 기록한다.
2. Goal Set의 모든 Phase와 Goal coverage가 누락·중복 없이 완료됐는지 확인한다.
3. Final Freeze의 policy/decision/acceptance가 final implementation과 Phase 감사까지 추적되는지 확인한다.
4. Phase 사이 authority transfer, schema/version/migration chain, ordering, persistence, transport/DTO,
   producer-consumer와 approved fallback 제거 조건을 final actual에서 검사한다. Phase-local test가 통과했어도
   duplicate authority, test-specific production behavior, silent recovery, 미래 전용 abstraction 또는
   arbitrary timing/sequence workaround가 cross-Phase seam에 남아 있지 않은지 source에서 직접 확인한다.
5. integration acceptance별 seam disposition과 current availability를 확인한다. existing 또는 Phase-owned
   durable seam은 read-only로 재실행할 수 있다. `audit_local_read_only` probe는 audit target을 바꾸지 않는
   임시 정의·입력·출력으로 작성·실행하고 재현 방법과 결과를 audit evidence에 보존할 수 있다.
6. 모든 Phase의 Validation Dimension coverage·N/A·baseline contract를 final actual에서 연결한다.
   cross-Phase/broad integrated evidence의 유효성을 판단하고, 부족·invalid한 범위를 read-only로 재실행한다.
   Phase-local happy path만으로 통합을 증명하지 않으며, 새 감사라는 이유로 유효한 full baseline을 반복하지 않는다.
   audit-local probe는 지정된 final baseline owner의 이행 근거를 대신하지 않는다.
   §5/6.4/6.7의 조건별 영향·공통 증거 재사용은 Master에도 적용한다. Master는 cross-Phase 결합과 전체
   completion을 직접 판단하되 유효한 Phase 내부 조사·로그·기계 집계를 복제하지 않는다. 새 통합 위험,
   미해결 finding 또는 근거 불일치에는 필요한 source와 시험으로 확장한다.
7. durable canonical 문서, live data/fixture, legacy 부재와 전체 dirty-work attribution을 검토한다.
8. Master 전체 완료 조건마다 current evidence를 대조하고 completion-blocking unknown이나 hidden deferral이
   없는지 확인한다.
9. Master `COMPLETION_AUDIT.md`에 Master Audit Target Record, `Complete / High | Not Complete | Blocked`
   verdict와 영향을 받은 Phase/owner를 기록한다.

#### 산출물

- Master `COMPLETION_AUDIT.md`
- Master Audit Target Record와 모든 Phase audit currentness/continuity 결과
- policy-to-Phase-to-final-actual coverage 결과
- cross-Phase seam과 integrated validation 결과
- primary verdict, remaining risk/out-of-scope와 필요한 반환 단계

#### 종료 조건

`Complete / High`는 다음을 모두 의미한다.

- 모든 Phase와 Master 전체 완료 조건이 충족됨
- final implementation이 하나의 일관된 policy/runtime/data/consumer contract를 이룸
- final implementation이 Implementation Quality Guardrails를 위반하지 않고 applicable negative/edge
  validation dimension을 happy path로 대체하지 않음
- cross-Phase seam과 required integrated validation이 current direct evidence로 pass함
- completion-blocking unknown, hidden compatibility path, legacy authority 또는 미감사 수정이 없음
- verdict가 complete Master Audit Target Record에 귀속되고 audit 종료 시까지 target currentness가 유지됨
- 감사 중 implementation을 수정하지 않음

`Complete / High`일 때 Goal Set 실행은 terminal completion으로 보고할 수 있다. 배포, 다음 정책 revision 또는
후속 Goal은 자동 승인하지 않는다.

#### 실패와 반환

기술 결함이면 책임 delta와 exact Single/Phase owner를 식별하여 correction GEX-10과 relevant GEX-20을
수행한다. predecessor audit 무효화가 포함되면 section 6.6의 제한적 교정·감사 복원 조건을 적용하고 필요한
final baseline도 owning executable Goal에서 복원한다. 기존 감사는 삭제하거나 성공으로 덮어쓰지 않는다.
policy/Phase/Goal 구조 문제이면 section 8의 P2G owner로 반환한다. 필요한 durable integration validation seam의 Final Execution Node/Phase owner가 없으면
P2G-40, Phase Plan에 owner가 있지만 Goal mapping만 누락됐으면 P2G-50으로 반환한다. `stage-run`에서는
correction과 GEX-30 재감사에 각각 새 사용자 요청이 필요하다. active
Goal Set `goal-run`에서는 section 6.6에 따라 correction GEX-10, fresh GEX-20과 fresh GEX-30을 분리된 pass로
이어갈 수 있다.

<!-- GEX-30 END -->

## 8. Finding 반환 기준

| 발견 내용 | owner / 권장 반환 단계 |
| --- | --- |
| material policy, 사용자-visible 의미 또는 Observable Policy Acceptance 변경 필요 | `P2G-10` |
| current code behavior, source basis, implementation delta 또는 consumer boundary 재검토 필요 | `P2G-20` |
| Final Policy Freeze membership/currentness 또는 freeze completeness 결함 | `P2G-30` |
| Work Package, dependency, Atomic Group, Phase 경계·순서·병렬성 변경 필요 | `P2G-40` |
| Phase 구조는 그대로지만 Goal scope, validation, completion 또는 mapping 문구 교정 필요 | `P2G-50` |
| upstream artifact는 맞지만 Handoff exact ref, summary 또는 authorization boundary 결함 | `P2G-60` |
| required durable integration validation seam에 Final Execution Node/Phase owner가 없음 | `P2G-40` |
| Phase Plan의 integration owner는 맞지만 Goal validation/completion mapping이 누락됨 | `P2G-50` |
| 정책·Phase·Goal 의미가 그대로인 selected Goal 내부 기술 결함 | exact owner의 `GEX-10` |
| 정상 인수한 후속 mutation이 선행 audit를 무효화/회귀시켰고 교정 owner가 확정됨 | section 6.6 제한 조건의 owner `GEX-10`, 영향 검증·audit 복원; 피해를 본 Phase를 무조건 교정 owner로 삼지 않음 |
| 현재 수행할 final/focused baseline 실행·재확보가 필요하고 owner가 유효하게 배정됨 | owner의 검증 전용 `GEX-10`과 필요한 fresh audit. root 밖이면 별도 권한 |
| mutation 전 필수 initial baseline이 이미 누락됨 | section 6.4의 과거 근거·계약 위반 판정. current 실행을 과거 initial 성공으로 대체하지 않음 |
| implementation defect 없이 target delta나 evidence expiry로 이전 Phase audit만 `stale`해짐 | `GEX-20` fresh audit |
| 구현은 그대로이고 누락된 감사 evidence만 새로 이용 가능해짐 | `GEX-20`, Master이면 `GEX-30` fresh audit |
| final cross-Phase 기술 결함 | 책임 owner의 `GEX-10`, 영향 Goal의 `GEX-20`·baseline 복원, 이후 `GEX-30` |
| 실행 문서의 역할/locator 또는 근거로 복원 가능한 요약 누락 | section 5 범위의 `GEX-00`; 실행 사실 유실·identity 모순이면 Blocked |

P2G owner로의 반환을 보고하는 것은 그 P2G 단계를 실행하거나 승인하는 것이 아니다. 발견 evidence, 완료되지
않은 이유, 영향 범위와 권장 owner를 보고하고 현재 run을 중단한다. GEX 단계로의 기술적 반환은 현재 pass 안에서
수정한다는 뜻이 아니다. `stage-run`은 새 사용자 요청을 기다리고, active `goal-run`만 section 6.6에 따라 새
correction 또는 audit pass를 시작할 수 있다.

## 9. 기존 자료에서 유지한 원칙과 제외한 구조

이 문서는 기존 Phase/Master Goal의 실제 운용에서 다음 원칙을 유지한다.

- PLAN·EXPERIMENTS·EXPERIMENT_NOTES·COMPLETION_AUDIT의 역할 분리와 기존 동등한 구조 재사용
- current 완료·감사 대기·교정·stale을 구분한 재개, 초기화 누락과 실행 사실 유실의 구분
- P2G의 권장 설계를 코드 기반 상세 설계로 연결하고 위험에 비례한 준비 점검 수행
- 구현 전에 current tree와 dirty baseline을 다시 확인
- Phase 구현 뒤 별도의 독립 완료 감사 수행
- Phase 감사 통과 전 신규 dependent Phase를 실행하지 않음. 이미 수행한 delta의 제한적 교정만 section 6.6을
  따르며, 무효화된 prerequisite를 통과로 취급하지 않음. 독립 Phase는 scoped blocker/currentness 계약을 따름
- active goal-run에서도 GEX 단계별 pass와 mutation/read-only 권한을 분리
- 모든 Phase 완료 뒤 Master final actual을 다시 감사
- Master가 Phase 구현 세부사항을 복제하거나 직접 구현하지 않음
- exact validation result, baseline failure, omission, consumer와 legacy evidence를 완료 주장에 연결
- 새 audit 판단과 검증 재실행을 구분하고 applicable baseline owner·재사용·영향 재검증을 유지
- 지침 revision 적용·기록 보완·현황 갱신만으로 upstream 승인이나 과거 완료 verdict를 소급 변경하지 않음
- 과거 Goal/audit를 current runtime authority로 사용하지 않음

기존 운용 자료에서는 실행 target 확인, implementation/validation, fresh audit와 Master integration audit의
분리를 참고했다. 다음 구조는 이 파이프라인의 목적에 필요하지 않아 가져오지 않는다.

- Campaign/Role/Basis 상태 기계와 전이 catalog
- Goal Gate ledger, activation snapshot과 authorization ID 체계
- content-addressed Canonical Candidate, Pipeline Head, CAS와 promotion/recovery protocol
- 전체 예외를 사전 열거하는 classifier와 blocker/resume overlay
- 고정된 five-dimension Fit 점수표와 과도한 intermediate registry

이 문서의 중심 계약은 실행 방법을 촘촘히 통제하는 것이 아니다. 한 단계의 목적·권한·입력·출력·완료 조건과
금지선을 분명히 하고, 그 안의 구체적인 수행 방법은 current evidence를 다루는 AI의 판단에 맡긴다.
