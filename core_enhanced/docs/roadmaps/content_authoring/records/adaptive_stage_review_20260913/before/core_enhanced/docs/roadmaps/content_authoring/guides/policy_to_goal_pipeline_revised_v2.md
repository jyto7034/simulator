# Policy-To-Goal Pipeline

Status: `ready`

Document Revision: `2026-09-13-r6`

r2에서 도입한 **단계별 수동 진행과 승인 범위 내 연속 진행의 구분**을 유지한다.
단계 ID·artifact owner·불변 revision·완전성 검증·정책/구현 권한 분리는 유지하되, 매 단계 종료마다 새 요청을
받아야 하는 일률적인 제한은 제거한다. 사용자 선택이 필요한 정책, 범위·위험·권한 확대와 필수 근거 부족은
여전히 중단 경계다. 위임된 통상적 정책은 `8.6`에 따라 결정·기록하고 승인 범위에서 계속한다.

이 문서의 `ready`는 지침의 사용 상태이지 실제 Policy Topic, validator 또는 GEX 통합의 통과 상태가 아니다.
Goal Quality Contract Version `2`와 companion CLI를 이번 문서 편집만으로 변경하지 않는다. 실행 방식 정보는
기존 작업 view/요청 기록에 두며 새 canonical artifact·Ref·GEX 실행 권한을 만들지 않는다. `8.8`의 도구 경계를 따른다.

r5는 2026-09-11 사용자 합의에 따라 통상적 정책과 사용자 선택이 필요한 정책의 판단 기준을 `8.6`에 둔다.
위임된 AI 결정과 사용자 직접 결정을 구분하며, 사용자 의논 때의 주 작업 대기·기존 독립 작업 조건을 유지한다.
기존 artifact와 과거 실행·감사 결과의 적용 revision을 소급 변경하지 않는다.

r6는 §8.8에 명시적 단계검토 요청의 설계 인계를 추가한다. 기본 구현 후 감사 인계와 선택적 중간 중단의
실행 계약은 GEX gex-r6가 소유한다. 기존 schema·validator·정식 artifact는 변경하지 않으며 P2G-60 뒤 구현은
여전히 별도 사용자 요청이 필요하다. 배포 사본은 이 원문과 동일하게 유지한다.

Scope: 임시 정책 문서 생성부터 최종 정책 동결, Phase 설계, Single 또는 Master/Phase Goal 작성까지

Downstream pipeline: [`goal_execution_pipeline.md`](./goal_execution_pipeline.md) — P2G-60 Handoff 이후의 Goal
선택, 구현·검증, Phase 완료 감사와 Master 최종 통합 감사

<!-- P2G-READER START -->
## Reader Contract

이 문서를 사용하는 AI는 전체 문서를 기본 입력으로 읽지 않는다. 먼저 이 Reader Contract를 읽고 사용자
요청을 `execute | inspect | status | audit | document-edit` mode 중 하나로 구분한다. `execute`일 때만 P2G ID와
직접 결과를 이용해 실행 단계를 route한다. 다른 mode의 P2G ID는 조회·상태 확인·감사·편집 대상을 식별할 뿐
단계 실행이나 이동 권한이 아니다.

### 작업 종류와 적용 범위

정식 정책 검토·동결·Phase/Goal 작성 또는 그 산출물의 수리가 이번 문서의 대상이다. 오타 수정, 설명 보완,
명확한 소규모 코드 변경에 Policy Topic·Freeze·Goal 전체 절차를 자동 적용하지 않는다. 이미 승인된 Goal의
필수 검증을 피하려고 실제 계약 변경을 소규모 작업으로 재분류하지도 않는다.

`inspect | status | audit | document-edit`는 요청 범위의 조회·검토·편집만 수행한다. 특히 이 지침을 개선해 달라는
요청은 문서를 수정할 권한이지 그 안의 P2G 단계나 프로젝트 구현을 실제 시작할 권한이 아니다.

### execute mode: 종료 결과와 진행 방식

먼저 요청에서 **대상 Topic·범위, 원하는 최종 결과, 허용 행동, 명시적 정지 지점**을 파악한다. 사용자가 아래
내부 용어나 양식을 직접 입력할 필요는 없다. 같은 범위의 이전 결정·현재 작업 문서로 확인 가능한 사실은
확인하고, 실질적인 범위·정책·권한의 선택이 남을 때만 묻는다.

| 진행 방식 | 선택 기준 | 실행 경계 |
| --- | --- | --- |
| `single_stage` | `P2G-20만`, `검토 후 멈춰`, 특정 P2G ID의 단독 실행처럼 한 단계 또는 명시적 검토 지점만 요청 | 해당 단계의 exit까지 수행. 다른 단계가 필요하면 누락과 이유를 보고하고 요청 범위를 넘지 않음 |
| `bounded_pipeline` | `이 Topic을 검토해서 Goal 작성까지`, `필요한 단계를 거쳐 handoff까지`처럼 여러 단계 또는 최종 산출물 완성을 요청 | 승인된 동일 범위에서 필요한 선행·후속·수리 단계를 각각 검증하며 종료 결과까지 이어감 |

단계 ID만 단독 지정한 요청은 `single_stage`로 해석한다. ID가 있어도 `P2G-20부터 P2G-50까지`처럼 끝점과
연속 진행 의도가 명확하면 `bounded_pipeline`이다. `이 Topic의 Goal 문서를 만들어줘`처럼 완성 산출물을
요청했고 범위가 확인되면 필요한 선행 단계를 포함하는 `bounded_pipeline`으로 해석할 수 있다.
`이번에는 코드 검토만`처럼 더 좁은 제한이 함께 있으면 그 제한을 우선한다.

`Goal 작성`의 종료는 P2G-50이며, `구현자에게 인계할 handoff까지`가 요청되면 P2G-60이다. P2G 단계 연속 진행은
GEX·production mutation·배포까지 확장되지 않는다. 여러 Topic 가운데 무엇을 수행할지 결정되지 않았다면
임의로 하나를 골라 초기화하지 않고 선택할 경계를 묻는다.

### 단계 선택·계속 진행

1. 요청과 기존 기록에서 위 실행 경계를 확인하고 짧게 기록한다. 별도 승인 양식이나 새 Ref는 만들지 않는다.
2. 범위·끝점에 필요한 prerequisite를 조사하여 current인 결과는 재사용한다. 없거나 더 이상 current가 아닌
   결과 중 가장 이른 필요한 owner 단계를 선택한다. 존재하지 않는 prerequisite를 충족된 것으로 처리하지 않는다.
3. `single_stage`는 지정 단계 밖을 수행하지 않는다. `bounded_pipeline`은 범위 안의 누락된 선행 결과도 각
   owner 단계에서 생성·검증할 수 있다. 초기 scope나 위임되지 않은 정책까지 대신 승인할 수는 없다.
   통상적 정책의 결정 권한·근거·기록은 `8.6`을 따르며 단계 진행 권한과 별도로 확인한다.
4. 각 단계의 entry와 exit, currentness·coverage·binding 검증은 그대로 수행한다. 여러 단계를 실행하더라도
   owner를 합치거나 candidate를 후속 current input으로 사용하지 않는다.
5. 완료한 단계에서 종료 결과에 아직 도달하지 않았고 다음 단계의 조건을 만족하면 계속한다. 초안 완성,
   단계 번호 변경 또는 검증기 실패 자체만으로 사용자 재승인을 요구하지 않는다. 실패는 원인·owner를 판정해
   승인 범위에서 교정·재검증하고, 해결할 수 없는 조건에서만 `8.6`에 따라 멈춘다.
6. 범위 안의 non-material 수리는 `8.4`의 불변 successor와 영향 범위 규칙을 지켜 가장 이른 owner에서 처리한
   뒤 필요한 후속 결과만 다시 검증한다. 사용자와 의논이 필요한 정책이 확인되면 `8.6`에 따라 주 작업은
   질문 후 대기하며, 이미 실행 중인 독립 작업만 기존 범위까지 완료할 수 있다. 그 전에 통상적 정책인지
   판정하여 위임 범위 안이면 결정·기록 후 계속한다. `bounded_pipeline`이 P2G-10을 포함하면 이미 확보한
   근거로 질문·중단 checkpoint를 등록하기 위해 별도의 단계 이동 허락을 다시 받지 않는다.
7. 사용자의 정책 답변이 기존에 중단된 연속 진행 요청의 질문을 해결하면 그 요청을 재개할 수 있다. 답변이
   모호하거나 범위가 바뀌거나 새 위험이 생기면 필요한 확인만 추가한다. 단독 정책 답변이나 `좋아`만으로
   존재하지 않던 연속 진행 권한을 새로 만들지 않는다. 명시적 사용자 checkpoint는 그대로 지킨다.
8. 요청한 최종 결과와 필요한 검증이 완료되면 멈춘다. 모두 이미 current라면 중복 생성하지 않고 결과를
   보고한다. 완료·blocker·checkpoint에서 실제 결과, 남은 범위와 필요한 결정만 요약한다.

이 진행은 현재 요청의 작업 범위다. 백그라운드 실행·예약이나 이후 별도 대화에서의 무기한 권한을 의미하지
않는다. 세션/도구 한계로 더 수행할 수 없으면 완료를 가장하지 않고 마지막 유효 결과와 재개 지점을 기록한다.

예: `이 Topic의 정책을 검토하고 코드 조사 후 Phase와 Goal까지 작성해줘. 새 정책 질문에서만 멈춰`는
P2G-50까지의 연속 진행 요청이다. 이미 승인된 정책을 다시 묻지 않고 필요한 P2G-00/10/20/30/40/50을 통과한다.
새 비용·수명·호환 정책이 필요한 경우에는 그 질문에서 멈추며, Goal 작성이 끝나도 구현을 시작하지 않는다.

### Routing Table

| 단계 | 요청에서 찾을 직접 결과 | 선택 시 추가로 읽을 section |
| --- | --- | --- |
| `P2G-00` | 새 Policy Topic과 사용자 승인 initial scope를 가진 새 Working Policy Document 초기화 | initial revision은 `8.7`; artifact 작성은 `9` |
| `P2G-10` | 기존 Working Policy Document의 scope/policy basis 변경, Policy Element, 미결 질문, 사용자 직접/위임 결정 또는 잠정 폐쇄 | 항상 `8.1`; 제한적 의사결정 지원 조사·Decision 재분할은 P2G-10의 해당 절; 코드 사실 확인이면 `8.5`; revision 작성은 `8.7`; post-freeze successor이면 `8.4`; blocker면 `8.6`; artifact 작성은 `9` |
| `P2G-20` | current code behavior, implementation delta 또는 code-informed review | `8.1, 8.2, 8.3, 8.5, 8.8`; 변경이면 `8.4`; blocker면 `8.6`; successor이면 `8.7`; artifact 작성은 `9` |
| `P2G-30` | Final Policy Freeze | `8.3, 8.7`; 변경이면 `8.4`; artifact 작성은 `9` |
| `P2G-40` | Work Package, dependency, Atomic Group 또는 ordered Phase Plan | `8.3, 8.8`; 새 질문/변경이면 `8.1, 8.4, 8.7`; artifact 작성은 `9` |
| `P2G-50` | Single Goal 또는 Master/Phase Goal | `8.3, 8.8`; 새 질문/변경이면 `8.1, 8.4, 8.7`; artifact 작성은 `9` |
| `P2G-60` | 구현 전 최종 handoff | `8.3`; revision 상태는 `8.4, 8.7`; report 작성은 `9` |

`P2G-00`은 신규 초기화 전용이다. requested physical path 점유 여부와 requested document identity의 기존 사용
여부를 독립된 축으로 판정한다. Filesystem-backed repository의 requested physical path는 section 5.2의
`<Target Documentation Root>/goals/<Policy Directory Name>/WORKING_POLICY[.<locale>].md` 규칙을 만족해야 한다.

- `same_identity_same_path`: 기존 문서의 scope 또는 policy basis 변경이면 P2G-10으로 route하고, Final Policy
  Freeze 이후 변경이면 `8.4`와 `8.7`도 적용한다. 초기화 결과가 이미 current이면 재초기화하지 않는다.
  `bounded_pipeline`의 후속 요청 결과가 남아 있으면 그 결과에 필요한 다음 단계로 진행하고, 초기화/상태만
  요청했거나 종료 결과도 current이면 상태를 보고한다.
- `path_occupied_by_different_identity`: 물리적 경로를 다른 document identity가 점유한다. 기존 artifact나
  requested identity를 바꾸지 않고 안전한 미점유 경로를 확인한다. 경로·이름 할당이 위임된 경우 같은 승인
  root 안의 다른 적합한 경로를 선택하고 기록할 수 있다. 명시적으로 고정된 경로이거나 의미·소유가 불분명하면
  사용자에게 묻는다. identity 변경만으로 path collision이 해소됐다고 간주하지 않는다.
- `identity_already_exists_at_other_path`: requested identity가 다른 경로의 기존 문서를 가리킨다. 동일 문서를
  의도했다면 기존 경로를 사용하고 변경 여부에 따라 current 상태 보고 또는 P2G-10으로 route한다. 별개의 새
  문서를 의도했고 이름 할당도 위임됐다면 새로운 unique identity와 conforming 미점유 경로를 배정할 수 있다.
  같은 문서인지 별개 문서인지 불명확하면 사용자에게 묻고 기존 문서는 변경하지 않는다.
- `identity_or_path_indeterminate`: 어느 문서도 수정하지 않고 사용자에게 requested identity와 path를 확인한다.

경로가 같다는 사실만으로 기존 문서 변경을 추론하거나 document identity 변경만으로 물리 충돌을 해결하지
않는다.

### 선택적 읽기·재개 규칙

- 이 문서를 처음 인수하거나 요청·지침이 바뀌었을 때 `P2G-READER`를 확인한다. 같은 맥락에서 이미 확인한
  지침을 매 수정·도구 호출마다 다시 읽지 않는다.
- 현재 수행할 단계의 `P2G-XX START/END`를 읽고, 다음 단계는 실제로 진입할 때 읽는다. prerequisite의
  존재·완료 여부만 확인할 때 다른 단계의 절차 전체를 미리 읽을 필요는 없다.
- artifact 종류·Ref·필드를 작성·검증할 때 `P2G-ARTIFACT-CATALOG`와 `8.0`의 해당 계약을 확인한다. 이미 읽고
  유효한 내용은 재사용한다. section 9도 실제 생성하는 artifact의 template만 읽고 전체 template를 채우지 않는다.
- Routing Table의 조건이 실제로 발생한 section만 추가로 읽는다. 현재 작업의 producer/consumer·수명·검증
  근거가 부족하면 필요한 원문까지 확장하되 무관한 저장소 전수 탐색은 하지 않는다.
- 재개 시 현재 범위·정책·artifact basis와 마지막 유효 결과·남은 문제를 복원한다. 관련 코드·데이터·지침·
  환경이 바뀌었거나 이전 근거가 모호할 때 해당 부분을 재확인한다. 관련 없는 변경만으로 전부 다시 읽지 않는다.
- 이 파이프라인 문서 자체의 편집·감사는 변경 영향에 필요한 여러 section을 읽고 교차 점검할 수 있다.
- 로드맵에서 Topic·다음 범위를 고를 때만 `8.9`, 안전한 진단·근거 재사용은 `8.5`, 중단 판단은 `8.6`을 본다.
- Section ID와 START/END marker가 안정적인 식별자다. line number는 읽은 위치를 보고하는 참고값이다.

<!-- P2G-READER END -->

## 1. 목적

정책 주제와 정책 요소를 사용자와 AI가 함께 닫고, 실제 프로젝트 코드에 근거해 구현 가능성을 검토한 뒤,
안전한 구현 순서와 Goal 구조로 변환한다.

이 문서는 다음 결과를 만드는 데까지만 책임진다.

1. 코드 검토를 거쳐 최종 동결된 정책
2. 정책 요소별 구현 검토 결과
3. 의존성과 atomic boundary가 반영된 Phase 설계
4. 구현을 위한 Single Goal 또는 Master/Phase Goal

Goal 작성은 구현 시작이나 production mutation 승인을 의미하지 않는다.

## 2. 범위

### 포함

- 임시 정책 문서 생성과 갱신
- 정책 요소 식별과 미결 정책 논의
- 정책 요소별 code-informed 구현 검토
- 코드 검토에서 발견한 정책 질문의 재개방
- 최종 정책 동결
- 구현 작업 패키지와 의존성 분석
- Single 또는 Master/Phase 구조 결정
- Goal 작성과 구현 전 handoff

### 제외

- Goal activation
- production code/data/store mutation
- 구현 검증과 completion audit
- 구현 결과물의 canonical publication 또는 production promotion
- 배포와 운영 환경 변경

위 항목은 후속 [Goal Execution Pipeline](./goal_execution_pipeline.md)에서 다룬다.

## 3. 핵심 개념

| 개념 | 의미 |
| --- | --- |
| Policy Topic | 정책 문서가 다루는 하나의 상위 주제 |
| Working Policy Document Identity | 경로와 별개로 한 Working Policy Document를 식별하는 stable Document ID와 Policy Topic identity |
| Policy Element | 사용자 직접/위임 결정으로 확정하거나 구현 결과로 보존해야 하는 개별 정책 의미 |
| Policy Intake Snapshot | P2G-00이 initial raw policy input을, P2G-10이 후속 raw policy input 변경을 의미 판정 없이 고정하는 immutable intake revision |
| Policy Element Candidate | P2G-10이 Policy Intake Snapshot과 current 정책 자료를 검토하며 식별하고 disposition을 부여하는 검토 단위 |
| Working Policy Document | immutable Working Revision의 current head와 별도 Derived Artifact Registry를 함께 보여주는 변경 가능한 작업 view |
| Working Revision | scope, current element map과 결정 등 정책 basis만 고정하는 immutable snapshot |
| Open Policy Question | 사용자 결정이 필요한 아직 닫히지 않은 정책 질문 |
| Technical Decision | 승인된 정책 의미 안에서 AI가 코드 근거로 결정할 구현 세부사항 |
| Evidence Gap | 필요한 사실을 아직 확인할 수 없는 상태 |
| Code Review Finding | 실제 코드 검토에서 확인한 current behavior, 구현 경계, 위험 또는 질문 |
| Review Batch | 하나의 Input Working Revision Ref에 대한 candidate review를 격리하고 Result Working Revision Ref·Code Review Binding Ref Set을 완전한 결과로 publish하는 P2G-20 단위 |
| Composite Basis | Working Revision, Final Freeze, Phase Plan, Goal Set과 authoritative Change Declaration Set의 순서화된 typed Exact Artifact Ref 조합 |
| Resolution Obligation | Post-Freeze Change를 수리하기 위해 owner 단계, prerequisite와 target projection을 container 안에 고정한 immutable member. exact identity는 `(Container Ref, Obligation ID)`다. |
| Change Obligation Declaration | Change, closure, 완전한 최초 obligation member set과 invalidation disposition을 함께 선언하는 immutable artifact |
| Work Package | 함께 구현해야 하는 기술 작업의 묶음 |
| Atomic Group | 나누면 유효한 중간 상태를 만들 수 없어 함께 구현해야 하는 Work Package 집합 |
| Phase | 독립적으로 구현·검증·완료할 수 있는 순서화된 실행 단위 |
| Freeze-scoped Policy Element Selector | Phase Plan의 Bound Final Policy Freeze Ref와 stable Element ID를 결합해 exact frozen member revision을 결정적으로 선택하는 축약 relation |
| Phase Plan Validation Receipt | P2G-40 사람용 설계와 분리해 selector resolution, coverage, graph와 cardinality를 exact Phase Plan/Freeze basis에서 기계적으로 검증한 immutable receipt |
| Single Goal | 최종 Phase Plan의 Phase 하나를 완료하는 구현 Goal |
| Master Goal | 구현 작업을 소유하지 않고 순차·병렬과 관계없이 여러 독립 완료 Phase Goal의 관계, 조정과 전체 완료 조건을 소유하는 Goal |
| Handoff Artifact | exact current Composite Basis와 authorization boundary를 구현 파이프라인에 전달하는 durable P2G-60 산출물 |

정책 요소는 결정 단위이고 Phase는 실행 단위다. 둘을 1:1로 간주하지 않는다.
이 표는 용어의 의미만 설명한다. 실제 산출물 이름, owner, 종류, typed Ref와 consumer는 section 5.1 Artifact
Catalog가 소유하며 두 section이 충돌하면 artifact 계약에는 5.1을 적용한다.

## 4. 정책 요소 상태

```text
Element E / Revision r1: open
  -> Element E / Revision r2: decided
  -> Element E / Revision r3: code_reviewed

Final Policy Freeze F1 --membership--> Revision r3
Final Policy Freeze F2 --membership--> Revision r3
```

| 상태 | 의미 |
| --- | --- |
| `open` | 정책 결정이 남아 있음. 사용자 질문 또는 위임 결정의 근거 확인·판정 전 상태 |
| `decided` | 현재 정보 기준으로 잠정 결정됐지만 코드 검토가 끝나지 않음 |
| `code_reviewed` | 결정 내용에 대해 필요한 코드 검토가 완료됨 |

Policy Element는 stable Element ID를 가지며, Policy Element Revision은 그 요소의 내용과 한 시점의 상태를
담은 immutable snapshot이다. 내용 또는 상태가 바뀔 때마다 predecessor를 가리키는 successor Policy Element
Revision을 만든다. 기존 revision의 상태를 바꾸거나 `open`으로 되돌리지 않는다. Working Policy Document의
current Working Revision이 요소별 current revision을 선택한다.

`frozen`은 Policy Element Revision의 상태가 아니다. 성공적으로 published된 Final Policy Freeze와 exact
Policy Element Revision 사이의 immutable Freeze Membership으로부터 파생한다.

```text
Freeze Membership := (Final Policy Freeze Ref, Policy Element Revision Ref)

is_frozen(revision) :=
  revision이 하나 이상의 published Final Policy Freeze에 포함됨
```

하나의 revision은 여러 successor Freeze에 재사용될 수 있으므로 여러 Freeze Membership을 가질 수 있다.
Freeze의 planning disposition이 나중에 `stale`, `superseded` 또는 `historical`이 되어도 published membership은
삭제되지 않는다. Freeze 이전이든 이후든 정책 basis가 바뀌면 immutable successor Working Revision을 만들고,
필요한 요소에만 successor Policy Element Revision을 만든다. 자세한 규칙은 `8.4 변경과 재검토`와 `8.7 문서
Revision과 Freeze`가 소유한다.

## 5. 문서와 산출물

<!-- P2G-ARTIFACT-CATALOG START -->
### 5.1 Artifact Catalog와 고정 규칙

이 catalog는 P2G가 생성·선택·소비하는 산출물 이름, owner, 종류와 exact identity의 유일한 색인이다. 단계
본문과 section 9 template는 여기의 canonical artifact name을 그대로 사용한다.

- catalog에 없는 항목을 새 필수 산출물이나 단계 prerequisite로 요구하지 않는다.
- 새 산출물을 추가하려면 먼저 이 catalog에 owner, 종류, Exact Ref 여부와 실제 consumer를 등록한다.
- `canonical|derived|candidate` 독립 artifact만 section 8.0의 typed Exact Artifact Ref를 가질 수 있다.
- `embedded` member/ref set은 parent payload 안에 포함하며 별도 Artifact ID나 Exact Artifact Ref를 만들지
  않는다.
- `view|external`은 current artifact에서 파생하거나 pipeline 밖에 보고하는 표현이다. 별도 Exact Artifact
  Ref를 소유하지 않는다.
- 이 catalog는 identity와 책임 경계만 고정한다. canonical payload와 lifecycle은 section 8.7, 작성 schema와
  표시 형식은 section 9가 소유한다.

#### 독립 Exact-Ref Artifact Catalog

이 표에는 parent artifact의 member나 파생 가능한 view가 아니라, producer-issued typed Exact Artifact Ref를
가지며 실제 consumer가 그 exact identity를 사용하는 artifact만 둔다.

| Canonical artifact name | Authoritative owner | 종류 | Typed exact identity | Required consumer | 고정 의미 |
| --- | --- | --- | --- | --- | --- |
| Policy Intake Snapshot | P2G-00 initial, P2G-10 successor raw-input change | canonical | Policy Intake Snapshot Ref | P2G-10, Working Revision | 의미 판정 전 사용자 raw input과 알려진 제약의 immutable intake revision |
| Working Revision | P2G-00 initial, P2G-10/20 result, section 8.4가 지정한 repair owner의 successor entry preparation | canonical | Working Revision Ref | 선택 P2G 단계와 P2G-30 | 한 시점의 exact policy basis |
| Policy Element Revision | P2G-10 `open`/`decided`, P2G-20 `code_reviewed` | canonical | Policy Element Revision Ref | Working Revision, Review Batch, Final Policy Freeze | 한 요소의 immutable statement·decision·Observable Policy Acceptance·intrinsic state |
| Decision Revision | P2G-10 | canonical | Decision Revision Ref | Working Revision, P2G-20, Final Policy Freeze | 사용자 직접 결정 또는 `8.6`의 위임에 따라 확정한 exact policy meaning |
| Open Policy Question | P2G-10 | canonical | Open Policy Question Ref | Working Revision, P2G-10 | 아직 사용자 결정이 필요한 정책 질문 |
| Candidate Policy Element Disposition Record | P2G-10 | derived | Candidate Policy Element Disposition Record Ref | 후속 P2G-10, Effective View | intake candidate의 immutable 검토 결과 |
| Review Scope/Source Requirement Plan | P2G-20 | derived | Review Scope/Source Requirement Plan Ref | Source Observation, Review Batch | exact input basis에서 필요한 source와 claim boundary |
| Source Observation | P2G-20 | derived | Source Observation Ref | Reviewed Source Basis, Review Batch | plan requirement를 exact input basis에서 평가한 결과 |
| Reviewed Source Basis | P2G-20 | derived | Reviewed Source Basis Ref | Code Review Binding, Review Batch, Final Policy Freeze | 검토에 실제 사용한 current source 선택 |
| Code Review Artifact | P2G-20 | derived | Code Review Artifact Ref | Code Review Binding, Review Batch | current actual, delta, boundary와 finding |
| Review Evidence | P2G-20 | derived | Review Evidence Ref | Code Review Binding, Review Batch | code review claim을 지지하는 immutable evidence |
| Candidate Review Envelope | P2G-20 | candidate | Candidate Review Envelope Ref | Review Batch | current publish 전 review/revalidation 후보를 격리한 envelope |
| Code Review Binding | P2G-20 | derived | Code Review Binding Ref | Review Batch, Final Policy Freeze | Working Revision·element revision·source·evidence 결속 |
| Review Batch | P2G-20 | canonical | Review Batch Ref | P2G-30 | 한 input basis의 review 결과와 binding을 완전하게 묶은 결과 |
| Post-Freeze Change Record | 변경 발견 P2G 단계, section 8.4 | canonical | Post-Freeze Change Record Ref | Change Obligation Declaration | 동결 이후 changed premise와 origin basis |
| Change Obligation Declaration | 변경 발견 P2G 단계, section 8.4 | canonical | Change Obligation Declaration Ref | stage gate, successor Working Revision | change, closure, repair obligation과 invalidation의 authoritative 선언 |
| Resolution Record | Resolution Obligation의 Owner Stage | canonical | Resolution Record Ref | stage gate와 downstream owner | Obligation Key가 지정한 의무를 해결한 exact prerequisite/result basis |
| Declaration Supersession Record | 변경 declaration 교체 owner | canonical | Declaration Supersession Record Ref | stage gate | predecessor와 successor declaration의 immutable 교체 관계 |
| Obligation Supersession Record | downstream closure를 바꾼 Owner Stage | canonical | Obligation Supersession Record Ref | stage gate와 downstream owner | obligation 집합의 완전한 분할·병합·교체 |
| Final Policy Freeze | P2G-30 | canonical | Final Policy Freeze Ref | P2G-40/50/60 | reviewed policy basis와 exact member/binding/source 동결 |
| Phase Plan | P2G-40 | canonical | Phase Plan Ref | P2G-50/60 | Work Package, dependency, Atomic Group과 ordered Phase |
| Phase Plan Validation Receipt | P2G-40 | derived | Phase Plan Validation Receipt Ref | P2G-40 Complete Stage Result, P2G-50 | exact Freeze에 대한 selector resolution, coverage, graph와 cardinality의 기계적 검증 결과 |
| Goal Set | P2G-50 | canonical | Goal Set Ref | P2G-60 | exact Phase Plan을 완전한 Single 또는 Master/Phase Goal member 구조로 변환한 authored 결과 |
| Handoff Artifact | P2G-60 | canonical | Handoff Artifact Ref | GEX-00 | exact current basis와 authorization boundary 전달 |

`candidate`는 current publication이 아니라는 lifecycle 의미다. Candidate Review Envelope처럼 후속 Review Batch가
exact candidate identity를 소비하는 경우에만 Exact Ref를 가진다.

#### Embedded member와 relation

| Canonical member/relation name | Parent owner | 독립 Ref | 의미 |
| --- | --- | --- | --- |
| Approved Metadata/Scope, Current Policy Element Revision Map, Decision Revision Ref Set, Open Policy Question Ref Set | Working Revision | 없음 | 승인 scope와 current policy basis의 ordered member selection |
| Decision Repartition Member Set (applicable한 P2G-10 transition만) | Working Revision; producer P2G-10 | 없음 | predecessor의 current Decision 의미를 보존하면서 복수의 독립 lineage로 재분할한 source/result와 coverage. Parent 내부 member이며 별도 supersession artifact를 만들지 않음 |
| Post-Freeze Change Record Ref/Authoritative Change Obligation Declaration Ref Basis | Working Revision | 없음 | current policy basis가 선택한 change와 declaration reference |
| Candidate Review Envelope Ref Set, Source Observation Ref Set, Code Review Artifact Ref Set, Review Evidence Ref Set, Policy Element Revision Ref Set, Code Review Binding Ref Set | Review Batch 또는 P2G-20 Complete Stage Result | 없음 | P2G-20의 ordered exact member list |
| Freeze Membership Set | P2G-30 Complete Stage Result | 없음 | Final Policy Freeze Ref와 Policy Element Revision Ref의 immutable relation set |
| Decision Revision Ref Set, Observable Policy Acceptance Set, Binding Completeness Evidence, Known Unknown Set | Final Policy Freeze | 없음 | 동결된 policy decision, member Policy Element Revision에서 계산한 acceptance projection, review completeness와 한계 |
| Policy Impact Closure, Artifact Invalidation Closure, Initial Obligation Member Set, Invalidated Artifact Disposition Set | Change Obligation Declaration | 없음 | change 영향, complete repair coverage와 authoritative invalidation |
| Predecessor Obligation Key Set, Successor Obligation Member Set | Obligation Supersession Record | 없음 | predecessor obligation을 대체하는 successor obligation member와 coverage |
| Resolution Record Ref Set, Obligation Supersession Record Ref Set | P2G-10 Result Working Revision current-selection boundary 또는 owner Complete Stage Result | 없음 | 같은 stage 성공 결과에서 함께 검증되는 repair member list |
| Complete Stage Result | P2G-20/30/40/50/60 stage execution | 없음 | 필수 artifact와 relation이 완전할 때만 current selection에 사용할 수 있는 semantic result member 집합 |
| Work Package Set, Freeze-scoped Policy Element Selector Coverage, Dependency Graph, applicable Cyclic SCC Disposition Set, Atomic Group Set, ordered Phase Set, Intermediate-State Audit, applicable Parallelization Evidence, Integration Validation Ownership Map, Goal Structure Fact, Known Unknown/Limitations | Phase Plan | 없음 | 구현 작업, Freeze-relative 정책 coverage, 의존성 해소, 유효한 중간 상태, 통합 검증 ownership과 독립 완료 단위의 canonical plan member |
| Goal Structure Fact, Single/Master/Phase Goal Member Set, Phase-to-Goal Member Mapping, applicable Shared Validation Baseline Reuse Contract, Authoring/Activation/Implementation Authorization Status | Goal Set | 없음 | exact Phase 구조에서 파생된 complete Goal member, 승인된 validation evidence 재사용 경계와 비실행 authoring 상태 |
| Exact Upstream Artifact Ref Set, Remaining Gap/Blocker Summary, P2G Scope/Authorization Boundary | Handoff Artifact | 없음 | current P2G 결과를 복제하지 않고 exact basis, 남은 조건과 P2G 금지선을 전달하는 canonical handoff member |

위 member/relation은 parent payload에서 stable member key로 구분한다. 별도 `... Set Ref`를 만들지 않는다.
`... Ref Set`이라는 표기는 Exact Artifact Ref member list라는 뜻이지 독립 Set
artifact라는 뜻이 아니다.

#### Derived view와 external record/report

| View 또는 external name | Owner/source | Identity | 용도 |
| --- | --- | --- | --- |
| Working Policy Document | current Working Revision과 external artifact | stable Document Identity, Content Digest 없음 | latest Current Working Checkpoint, P2G-10 completion predicate와 외부 artifact를 함께 보여주는 작업 view |
| External Artifact View / Derived Artifact Registry | 5.1의 non-Working-Revision artifact | materialized view | Working Policy Document에서 외부 artifact와 currentness 조회 |
| Document Metadata, User-Approved Scope View, Current Composite Basis | Working Revision과 current downstream artifact | projection view | identity, 승인 scope와 current exact basis 표시 |
| Policy Element Registry, Candidate Disposition Effective View | Working Revision과 disposition record | materialized view | current element와 candidate disposition 조회 |
| Finding Triage Log, Code Review Finding Registry, Open Policy Question Log, User Decision Log, Technical Decision Log, Evidence Gap Log | policy/review artifact | filtered view | 분류별 조회와 사용자 논의 지원 |
| Policy Element-to-Code Review/Phase/Goal Matrix | Review Batch, Final Policy Freeze, Phase Plan, Goal Set | optional standalone traceability view | 정방향·역방향 조회. 별도 Matrix 파일은 prerequisite가 아님. 현행 P2G-50 검증기가 읽는 Goal Set 내부 Policy–Phase–Goal Traceability는 owning relation에서 파생하는 입력 표현이며 독립 artifact·정책 authority가 아님 |
| Source Observation/Evidence Registry | Source Observation Ref Set | registry view | requirement별 current observation 조회 |
| Freeze Membership Registry, optional Binding Inclusion Index/View | Final Policy Freeze와 included Binding Ref Set | reverse-index view | membership과 binding inclusion의 선택적 역방향 조회 |
| Post-Freeze Change Log, Resolution Obligation View | declaration/supersession container | registry view | change와 stage별 Obligation Key 조회 |
| Artifact Disposition Registry, Effective Resolution Evaluation | declaration, resolution과 supersession | materialized view | lifecycle와 effective resolution 판정 표시 |
| P2G Stage Result Report, Goal Authoring Summary | 수행 단계 | external report by default | 수행·계속 진행·종료·실제 blocker 보고 |
| 실행 범위·진행 방식·종료/재개 기록 | 사용자 요청과 stage execution | 기존 작업 view/report | single_stage/bounded_pipeline의 요청 근거·끝점·제한. 새 canonical artifact·Ref·권한 authority가 아님 |

view나 report가 표시하는 값은 authoritative source와 다르면 source에서 다시 파생한다. view 갱신만으로
Working Revision successor나 새로운 canonical stage result를 만들지 않는다.

<!-- P2G-ARTIFACT-CATALOG END -->

### 5.2 Working Policy Document와 Working Revision 경계

파이프라인의 변경 가능한 작업 view다. 다음 두 change boundary를 함께 표시하지만 Working Policy Document
전체에 대해 하나의 Content Digest를 계산하지 않는다.

```text
Working Policy Document
  Current Working Revision Ref
  External Artifact View
```

Filesystem-backed repository에서 Working Policy Document는 임시 입력 문서와 분리된 정책 전용 Goal
workspace에 둔다.

```text
<Target Documentation Root>/goals/<Policy Directory Name>/WORKING_POLICY[.<locale>].md
```

- `Target Documentation Root`는 대상 repository의 canonical documentation root다.
- `Policy Directory Name`은 Policy Topic에서 할당한 stable filesystem-safe 정책 이름이다. 예:
  `conditional_wave_and_boss_summon_policy`.
- 기본 파일명은 `WORKING_POLICY.md`이며 repository가 locale suffix를 사용하는 경우
  `WORKING_POLICY.<locale>.md`를 사용할 수 있다. Initial publication 뒤에는 같은 document identity가 같은
  파일명을 유지한다.
- Working Policy Document를 `tmp/`에 만들거나 임시 정책 원문 파일을 Working Policy Document로 덮어쓰지
  않는다. 임시 원문은 Policy Intake Snapshot의 exact content/reference로만 보존한다.
- 같은 policy workspace에는 이후 이 Policy Topic에서 파생된 P2G/Goal 문서를 둘 수 있지만, 디렉터리나
  파일 경로는 Exact Artifact Ref를 대신하지 않는다.

Physical path는 mutable Working Policy Document view의 위치이며 Working Revision canonical payload가 아니다.
기존의 같은 document identity를 비표준/legacy 경로에서 위 표준 경로로 옮기는 요청은 `document-edit`로
처리한다. Source identity 일치, target 미점유와 충돌 부재를 확인한 뒤 파일을 이동하고 relocation evidence를
view에 기록하되, 정책 basis가 바뀌지 않았다면 successor Working Revision을 만들지 않는다.

`Current Working Revision Ref`는 최신 durable policy 작업 상태인 `Current Working Checkpoint`를 선택한다.
checkpoint는 사용자 논의를 다음 요청에서 정확히 재개하기 위해 `open` Policy Element Revision과 Open Policy
Question을 포함할 수 있다. current checkpoint라는 사실만으로 P2G-10 완료 또는 P2G-20 진입 자격이 생기지
않는다. `Current Working Checkpoint Ref`는 새 artifact identity가 아니라 이 역할로 선택된 exact Working
Revision Ref를 뜻한다.

`Current Complete P2G-10 Result`는 별도 artifact나 Ref가 아니라 다음 파생 판정이다.

```text
is_p2g10_complete(working_revision) :=
  working_revision이 Working Policy Document의 Current Working Checkpoint이고
  P2G-10의 모든 exit condition과 applicable repair condition을 만족함
```

P2G-20은 `is_p2g10_complete(Input Working Revision) = true`인 checkpoint만 prerequisite로 사용할 수 있다.
따라서 `current working checkpoint`와 `current complete-stage result`는 서로 다른 판정이며, incomplete
checkpoint를 후속 단계의 current prerequisite로 사용하지 않는다. incomplete successor checkpoint가 current
head로 선택되면 predecessor의 과거 complete 판정을 현재 basis의 fallback으로 사용하지 않으며, latest
checkpoint가 다시 P2G-10 exit condition을 만족할 때까지 P2G-20 진입을 막는다.

Working Revision의 canonical policy payload는 다음으로 한정한다.

- Predecessor Working Revision Ref 또는 `N/A`
- Predecessor Final Policy Freeze Ref 또는 `N/A`
- Approved metadata와 scope
- Policy Intake Snapshot Ref
- Current Policy Element Revision map
- Decision Revision Ref Set
- applicable Decision Repartition Member Set — 이 Working Revision의 predecessor→result transition에 실제 재분할이 있을 때만
- Open Policy Question Ref Set
- Post-Freeze Change/Impact와 authoritative declaration reference basis

producer는 위 payload를 고정한 Working Revision Ref를 발행한다. 같은 구현 안의 무결성 확인에 optional
Content Digest를 사용할 수 있지만 후속 consumer는 Ref를 다시 계산하지 않고 exact-match한다. Policy basis가
바뀌면 기존 revision을 수정하지 않고 successor Working Revision을 만든다.

Working Revision 밖의 항목은 5.1 Catalog의 derived/candidate artifact, embedded relation projection, view와
external record/report로 표시한다. 전체 artifact schema와 view는 section 9가 소유한다.

외부 artifact 본체는 Working Revision canonical payload에 포함하지 않는다. 다만 Working Revision이 current로
선택해야 하는 Decision, Question, Change 또는 declaration의 typed reference는 위 canonical field에 포함할 수
있다. 각 외부 artifact는 owner 단계가 요구하는 exact prerequisite Ref 또는 Input/Result Basis를 가리킨다.

외부 artifact나 materialized view만 추가·갱신해도 Working Revision을 새로 만들지 않는다. 조사 결과가 승인
scope, Policy Element statement, Decision 또는 Open Policy Question basis를 바꿔야 함을 드러낸 경우에만 해당
owner 단계에서 successor Working Revision을 만든다.

### 5.3 Final Policy Freeze

모든 in-scope current Policy Element Revision이 `code_reviewed`이고 P2G-30 검증을 통과할 때 complete result로 publish하는
변경 불가능한 정책 artifact다. 검증 중인 candidate는 Final Policy Freeze가 아니며 Freeze Membership을 만들지
않는다. P2G-30의 필수 구성 검증이 완료되면 Final Freeze, exact member Revision Set에 대한 Freeze Membership과
applicable P2G-30 Resolution Record Ref Set을 하나의 complete result로 publish한다. 이때 Freeze는 exact
Working Revision Ref, exact code-reviewed Policy Element Revision Ref Set, Current Code Review Binding Ref Set,
Reviewed Source Basis Ref와 Binding Completeness Evidence를 고정한다. 후속 변경은 이 artifact를 수정하지
않고 explicit predecessor link를 가진 successor Working Revision에서 수행한다.

### 5.4 Dependency and Phase Plan

Phase Plan은 exact current Final Policy Freeze와 prerequisite Composite Basis 하나를 입력 기준으로 고정하고,
Work Package, dependency edge, cyclic SCC disposition, Atomic Group, ordered Phase와 각 중간 상태의 유효성을
기록한다. dependency type과 graph 분석 알고리즘은 폐쇄된 enum이나 강제 절차가 아니다. Phase Plan의
canonical member는 P2G-40 section과 section 9 template가 정하며 별도 SCC, audit 또는 parallelization
artifact를 만들지 않는다.

Phase Plan은 Freeze의 Policy Element Revision Ref Set, Decision Revision Ref Set 또는 Observable Policy
Acceptance 본문을 다시 복제하지 않는다. Work Package와 Phase는 top-level `Bound Final Policy Freeze Ref`에
상대적인 stable Element ID selector를 기록한다.

```text
Freeze-scoped Policy Element Selector :=
  (Phase Plan의 Bound Final Policy Freeze Ref, Stable Element ID)
```

selector가 가리키는 exact Policy Element Revision Ref는 Bound Freeze의 member set에서, Decision Revision Ref와
Observable Policy Acceptance는 그 exact member revision에서 결정적으로 해석한다. 한 selector가 없거나 둘
이상의 Freeze member로 해석되면 Phase Plan은 invalid다. 사람용 Phase Plan에는 stable Element ID와 의미 있는
ownership 관계만 표시하고, exact expansion·중복·coverage·cardinality·graph 검사는 별도 immutable Phase Plan
Validation Receipt가 소유한다.

Validation Receipt는 exact candidate Phase Plan Ref와 Bound Final Policy Freeze Ref를 가리키며 최소한 selector
resolution, Freeze member primary-owner coverage, Work Package/Atomic Group partition, dependency/SCC/condensation
graph, Final Execution Node-to-Phase cardinality, required audit/validation-owner row completeness와 Goal Structure
Fact를 기계적으로 검증한다. receipt는 Work Package 경계, dependency 이유, intermediate-state validity 또는
validation seam의 적절성을 대신 판단하지 않는다. 이 의미 판단과 근거는 Phase Plan 본문이 계속 소유한다.
receipt의 full payload는 repository 표준 생성기가 만드는 machine-readable JSON companion으로 둔다. 사람용
Phase Plan view에는 Receipt Ref, 검사 method/version, overall result, 핵심 count와 failure summary만 표시한다.
passed detail을 사람용 표로 다시 전개하거나 JSON을 수동으로 전사하지 않는다. 생성기는 Phase Plan의 의미적
설계 판단을 대신하지 않으며 production source/data 또는 game test를 건드리지 않는다.

cyclic SCC가 없으면 별도 empty SCC 표를 만들지 않고 receipt에 `cyclic_scc_count: 0`을 기록한다. 병렬 Phase를
승인하지 않으면 상세 Parallelization Evidence 표 대신 Phase Plan에 `Parallelization: none approved` 한 번만
기록한다. 단일 Phase도 같은 canonical 의미를 유지하되 ordered Phase와 Intermediate-State Audit을 하나의
축약 row로 표시할 수 있다. 이러한 축약은 dependency, atomic boundary, 유효 상태 또는 validation ownership
검사를 생략한다는 뜻이 아니다.

### 5.5 Goal Artifacts

Phase 설계에 따라 다음 중 하나를 작성한다.

- Single Goal 하나
- Master Goal 하나와 ordered Phase Goal 집합

### 5.6 Post-Freeze Change와 Artifact Disposition

동결 이후 변경은 기존 Freeze나 산출물을 수정하지 않고 Post-Freeze Change Record에 기록한다. Change
Obligation Declaration이 Change, closure, complete initial obligation member set과 Invalidated Artifact Disposition
Set을 함께 고정하며 invalidation의 authoritative owner다. Artifact Disposition Registry는 declaration과
후속 resolution에서 계산한 materialized view일 뿐이다. 수리 작업은 declaration 또는 supersession container에
포함된 stage별 Resolution Obligation member와 prerequisite
graph로 표현하고, exact 결과 basis는 Resolution Record가 소유한다. downstream closure가 바뀌면 집합 단위
Obligation Supersession Record를 사용한다.

## 6. 전체 흐름

```text
요청의 Topic·범위·종료 결과·진행 방식 확인
  -> 필요한 가장 이른 current하지 않은 단계 선택
  -> P2G-00 initial scope와 raw intake 고정 (신규일 때)
  -> P2G-10 정책 검토·기존 승인 결속
       -> 새 material question이면 결정 지점에서 중단
       -> 질문 해결 뒤 같은 승인된 연속 진행 요청 재개
  -> P2G-20 code-informed 검토
       -> non-material 오류는 owner 단계에서 수리·재검증
       -> 새 정책은 P2G-10으로 반환, 8.6에 따라 위임 결정 또는 사용자 질문
  -> P2G-30 exact reviewed policy 동결
  -> P2G-40 Work Package·dependency·Atomic Group·Phase 검증
  -> P2G-50 complete Goal Set 작성·검증
  -> P2G-60 Handoff (요청한 종료 결과에 포함된 경우에만)
  -> 요청 완료 보고; GEX/production 구현으로 자동 전환하지 않음
```

`single_stage`는 지정한 한 단계에서 종료한다. `bounded_pipeline`은 승인된 끝점까지 각 단계의 entry/exit를
검증하며 진행하고, 끝점 전에는 새로운 결정·권한·필수 근거 문제나 명시적 checkpoint가 있을 때만 멈춘다.
이 흐름은 stage owner와 complete publication을 합치는 예외가 아니다. 권한은 사용자 요청에서, 다음 단계의
진입 자격은 실제 current prerequisite에서 각각 확인한다.

## 7. 단계별 골격

<!-- P2G-00 START -->
### P2G-00 — 입력과 작업 범위 고정

#### 목적

새 Policy Topic의 정책 검토를 시작하기 전에 사용자 승인 initial scope와 요청 결과를 고정하고, 사용자가
제공한 임시 정책 입력을 의미 판정 없이 보존한 새 Working Policy Document와 immutable initial Working
Revision을 초기화한다. 기존 Working Policy Document의 scope나 policy basis를 수정하는 단계가 아니다.

#### 입력

- 사용자가 지정한 Policy Topic
- 사용자가 요청한 직접 결과인 Requested Result
- 요청했거나 새로 할당할 stable Working Policy Document ID/identity
- Target Documentation Root
- 요청했거나 Policy Topic에서 새로 할당할 stable filesystem-safe Policy Directory Name
- section 5.2의 경로 규칙으로 도출한 새 Working Policy Document physical path와 선택한 locale suffix
- 대상 프로젝트 또는 repository
- 사용자가 제공한 임시 정책 원문 또는 exact content reference
- 사용자가 제공한 참고 자료 위치와 알려진 제약
- 사용자가 승인한 initial in-scope/out-of-scope 또는 승인을 받을 수 있는 AI 제안 scope

#### 진입 조건

- 새 Policy Topic 초기화를 직접 요청했거나, 동일한 승인 scope의 `bounded_pipeline` 종료 결과에 신규 초기화가
  실제 prerequisite로 필요함. 요청의 최초 scope가 불명확하면 초기화 전에 필요한 범위를 확인함
- requested physical path가
  `<Target Documentation Root>/goals/<Policy Directory Name>/WORKING_POLICY[.<locale>].md` 규칙을 만족함
- requested physical path가 미점유 상태임
- requested document identity를 사용하는 기존 Working Policy Document나 current Working Revision이 없음
- requested document identity, 생성 경로와 대상 repository identity를 서로 구분할 수 있음

requested path가 점유됐거나 requested document identity가 이미 사용 중이면 신규 초기화 조건을 충족하지
않는다. 해당 artifact와 revision을 덮어쓰거나 수정하지 않고 Routing Table의 두 축을 판정한다. 위임된
단순 이름·경로 할당으로 해결 가능한 경우 안전한 경로를 다시 검증하고 계속할 수 있지만, 고정 경로·의도·
소유 충돌은 필요한 입력을 보고하고 멈춘다.

- `same_identity_same_path`: 변경 요청이면 P2G-10, 변경이 없으면 existing/current 상태 보고
- `path_occupied_by_different_identity`: 위임된 이름 할당이면 unique Policy Directory Name 아래의 conforming
  미점유 경로를 선택·기록. 위임되지 않은 고정 경로는 변경 승인을 요청. identity 변경만으로 해결하지 않음
- `identity_already_exists_at_other_path`: 동일 문서이면 기존 경로 사용. 별개 문서 생성·이름 할당이 승인돼 있으면
  unique identity와 conforming 미점유 경로를 배정. 동일성 의도가 불명확하면 확인 요청
- `identity_or_path_indeterminate`: requested identity/path 확인 요구

requested path가 section 5.2의 규칙 밖이면 `nonconforming_policy_workspace_path`로 기록한다. 경로 할당이
위임돼 있고 승인된 root가 분명하면 적합한 미점유 경로를 선택해 entry를 다시 확인한다. 사용자가 경로를
고정했거나 root/identity가 불명확하면 파일을 쓰지 않고 필요한 확인을 요청한다. 이미 존재하는 문서의 이동은
별도 `document-edit` relocation 범위이며, 신규 이름 할당의 재량으로 기존 문서를 옮기지 않는다.

둘 이상의 충돌이 동시에 성립하면 하나를 임의로 우선하지 않고 모든 충돌 축과 각각 필요한 입력 변경을
P2G-00 Result Report에 기록한다.

이 entry failure의 P2G-00 Result Report는 사용자 응답 또는 별도 실행 기록으로만 남기며 발견한 기존 Working Policy
Document에 기록하지 않는다.

#### 주요 활동

1. Target Documentation Root와 Policy Directory Name을 고정하고 section 5.2 규칙으로 physical path를
   도출한다. Path 규칙 준수 여부와 점유 여부, requested document identity의 기존 사용 여부 및 대상 repository
   identity를 각각 확인한다. 충돌이 있으면 각 축의 evidence를 분리해 위 무수정 종료 규칙을 적용한다.
2. Requested Result와 Reader Contract의 진행 방식·종료 결과를 확인한다. 이 단계는 초기화 artifact만 만들고
   자기 exit를 검증한다. 이후 진행 여부는 별도로 기록된 승인 범위와 다음 단계 entry로 판단한다.
3. AI Proposed Scope와 User-Approved Initial Scope를 분리한다. AI proposal은 협의 중 transient 응답일 뿐
   성공한 P2G-00 artifact가 아니다. 사용자가 승인하지 않은 in-scope/out-of-scope 항목이 있으면 initial
   Working Revision을 publish하지 않고 필요한 scope 결정을 보고한다. 각 승인된 out-of-scope 항목에는 exact
   사용자 승인 reference를 기록한다.
4. 기존 정책, 코드와 관련 Goal 자료의 확인을 identity, 위치와 bootstrap 접근 가능성 확인으로 제한한다.
   정책 의미, completeness, 모순, 중복, material question, 결정 내용, runtime behavior, source necessity 또는
   implementation delta는 검토하지 않는다.
5. 사용자 임시 정책 원문과 제공된 참고 위치를 immutable Policy Intake Snapshot으로 고정한다. canonical
   snapshot은 raw content 또는 exact content reference, user-provided source locator, user-stated
   constraint와 capture limitation만 포함한다. 접근 상태는 snapshot 생성 시의 bootstrap capture 사실일 뿐
   current source claim이나 P2G-20 evidence로 사용하지 않는다.
6. `8.0`과 `8.7`의 payload와 Exact Artifact Ref 규칙에 따라 다음을 가진 candidate initial Working Revision을
   구성한다.

```text
Predecessor Working Revision: N/A
Predecessor Final Policy Freeze: N/A
Working Policy Document ID/identity
Policy Topic
Requested Result
Target Project/Repository identity
User-Approved Initial Scope와 approval reference
Policy Intake Snapshot Ref
Current Policy Element Revision map: empty
Decision Revision Ref Set: empty
Open Policy Question Ref Set: empty
Post-Freeze Change/Closure basis: N/A
```

7. producer-issued Policy Intake Snapshot Ref와 candidate initial Working Revision Ref의 상호 참조와 필수
   payload를 검증한다. 이때 Candidate/Disposition Registry, Source Requirement/Observation,
   Policy Element Revision 또는 뒤 단계 artifact를 만들지 않는다.
8. section 5.2의 정책 전용 Goal workspace에 새 Working Policy Document를 publish하고 initial Working
   Revision을 current head로 선택한다.
9. 수행 결과와 exact 생성 artifact를 P2G-00 Result Report에 기록한다. `single_stage` 또는 종료/중단 경계면
   사용자에게 요약하고 멈춘다. `bounded_pipeline`에서 후속 결과가 남고 entry가 충족되면 다음 단계로 계속한다.
   이 report는 기본적으로 canonical/derived registry에 편입하지 않으며 연속 진행의 중간 report를 매번 사용자
   승인 요청으로 보내지 않는다.

#### 산출물

- 새 Working Policy Document
- immutable Policy Intake Snapshot Ref
- immutable initial Working Revision Ref
- P2G-00 Result Report — 기본적으로 외부 사용자 응답

#### 종료 조건

- 새 document 초기화 진입 조건을 만족했고 기존 Working Policy Document를 덮어쓰지 않았음
- Working Policy Document가 Target Documentation Root 아래
  `goals/<Policy Directory Name>/WORKING_POLICY[.<locale>].md`에 있고 `tmp/` 또는 임시 정책 원문 경로를
  재사용하지 않았음
- Policy Topic, Requested Result와 Target Project/Repository identity가 명시됨
- 모든 initial in-scope/out-of-scope 항목에 exact 사용자 승인 reference가 있음
- 승인되지 않은 proposed/pending scope가 successful initial Working Revision에 포함되지 않음
- immutable Policy Intake Snapshot이 raw input/reference와 known bootstrap limitation을 의미 판정 없이 고정함
- Policy Intake Snapshot Ref의 필수 member와 producer-issued identity가 검증됨
- `8.7`의 payload를 가진 immutable initial Working Revision Ref가 존재하고 새 Working
  Policy Document가 이를 current head로 선택함
- initial Working Revision의 Policy Intake Snapshot Ref가 산출된 exact Policy Intake Snapshot Ref와 일치함
- initial current element map, Decision Revision Ref Set과 Open Policy Question Ref Set이 empty임
- P2G-00이 Candidate/Disposition Registry, Source Requirement/Observation, Policy Element Revision 또는 다른
  P2G stage artifact를 생성하지 않았음
- Result Report에 수행 단계·충족한 exit, 다음 단계 또는 종료 이유, 현재 실행 범위에서의 진행 가능 여부와
  실제로 필요한 사용자 입력을 기록함. `New User Request Required`를 무조건 Yes로 설정하지 않음

참고 위치에 접근할 수 없거나 capture limitation이 있어도 raw input 자체와 승인 scope를 고정할 수 있으면
P2G-00은 종료할 수 있다. 다만 해당 source가 필요한 정책 식별 또는 code-informed claim은 닫힌 것으로 간주하지
않으며 P2G-10 또는 P2G-20에서 evidence gap/blocker로 처리한다.

P2G-00의 scope/intake artifact 자체는 후속 단계 권한을 부여하지 않는다. 초기화 동안에는 원문의 의미를
판정하지 않고 raw snapshot으로 보존한다. 다만 원래 사용자 요청이 후속 단계까지 포함한 `bounded_pipeline`이면
P2G-00의 complete 결과를 확인한 뒤 그 **기존 요청의 권한**으로 P2G-10에 진입할 수 있다.

#### bootstrap 표현 경계

- `current`는 사용자가 제공한 raw input/reference, 대상 repository identity와 P2G-00에서 관찰한 접근 가능성이다.
- `target`은 사용자가 요청하고 승인한 Policy Topic, Requested Result와 initial scope다.
- `proposal`은 AI가 제안했지만 아직 승인되지 않은 scope나 해석이며 successful initial Working Revision에
  포함하지 않는다.
- 사용자가 raw content를 직접 제공하면 Policy Intake Snapshot에 그대로 고정한다. 외부 content reference를
  사용하면 repository/source identity와 그 content를 다시 식별할 수 있는 exact revision 또는 content
  identifier를 함께 기록한다.
- mutable locator나 접근 불가능한 위치만 제공돼 exact content를 고정할 수 없으면 locator와 capture limitation을
  보존하되 content를 확인한 것으로 표현하지 않는다. 그 내용이 필요한 P2G-10/20 claim은 후속 evidence
  gap이며 P2G-00이 의미를 추정해 보완하지 않는다.

<!-- P2G-00 END -->

<!-- P2G-10 START -->
### P2G-10 — 정책 요소 검토와 잠정 폐쇄

#### 목적

정책 요소를 검토하고 material policy를 모두 잠정 결정한다. `8.6`의 통상적 정책은 위임 범위에서 AI가
결정하고, 사용자 선택이 필요한 정책은 실제 답변을 받아 닫는다.

#### 입력

- Working Policy Document와 exact current Working Revision Ref
- Input Working Revision이 선택한 exact current Policy Intake Snapshot Ref
- 기존 Policy Element Registry와 Candidate Policy Element Disposition Record Ref Set/Effective View가 있으면 그 exact current set
- 최초 P2G-10이면 Policy Element Registry와 Candidate Policy Element Disposition Record Ref Set/Effective View는 absent 또는 empty일 수 있음
- 기존 문서의 scope/policy basis 변경 요청이면 exact user-approved target scope/change request
- 사용자 raw policy input이 바뀌었다면 새 raw content 또는 exact content reference와 capture limitation
- 기존 결정과 정책 자료

`P2G-10 Evaluation Basis`는 다음 세 항목을 묶어 부르는 절차상 basis다.

```text
P2G-10 Evaluation Basis
  Input Working Revision Ref
  Exact user-approved target scope/change request 또는 N/A
  Input 또는 successor Policy Intake Snapshot Ref
```

이는 독립 artifact가 아니며 별도 Artifact ID, Exact Ref 또는 Content Digest를 만들지 않는다. scope만 바뀌고
raw input은 그대로면 Input Working Revision이 선택한 snapshot을 사용한다. raw input 자체가 바뀌면 P2G-10이
의미 판정 전에 immutable successor Policy Intake Snapshot을 만들고 그 Ref를 evaluation basis에서 선택한다.
두 경우 모두 기존 Working Revision과 기존 snapshot은 수정하지 않는다.

정책 basis만 보완하는 통상적 결정이면 `8.6`의 실제 사용자 위임과 해당 변경의 조건 충족 근거를 위
user-approved change request의 근거로 연결할 수 있다. initial/변경 scope를 임의로 승인하거나 AI의
결정 내용을 사용자 raw input으로 기록하지 않는다. 기존 scope를 바꾸려면 별도 사용자 승인이 필요하다.

#### 진입 조건

- 제공된 current Policy Intake Snapshot Ref가 Input Working Revision이 선택한 Ref와 exact-match함. raw input
  변경 요청에서 새로 만들 successor snapshot은 current snapshot을 대체한 입력인 것처럼 취급하지 않고 위
  Evaluation Basis의 별도 선택 항목으로 검증함
- 기존 Final Policy Freeze에 대한 새 scope/policy basis 변경 요청 자체가 아직 declared Change가 아니라면,
  P2G-10 본활동 전에 `8.4`의 entry preparation으로 Post-Freeze Change, closure, complete obligation member set과
  authoritative Change Obligation Declaration을 완전하게 publish함. valid current declaration이 없으면 P2G-10에
  진입하지 않음. 현재 요청이 동결된 정책 변경을 명시적으로 승인한 P2G-10 실행 요청이면 이 entry
  preparation과 이어지는 P2G-10 본활동을 같은 요청 안에서 수행하며 두 번째 P2G-10 승인을 요구하지 않음
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-10보다 앞선 owner stage는 없으므로 earlier-stage active leaf gate는 없음
- P2G-10 소유 unresolved active leaf는 이 단계의 repair scope로 허용됨

#### 주요 활동

1. exact Input Working Revision과 사용자 승인 change request를 고정한다. raw policy input이 바뀌었다면 기존
   snapshot을 수정하지 않고 입력 원문·reference와 알려진 capture limitation을 의미 판정 없이 고정한
   immutable successor Policy Intake Snapshot을 먼저 만든다. 변경이 없다면 Input Working Revision이 선택한
   current snapshot을 그대로 사용한다. 이 세 입력으로 `P2G-10 Evaluation Basis`를 확정한다.
2. Evaluation Basis가 선택한 exact Policy Intake Snapshot과 현재 정책 자료를 기준으로 Policy Topic을 구성할 Policy Element Candidate를
   P2G-10에서 처음 식별하고 Candidate ID와 exact source span/reference를 부여한다. 원본 snapshot과 기존
   immutable disposition record는 수정하지 않는다. 정식 요소라고 판정한 candidate만 stable Element ID와
   initial `open` Policy Element Revision으로 승격하고 새 immutable disposition record에 `promoted`로 기록한다.
   나머지 candidate에는 `duplicate|out_of_scope|not_a_policy_element`와 evidence/decision reference를 남기며
   정식 Registry에 넣지 않는다. 각 disposition은 자신을 평가한 exact P2G-10 Evaluation Basis의 세 구성
   항목을 기록한다. 최초 실행에 disposition record/view가 없는 것은 정상 입력이다.
3. 발견 사항마다 `8.1 정책 발견 사항 분류와 결정 권한`을 적용해 하나의 primary classification을 기록한다.
4. `material_policy`마다 `8.6`의 통상적 정책 기준과 위임 근거를 확인한다. 충족하면 AI가 결정하고 근거를
   기록한다. 사용자 선택이 필요한 경우에만 서로 독립된 최소 질문으로 나누어 Open Policy Question을 만든다.
   위임 결정에 형식적인 질문·사용자 답변을 만들지 않는다. 다음 5–6은 실제 질문이 있는 경우에 적용한다.
5. 각 질문에는 affected element, 결정할 의미 하나, 선택지와 observable impact를 기록한다. 새 `open`
   Policy Element Revision과 Open Policy Question Ref Set을 고정할 immutable successor Working Revision을
   구성하고 내부 reference가 일치하면 Current Working Checkpoint로 선택한다. 이 checkpoint는 사용자 논의를
   재개하기 위한 durable 작업 상태이며 아직 P2G-10 complete result가 아니다.
6. identity, 처리 순서, compatibility, data loss, persistence lifetime, migration과 rollout이 서로 독립된
   선택이면 한 질문에 묶지 않는다.
7. 사용자 직접 결정 또는 위임 범위의 AI 결정을 구분하고, exact decision content와 요소별
   `Observable Policy Acceptance`를 Decision Revision Ref와 함께 기록한다. 실제 질문은 사용자와 논의하여
   답변을 받은 뒤 닫는다. 통상적 정책은 `8.6`의 위임 승인·판단 근거를 연결하며 사용자 직접 답변으로
   기록하지 않는다. Acceptance는 외부에서 정책 의미의 충족 여부를 판정할 수 있어야 하며 결정과 모순되거나
   구현 수단 자체를 강제해서는 안 된다.
8. 결정이 닫힌 요소에는 current revision을 predecessor로 하고 policy statement, Decision Revision Ref,
   Observable Policy Acceptance와 intrinsic state `decided`를 고정한 candidate immutable Policy Element
   Revision을 만든다. 해당 검토 묶음의 policy basis 변경을 고정할 하나의 immutable candidate successor
   Working Revision을 구성한다. Candidate Result Working Revision은 current Decision Revision Ref Set을
   다음처럼 갱신하고, 해결된 Open Policy
   Question Ref를 current Open Policy Question Ref Set에서 제외하며, current element map에서 새 `decided`
   revision을 선택한다. 이 checkpoint 자체의 필수 reference가 일치하면 Working Policy Document의 current
   head로 선택할 수 있지만, 다른 Open Policy Question이 남아 있으면 P2G-10 complete result나 P2G-20
   prerequisite가 아니다.

```text
Initial Decision — Predecessor Decision Revision Ref = N/A:
  Current Decision Revision Ref Set += new Decision Revision Ref

Successor Decision:
  require predecessor Decision Revision Ref가 해당 transition 직전 Input Working Checkpoint의 current set member
  Current Decision Revision Ref Set
    -= predecessor Decision Revision Ref
    += successor Decision Revision Ref

Repartition Decision — 아래 “Decision 재분할” 계약 적용:
  require source refs가 transition 직전 current set member
  require result refs가 의미 보존된 새 독립 lineage의 initial Decision
  Current Decision Revision Ref Set
    -= complete source Decision Revision Ref Set
    += complete result Decision Revision Ref Set
  affected current Policy Element Revision의 Decision 참조도 같은 checkpoint에서 갱신

Unaffected Decision Revision Ref:
  exact preserve
```

   일반 successor는 하나의 독립 Decision 의미 단위를 완전히 대체한다. 일부 affected element/claim만
   바꿔야 하면 아래 “Decision 재분할”로 기존 의미를 보존한 독립 lineage를 먼저 만들고, 변경할 lineage에
   사용자 승인 successor를 적용한다. 두 transition은 같은 P2G-10 요청 안에서 수행할 수 있으나 각 transition의
   Input은 직전 current checkpoint다. 기존 Question·Decision artifact는 history로 보존한다.
9. 새 결정의 영향을 받는 Policy Element와 기존 결정을 다시 검토한다.
10. 미결 질문이 생기면 현재 단계 안에서 사용자와 논의하고 이 절차를 반복한다. current element map,
    Decision Revision Ref Set 또는 Open Policy Question Ref Set이 바뀔 때마다 기존 checkpoint를 수정하지 않고
    immutable successor Working Revision을 만들며, 자체 필수 reference가 검증된 최신 revision을 Current
    Working Checkpoint로 선택한다. 같은 요청의 다음 decision/repartition transition은 그 최신 checkpoint를
    Input으로 삼는다. 새 candidate를 평가할 때 그 Input에 맞는 Evaluation Basis를 고정하며, 이전 candidate나
    disposition이 기록한 과거 Evaluation Basis를 소급 수정하지 않는다.
11. 정상·repair 경로 모두 최신 Current Working Checkpoint의 Policy Element/Decision Revision, 필수
    cross-reference와 종료 조건을 검증한다. repair이면 successor Working Revision Ref/Decision Revision Ref
    Set과 exact prerequisite/result basis를 가리키는 candidate P2G-10 Resolution Record도 함께 검증한다. 모든
    조건을 만족하면 해당 checkpoint를 `Current Complete P2G-10 Result`로 판정하며, repair이면 같은 성공
    결과에서 Resolution Record도 effective하게 한다. 그 전의 checkpoint와 candidate record/revision은
    P2G-20 또는 다른 후속 단계의 current prerequisite로 사용하지 않는다.
12. 기존 문서의 scope 변경이면 exact 사용자 승인 target scope/change request와 영향받는 policy basis를 검토한다. Final Policy
    Freeze 전 변경은 `8.7`의 immutable successor Working Revision에 반영한다. Freeze 이후 변경은 entry
    preparation에서 current로 선택된 `8.4` declaration basis를 사용해 `8.7`의 successor Working Revision에 반영한다.
    후속 P2G-20 이후는 Reader Contract의 승인된 종료 결과에 포함되고 해당 entry가 충족될 때만 진행한다.
    P2G-10의 결과 자체에서 새로운 진행 권한을 파생하지 않는다.

질문은 구현 방식을 대신 선택하게 해서는 안 되며, 사용자가 경험하거나 의존하는 의미와 선택지별 영향을
설명해야 한다. AI의 추천, 위임 범위에서 확정한 AI 결정과 사용자 직접 결정을 구분해 기록한다.

#### 산출물

- 갱신된 Policy Element Registry
- Produced immutable Candidate Policy Element Disposition Record Ref Set — P2G-10이 식별한 candidate와 exact
  intake/source traceability를 함께 포함
- Result Current Candidate Policy Element Disposition Effective View
- candidate-to-promoted-element 승격 traceability
- raw input 변경 시 immutable successor Policy Intake Snapshot Ref
- Produced `decided` Policy Element Revision Ref Set과 각 revision의 Observable Policy Acceptance
- 변경된 policy basis와 미결 질문을 고정한 latest Current Working Checkpoint Ref
- 성공 종료 시 `is_p2g10_complete(Current Working Checkpoint) = true` 판정 evidence — 별도 artifact/Ref 아님
- Finding Triage Log
- Open Policy Question Log
- User Decision Log
- 해당 시 Technical Decision, Evidence Gap과 Out-of-Scope Finding 기록
- repair이면 P2G-10 Resolution Record Ref

#### 종료 조건

다음을 모두 만족해야 한다.

1. 현재 이용 가능한 정책 자료와 명시된 scope를 기준으로 모든 in-scope Policy Element가 식별돼 있다.
2. 모든 발견 사항에 `8.1`의 primary classification과 disposition이 있다.
3. 모든 `material_policy`에 exact decision과 Decision Revision Ref가 있고, 사용자 직접 승인 또는 `8.6`의
   통상적 정책 위임 승인·조건 충족 근거가 연결돼 있다. 추천만 있는 항목은 결정 완료로 세지 않는다.
4. 사용자 결정을 요구하는 Open Policy Question이 남아 있지 않다.
5. 현재 결정끼리 모순되지 않고, current set에서 선택한 각 active Decision lineage에는 정확히 하나의 leaf만
   있다. 재분할로 대체된 lineage는 이 target ancestry에서 current member가 0개이며 history로 보존된다.
   일반 predecessor/successor 또는 재분할 source/result를 동시에 current로 선택하지 않는다. 모든 current
   `decided|code_reviewed` Policy Element Revision의 Decision Ref가 current set에 있고, 적용한 Decision Repartition
   Member의 의미 보존·전체 coverage·element 재결속 검증이 통과했다.
6. `out_of_scope` finding이 현재 scope를 암묵적으로 확장하지 않는다.
7. `evidence_gap`을 current, compatible, safe 또는 pass로 간주하지 않는다.
8. 모든 in-scope Policy Element가 최소 `decided`다.
9. 모든 in-scope `decided` Policy Element Revision에 비어 있지 않고 해당 Decision Revision과 일치하는
   unambiguous Observable Policy Acceptance가 있다.
10. 현재 P2G-10이 식별한 모든 Policy Element Candidate에 정확히 하나의 effective current disposition
   record가 있고, `promoted` candidate만 stable Element ID와 initial `open` revision을 가진다.
11. 각 disposition이 exact P2G-10 Evaluation Basis의 Input Working Revision Ref, 승인 change request 또는
    N/A, 선택한 Policy Intake Snapshot Ref를 기록한다.
12. raw input 변경 시 successor Policy Intake Snapshot이 immutable하게 생성되고 Result Working Revision이
    그 exact Ref를 선택한다. raw input이 바뀌지 않았다면 current snapshot Ref가 유지된다.
13. 모든 applicable Change에 valid authoritative declaration이 존재한다.
14. repair이면 P2G-10이 소유한 모든 applicable active obligation leaf가 successor Working
    Revision Ref/Decision Revision Ref Set으로 effectively resolved됐다.
15. exact Current Working Checkpoint의 모든 필수 결과와 cross-reference가 검증됐고
    `is_p2g10_complete(Current Working Checkpoint) = true`이며, repair이면 같은 성공 결과의 Resolution
    Record가 effective하다.

이 단계의 종료는 코드 검토 완료, Final Policy Freeze, Phase 설계, Goal 작성 또는 구현 승인을 의미하지 않는다.

#### 되돌림

- 미결 정책이 남아 있으면 현재 단계 안에서 이 절차를 반복한다.
- 새 결정이 current Policy Element Revision의 전제를 바꾸면 원본을 보존하고 같은 Element ID의 successor
  Policy Element Revision을 `open`으로 생성한다. 이후 결정이 닫히면 또 다른 immutable successor revision을
  `decided` 상태로 만든다. 각 current element map 또는 Decision/Open Question basis 변경은 기존 Working
  Revision을 수정하지 않고 immutable successor Working Revision으로 publish하고 Current Working Checkpoint로
  선택한다. checkpoint에 Open Policy Question이 남아 있으면 P2G-10 complete result로 판정하지 않는다.
- predecessor Freeze의 member revision을 상속한 successor Working Revision에서도 원본 revision이나 Freeze
  Membership을 변경하지 않고 같은 successor element revision 규칙을 적용한다.
- 정책 결정에 필수인 evidence가 없으면 사실을 추측하지 않고 `evidence_gap`으로 남긴다.

#### 정책 의사결정 지원용 제한적 사실 확인

P2G-10에서 정책 분류·결정에 필요한 사실이 부족할 때는 해당 판단을 가능하게 만드는 범위의 read-only
code/data/문서 확인을 수행할 수 있다. 이것은 P2G-20 실행이나 정식 code-informed review의 선행 완료가 아니다.
이 허용은 사용자 의논이 필요한 정책인지 판별하는 동안에만 적용한다. 의논 필요가 확인되면 `8.6`에 따라
주 작업과 정책에 의존하거나 의존성이 불분명한 작업을 중단하고, 이미 확보한 근거와 남은 unknown을
질문에 적어 답변을 기다린다. 이미 실행 중인 독립 작업은 `8.6`의 기존 범위 완료 조건을 따른다.

- 조사 전에 연결할 Open Policy Question 또는 candidate, 필요한 사실과 조사 종료 조건을 적는다. 저장 shape,
  현재 owner, 실제 reader/writer, 결정에 영향을 주는 호출·참조 같은 좁은 사실을 확인할 수 있다.
- exact source identity/revision와 relevant dirty 상태, 읽은 구간, 방법, 관찰 사실, 미확인 범위를 기존 Finding
  Triage Log/Evidence Gap Log의 근거 또는 external 응답에 기록한다. 별도 Exact-Ref artifact는 만들지 않는다.
- `inspect` mode 또는 이전 조사 결과도 의사결정 보조 근거로 읽을 수 있다. 그 요청 자체를 P2G 단계 실행,
  승인 또는 current Code Review Binding으로 승격하지 않는다.
- 전체 구현 가능성·변경 경계·consumer coverage를 판정하는 P2G-20 업무를 이 경로로 대신하지 않는다.
  Source Observation, Review Batch, formal `code_reviewed` revision과 Code Review Binding은 만들지 않는다.
- 허용 범위는 읽기와 이미 제공된 진단 결과 확인이다. build/test/probe를 새로 실행하거나 production 파일·
  store를 바꾸지 않는다. 결정에 실험이 필요하면 필요한 별도 조사·실험의 범위와 권한을 보고한다.
- 필요한 사실이 확보되면 조사를 멈추고 같은 P2G-10에서 `8.6`에 따라 위임 결정 또는 사용자 의논으로
  처리한다. 사실이 여전히 unknown이면 질문을 추정으로 닫거나 “나중에 P2G-20이 확인한다”는 조건만 붙여
  P2G-10을 완료하지 않는다.
- 이후 정식 P2G-20은 자기 exact input/plan/source basis를 평가한다. 재사용할 원시 근거가 있어도 P2G-20이
  필요한 currentness와 coverage를 확인하고 자신의 artifact에 결속해야 한다.

#### Decision 재분할 — 의미 보존과 current-selection

일반 `Predecessor Decision Revision Ref`는 1:1 lineage 교체다. 복합 결정을 나누기 위해 같은 predecessor에
복수 successor를 붙이는 방법은 사용하지 않는다. 재분할은 **의미를 보존하는 표현 변경**과 **별도 정책 결정이
필요한 의미 변경**을 분리하는 P2G-10 내부 transition이다. 결정 권한은 `8.6`을 따른다.

1. transition 직전 Current Working Checkpoint의 exact source Decision Ref Set을 고정한다. source는 모두 current
   member여야 하며, 범위 밖·과거·이미 재분할된 Decision을 다시 current로 가져오지 않는다.
2. source가 승인한 의미와 acceptance를 독립 단위로 나눈다. 각 result Decision은 새로운 stable Decision ID와
   `Predecessor Decision Revision Ref: N/A`를 가진 initial revision이다. 기존 source는 lineage predecessor가
   아니라 아래 embedded member의 provenance로 참조한다.
3. 각 독립 단위에 원래 approval/source span, 보존할 의미, acceptance와 영향을 받는 stable Element ID를
   대응시킨다. 의미·범위·예외·조건·승인된 acceptance는 더하거나 없애지 않는다. 동일한 승인 문구가 해당
   단위를 모호하지 않게 포함한다면 기존 exact approval을 재사용하며, 형식적 재분할만을 위한 재승인은
   요구하지 않는다. 그런 포함 관계를 증명할 수 없으면 사용자 결정 전에는 재분할을 publish하지 않는다.
4. result Working Revision에 아래 `Decision Repartition Member Set`을 포함한다. member는 parent 안의 stable
   `Repartition ID`로 식별하며 독립 Artifact ID/Ref나 자기 parent Ref를 payload에 넣지 않는다.
5. 영향을 받는 current Policy Element의 Decision 참조를 새 Decision으로 바꾼 successor revision을 준비한다.
   같은 stable Element ID·statement·acceptance를 보존하고 상태는 P2G-10의 `decided`로 둔다. 이전
   `code_reviewed`를 그대로 승계하지 않으며 후속 P2G-20의 source·binding 재평가에서 기존 근거 재사용을
   판정한다. 영향받지 않은 element와 Decision은 exact하게 보존한다.
6. source 의미 전체가 result에 빠짐없이 보존되고, result끼리 독립 결정 의미가 중복되지 않으며, 모든 affected
   current element가 정확히 하나의 result Decision을 참조하는지 검사한다. 기존 단일 Decision Ref 필드에
   임의로 복수 Ref를 넣지 않는다. 한 element의 의미를 나누지 않고 이 조건을 충족할 수 없다면 먼저 P2G-10에서
   element/decision 경계를 명시적으로 재검토하고 coverage가 해결될 때까지 재분할을 완료하지 않는다.
7. source Ref 제거, result Ref 전체 추가, affected element map 변경과 embedded member를 **같은 successor
   Working Checkpoint**에서 검증한 뒤 선택한다. source만 제거하거나 일부 result만 선택하는 부분 전이는
   금지한다. 필수 검증 실패 시 transition 이전 checkpoint를 유지한다.
8. 의미를 실제로 바꿀 독립 단위는 새 checkpoint를 Input으로 하는 일반 successor Decision으로 처리한다.
   사용자의 새 decision/acceptance를 그 lineage에만 적용한다. 재분할과 의미 변경은 같은 P2G-10 요청 안에서
   이어갈 수 있지만 하나의 불투명한 변경으로 합치지 않는다.

```text
Decision Repartition Member — 해당 Result Working Revision payload 내부
  Repartition ID
  Source Decision Revision Ref Set
  Result Initial Decision Revision Ref Set
  Preserved Meaning Coverage by source claim/acceptance locator
  Exact Existing Approval References and Preservation Evidence
  Affected Stable Element ID → Result Decision Revision Ref mapping
```

이 set은 해당 predecessor→result transition만 기록한다. 재분할이 없는 transition에서는 생략할 수 있으며
생략은 “이번 transition에 재분할 없음”만 뜻한다. 과거 재분할 이력은 predecessor Working Revision ancestry를
따라 확인한다. 결과 Decision의 canonical payload에는 source Decision, 자신을 담은 Working Revision 또는
아직 발행할 affected Policy Element Revision의 Ref를 역참조로 넣지 않는다. element→Decision 결속은 element
payload가, 재분할 provenance는 위 member가 소유한다. User Decision Log의 affected element 열은 이 관계를
보여주는 역방향 view이며 Decision identity에 대한 순환 payload가 아니다.

현재 set에서 선택된 lineage만 `active`이며 각 active lineage에는 current leaf가 하나다. 재분할 source의
lineage는 이 ancestry에서 current member가 없고 history로 남는다. 뒤 단계에서 그 source를 initial add로
되살리거나 result와 함께 선택하면 invalid다. 과거의 current 선택과 published Freeze Membership은 바꾸지
않는다. Freeze 이후의 재분할은 의미 보존이어도 Ref·binding 영향을 `8.4`로 평가한다. 정책 의미 변경이 없다는
이유로 기존 Freeze의 member·Decision·Binding Ref를 직접 고칠 수 없다.

예: `D0: E_A와 E_B는 재배치 때 초기화`를 `D_A0: E_A 초기화`, `D_B0: E_B 초기화`로 먼저 의미 보존
재분할한다. 새 checkpoint는 D0 대신 D_A0/D_B0를 선택한다. 사용자가 A 유지로 변경하면 그 뒤 D_A0→D_A1을
일반 successor로 처리하고 D_B0는 유지한다. D0→D_A1/D_B0의 분기 lineage를 만들거나 B의 의미를 누락하지 않는다.

#### Decision lineage와 deferred policy

- 일반 predecessor 교체는 주요 활동 8, 재분할은 위 “Decision 재분할”, 최종 일관성은 종료 조건 5가 소유한다.
  별도 supersession registry는 만들지 않는다. Result Working Revision의 current Decision Revision Ref Set과
  applicable embedded 재분할 member가 선택·대체 관계를 고정한다.
- 일부 claim만 달라지는 결정을 불완전한 successor로 덮지 않는다. 재분할 source의 lineage는 history로만
  남기고, 독립 result lineage별로 정확히 하나의 current leaf를 선택한다.
- in-scope Policy Element의 statement, Decision 또는 Observable Policy Acceptance에 영향을 주는 질문은
  deferred로 표시해 P2G-10 종료를 우회할 수 없다.
- deferred는 사용자가 명시적으로 out-of-scope로 승인했거나, 현재 in-scope 의미와 완료 조건에 영향을 주지
  않는 후속 주제일 때만 허용한다. owner/후속 요청 조건과 현재 scope에 영향이 없다는 근거를 남기며 current
  Open Policy Question Set에는 포함하지 않는다.

<!-- P2G-10 END -->

<!-- P2G-20 START -->
### P2G-20 — Code-Informed 구현 검토

#### 목적

하나의 고정된 policy basis를 기준으로 필요한 모든 요소의 구현 경계를 검토·재검증하고, 그 결과를 서로 다른
중간 Working Revision에 노출하지 않는 하나의 완전한 Review Batch로 publish한다.

#### 입력

- exact Input Working Revision Ref와 current Policy Element Revision map
- 검토·재검증할 exact current Policy Element Revision(`decided` initial review 또는 `code_reviewed`
  revalidation), Decision Revision과 그 revision이 고정한 exact Observable Policy Acceptance
- 대상 프로젝트 코드와 데이터
- 확인 가능한 runtime, persistence, transport와 consumer 자료
- repository/source basis와 기존 Code Review artifact(있는 경우)
- repair이면 applicable active Obligation Key Set, prerequisite resolution과 각 Policy/Artifact closure

Observable Policy Acceptance의 정책 의미 owner는 P2G-10이다. P2G-20은 이를 입력으로 고정하고 구현 가능성,
current code delta, validation seam과 evidence coverage를 검토하지만 문구나 의미를 변경하지 않는다. 코드 검토
결과 Acceptance 변경이 필요하면 아래 의사결정 지원 조사 경계 안에서 finding을 명확히 한 뒤
`material_policy`로 분류하고 현재 batch의 성공 경로를 abort해 P2G-10으로 route한다.

post-freeze non-material 재검증에 target successor Working Revision이 아직 없다면, Reader Contract에서
P2G-20 수행 권한을 확인한 뒤 per-element review 전에 `8.7`에 따라 predecessor Freeze를 가리키는 immutable successor
Working Revision을 만든다. 이 revision은 predecessor의 current element map을 그대로 상속하고 해당
change/closure basis를 고정한다.

#### 진입 조건

- exact Input Working Revision Ref와 current element map이 current head로 검증됨
- `is_p2g10_complete(Input Working Revision) = true`이며 incomplete Current Working Checkpoint가 아님
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-20보다 앞선 owner stage의 모든 applicable active obligation leaf가 effectively resolved됨
- P2G-20 소유 unresolved active obligation leaf는 이 Review Batch의 repair scope로 허용됨

#### Review Batch와 완전성

P2G-20 실행 하나는 하나의 Review Batch다.

```text
exact Input Working Revision 고정
→ candidate Review Scope/Source Requirement Plan과 Source Observation 구성
→ 모든 candidate review/revalidation 수행
→ Result Working Revision을 0개 또는 1개 결정
→ 모든 current binding을 exact Result basis에 연결
→ Review Scope/Source Requirement Plan Ref + Source Observation Ref Set + Code Review Artifact Ref Set
  + Review Evidence Ref Set + Result Working Revision Ref
  + Result-basis Code Review Binding Ref Set + Review Batch Ref를 complete result로 publish
```

- Batch 동안 Input Working Revision과 review scope를 바꾸지 않는다. policy basis가 바뀌면 현재 batch를
  abort하고 새 P2G 단계가 필요함을 보고한다.
- candidate Code Review Artifact와 Review Evidence는 proposed element payload와 함께 complete result publish 전까지 current가 아니다.
- 새 `code_reviewed` element revision이 필요하면 Result Working Revision은 정확히 하나다. 여러 요소를
  순차 검토하면서 중간 Working Revision을 만들지 않는다.
- current element map이 바뀌지 않는 순수 artifact 재검증이면 Result Working Revision은 Input Working
  Revision과 동일할 수 있다.
- Result Working Revision이 새로 생기면 Result map의 검토 대상이 아니었던 기존 `code_reviewed` revision도
  Result basis를 가리키는 current binding을 가져야 한다. 이를 위해 아래 일반 재검증 경로를 사용한다.
- material finding이 확정되거나 필수 evidence blocker로 batch가 중단되면 current Review Batch, Result Working Revision,
  정식 `code_reviewed` Policy Element Revision과 current binding set을 만들지 않는다. 조사 결과는
  `aborted/non-current` Review Scope/Source Requirement Plan, Source Observation, Candidate Review Envelope와
  immutable Review Evidence로만 보존하며 정식 Policy Element Revision Registry에는 넣지 않는다. material
  여부가 아직 불명확한 동안에는 아래 규칙에 따라 bounded read-only 조사를 할 수 있다. 사용자 의논이
  필요하다고 확인된 뒤에는 `8.6`의 주 작업 대기·기존 독립 작업 예외를 적용하며 해당 batch의 추가 조사나 진행을 하지 않는다.

#### 검토 경로

각 candidate element review는 다음 중 정확히 한 경로를 선택한다.

| 경로 | Entry condition | Candidate 결과 | Complete publish 결과 |
| --- | --- | --- | --- |
| `policy_revision_review` | Input map의 exact current revision이 `decided` | Code Review Artifact, Review Evidence와 proposed `code_reviewed` revision payload를 Candidate Review Envelope에 기록 | 성공한 complete batch가 formal `code_reviewed` revision을 materialize하고 Result map과 Result-basis binding이 이를 exact하게 선택·참조 |
| `code_reviewed_artifact_revalidation` | reviewed revision이 planned Result map의 해당 Element ID에 대한 exact current revision이고 intrinsic state가 `code_reviewed`; 해당 Element ID가 applicable active Policy Impact Closure 합집합 밖에 있음; policy meaning unchanged | 기존 revision과 Freeze Membership이 있으면 그대로 보존하고 fresh review/rebinding candidate 생성 | 성공한 complete batch가 exact Result Working Revision Ref, existing revision, source basis와 evidence를 잇는 current binding을 선택 |

일반 재검증 경로는 Freeze Membership 보유 여부와 무관하다. 따라서 아직 Freeze에 포함되지 않은 기존
`code_reviewed` revision도 새 Result Working Revision에 안전하게 재결속할 수 있다. 이미 더 새로운 revision이
planned Result map의 current인 과거 revision은 재검증 대상으로 선택할 수 없다.
경로를 결정할 수 없거나 입력 상태가 `open`이면 review를 시작하지 않고 필요한 P2G 단계를 보고한다.

#### 허용 / 금지

- 허용: read-only code/data/store/consumer 조사, current behavior 확인을 위한 비파괴 진단, Working Policy와
  Code Review artifact 갱신.
- 금지: production code/data/store 수정, 구현 착수, unrelated cleanup, 사용자를 대신한 material choice.
- 금지: Decision Revision 또는 Observable Policy Acceptance의 정책 의미 수정. 필요한 변경은
  `material_policy`로 반환.
- 금지: inaccessible/unknown source를 current나 compatible로 간주하거나 test/type 존재를 reachability 증거로
  대체.

#### 권장 코드 조사 방법론

아래 방법은 명령 순서나 완료 checklist가 아니라 상황별 권장 도구다. AI는 claim, repository 구조와 이용
가능한 evidence에 맞춰 방법, 순서와 깊이를 스스로 선택할 수 있다. P2G-20이 강제하는 것은 특정 검색 도구가
아니라 read-only 안전, evidence에 맞는 claim, policy owner와 current publish 경계다.

| 방법 | 주된 용도 | 해석 시 주의점 |
| --- | --- | --- |
| entrypoint와 forward call graph 추적 | input/command에서 실제 처리·mutation·output 경계로 이동 | 정적 graph는 reflection, dynamic dispatch, generated wiring과 configuration route를 누락할 수 있음 |
| reverse call graph와 reference search | 발견한 owner가 어떤 caller, command와 consumer에서 reachable한지 확인 | reference 존재만으로 실행 가능성·빈도·current route를 증명하지 않음 |
| authority와 write/read path 추적 | state의 authoritative writer, cache/view, duplicate representation과 read owner 구분 | type 이름이나 저장 위치보다 실제 write/read 선택과 lifecycle을 근거로 삼음 |
| data·state flow 추적 | validation, transform, lifetime, loss와 failure propagation 확인 | sample 한 건을 전체 상태 의미로 일반화하지 않음 |
| schema·serialization·persistence 추적 | DTO/order/version, save/load, migration, backward compatibility 경계 확인 | schema 정의만 보지 않고 실제 loader, writer, version selection과 consumer를 확인 |
| producer–transport–consumer 추적 | message/event/API의 생성, 전달, ordering과 reachable consumer 확인 | inaccessible consumer는 compatible/current로 추정하지 않고 limitation을 기록 |
| configuration·feature flag·build/release wiring 확인 | checkout code와 실제 선택되는 implementation/release 경로 구분 | default 선언을 released/deployed selection 증거로 대체하지 않음 |
| focused test·runtime trace·read-only probe | 정적 추론을 실제 execution, state transition과 observable output으로 보강 | 실행 수 0, skip, 환경 불일치와 관찰 불능은 pass가 아님 |
| history·diff·blame 조사 | 변경 의도, regression 지점과 source drift 후보 파악 | 과거 의도는 current behavior나 approved target의 owner가 아님 |

한 claim에 모든 방법을 적용할 필요는 없다. 서로 다른 방법의 evidence를 조합하면 static structure, runtime
reachability와 consumer effect를 구분하기 쉽다. 사용한 방법, 관찰한 exact source/result와 limitation을 Review
Evidence에 남긴다. 아래 `주요 활동`의 경계 질문에 답할 수 있으면 조사를 멈추며, 관련 없는 repository 영역을
완전 탐색하려 하지 않는다.

#### 조사 중 material policy candidate 처리

코드 조사 중 새 정책이 바로 확정되는 것은 아니다. 먼저 `material_policy candidate` finding으로 기록하고
`8.1`에 따라 technical decision 또는 evidence gap과 구분한다.

1. material 여부가 불명확하면 AI는 current behavior, authoritative owner와 consumer effect를 판정하는 데
   필요한 read-only 조사를 계속할 수 있다.
2. 사용자에게 결정 가능한 질문을 제시하기 위해 affected Policy Element/claim, 기존 정책으로 결정할 수 없는
   이유, 선택지, observable impact, compatibility·migration·data-loss·lifetime·rollout 영향, unknown과
   candidate Policy/Artifact closure를 조사할 수 있다.
3. 이 조사는 정책을 대신 결정하거나 구현안을 current target으로 선택하기 위한 것이 아니다. 확인된
   `material_policy`는 P2G-20이 canonical Open Policy Question, Decision Revision, 변경된 Observable Policy
   Acceptance 또는 successor `decided` revision으로 만들지 않는다.
4. 사용자 의논이 필요한 `material_policy`가 확인되면 `8.6`에 따라 주 작업과 정책에 의존하거나 의존성이
   불분명한 작업을 임시중단한다. 이미 확보한
   근거로 질문·영향·unknown을 정리하며, 추가 조사나 나머지 요소의 완료 처리, formal `code_reviewed` revision,
   Result Working Revision, current binding 또는 current Review Batch의 publication을 하지 않는다.
5. current actual, finding, source/evidence와 question candidate는 Candidate Review Envelope와 immutable Review
   Evidence로 보존한다. P2G-10이 같은 evidence를 참고할 수 있지만 canonical Open Policy Question과 정책
   결정은 P2G-10이 소유한다.
6. material finding으로 중단한 batch의 영향과 필요한 `8.4` owner 처리를 기록한다. 통상적 정책이라도
   P2G-20에서 결정·publication하지 않으며, 허용된 P2G-10으로 반환해 위임 결정과 필요한 후속 검토를 수행한다.
   사용자 의논이 필요하고 P2G-10이 승인 범위에 있으면 이미 확보한 근거로 그 owner의 질문·checkpoint만
   남긴다. 범위 밖이면 필요한 owner와 권한을 함께 질문한다. 사용자 의논 대기 중에는 Change 수리나 후속
   단계를 진행하지 않는다.

추가 조사 결과 승인된 정책 의미 안의 내부 realization 문제로 판정되면 `technical_decision`으로 기록하고
현재 batch를 계속할 수 있다. 판단에 필수인 evidence를 얻지 못하면 `evidence_gap`이며, 필수 claim을 침범하면
성공 batch를 publish하지 않는다.

#### 주요 활동

1. exact Input Working Revision의 current Policy Element claim/boundary에서 출발해 `8.2 Source와 Evidence`의
   P2G-20 Review Scope/Source Requirement Plan을 만든다. plan은 required runtime/data/store/transport/consumer,
   applicability와 validation requirement를 고정하되 Working Revision policy basis에는 포함하지 않는다. plan에
   따라 Workspace, Released, Deployed actual의 Source Observation과 exact reviewed source basis를
   고정한다.
2. 정책 주제 수준에서 input/command, authoritative runtime, live data, persistence, transport/server,
   reachable consumer와 test/acceptance entrypoint의 전체 경계를 한 번 식별한다.
3. 각 Policy Element의 observable behavior에서 시작해 그 요소의 claim을 설명하는 데 필요한 reachable
   code/data/store/consumer만 추적한다.
4. 타입, 함수, schema 또는 test가 존재한다는 사실만으로 runtime reachability나 current behavior를
   주장하지 않는다.
5. 다음을 서로 분리해 Code Review artifact에 기록한다.
   - Current Actual
   - Approved Target
   - Approved Observable Policy Acceptance
   - Implementation Delta
   - Proposed Implementation — `8.8`의 권장 구조·공유 계약·대안·코드 기반 가정과 상세 설계 인계
   - Unknown과 Evidence Gap
6. code/data/schema, persistence lifetime, DTO/transport ordering, required consumer, compatibility, migration,
   rollout과 validation seam 중 해당 요소에 영향을 주는 경계를 기록한다.
7. 코드에서 발견한 모든 사항에 `8.1 정책 발견 사항 분류와 결정 권한`을 적용한다. 분류를 이 section에
   다시 정의하지 않으며 다섯 분류 모두를 허용한다. 기존 Observable Policy Acceptance가 모호하거나 정책
   의미 변경 없이는 검증할 수 없다는 finding은 P2G-20이 acceptance를 보완하지 않고 `material_policy`로
   분류한다. material 여부나 선택지·영향을 설명할 evidence가 부족하면 위 의사결정 지원 조사만 수행하고,
   material이 확정된 뒤에는 batch 성공 경로를 계속하지 않는다.
8. finding, reviewed revision, candidate evidence와 affected claim을 양방향으로 연결한다.
9. 각 candidate 경로의 결과를 Candidate Review Envelope에 기록한다. 이 시점에는 formal `code_reviewed`
   revision, Result Working Revision이나 current binding을 만들지 않는다.
10. 모든 candidate가 성공하면 Produced `code_reviewed` Policy Element Revision Ref Set을 반영한 planned Result current element
    map을 계산하고 Result Working Revision이 Input과 같은지, 하나의 successor가 필요한지 결정한다.
11. Result Working Revision이 새로 생기면 Produced set 밖의 모든 Result Current `code_reviewed` revision에
    `code_reviewed_artifact_revalidation`을 적용해 exact Result-basis binding candidate를 만든다.
12. immutable Code Review Artifact Ref Set, Review Evidence Ref Set과 exact Reviewed Source Basis Ref, Result Current
    `code_reviewed` Policy Element Revision Ref Set 전체의 claim/boundary coverage, 모든 binding의 Result-basis
    일치를 검증한다.
13. 다음 candidate Review Batch payload를 구성한다. Working Revision payload는 binding을 포함하지 않으므로
    순환 참조가 없다. producer는 payload 밖의 Review Batch Ref를 발행한다.

```text
P2G-20 Review Batch Canonical Payload
  Input Working Revision Ref
  Candidate Review Envelope Ref Set
  Review Scope/Source Requirement Plan Ref
  Source Observation Ref Set
  Code Review Artifact Ref Set
  Review Evidence Ref Set
  Reviewed Source Basis Ref
  Produced code_reviewed Policy Element Revision Ref Set
  Result Working Revision Ref
  Result Current code_reviewed Policy Element Revision Ref Set
  Result-basis Code Review Binding Ref Set
  Batch Completeness Evidence
  Applicable P2G-20 Obligation Key Set
```

`Produced code_reviewed Policy Element Revision Ref Set`은 이번 batch가 새로 materialize한 revision만 뜻한다.
`Result Current code_reviewed Policy Element Revision Ref Set`은 Result Working Revision의 current element
map이 선택한 전체 `code_reviewed` revision 집합이며 Batch Completeness와 Result-basis Code Review Binding
Ref Set coverage의 기준이다.

14. applicable P2G-20 active leaf가 있으면 candidate Review Batch Ref와 exact prerequisite/result
    Composite Basis를 가리키는 candidate P2G-20 Resolution Record Ref Set을 준비한다. 없으면 명시적인 empty set을
    사용한다.
15. 다음 전체의 reference와 coverage를 하나의 complete P2G-20 result로 검증한다.

```text
P2G-20 Complete Result
  Review Scope/Source Requirement Plan Ref
  Source Observation Ref Set
  Code Review Artifact Ref Set
  Review Evidence Ref Set
  Reviewed Source Basis Ref
  Produced code_reviewed Policy Element Revision Ref Set
  Result Working Revision Ref
  Result-basis Code Review Binding Ref Set
  Review Batch Ref
  P2G-20 Resolution Record Ref Set
```

필수 구성과 종료 조건이 모두 검증된 뒤에만 formal `code_reviewed` revision, Result Working Revision,
binding, Batch와 Resolution Record를 current 결과로 publish한다. 그 전에는 모두 non-current candidate이며
Candidate Review Envelope만 보존할 수 있다. Working Policy Document의 current head view는 complete P2G-20
result가 있을 때만 Result Working Revision Ref를 선택한다. Result가 Input과 동일하면 같은 head를 유지한다.
검증 전 batch가 aborted되거나 complete result를 만들지 못하면 candidate Result를 선택하지 않고 exact Input
Working Revision head를 유지한다.

다음 질문에 모두 답할 수 있으면 필요한 코드 경계에 도달한 것이다.

- current behavior와 authoritative owner는 무엇인가?
- approved target과 current actual은 어디에서 다른가?
- 어떤 code/data/store/transport/consumer가 implementation delta에 속하는가?
- 구현 완료를 어떤 Observable Policy Acceptance와 validation seam으로 확인할 수 있는가?
- 확인하지 못한 경계가 어느 claim을 제한하는가?

이 답을 얻은 뒤에는 관련 없는 repository 영역을 계속 조사하지 않는다. 범위 밖 finding은 기록·route하되
현재 review scope를 자동 확장하지 않는다.

#### 산출물

성공한 batch는 다음을 산출한다.

- current Review Batch Ref
- exact Input Working Revision에 bound된 P2G-20 Review Scope/Source Requirement Plan Ref
- immutable Source Observation Ref Set
- immutable Code Review Artifact Ref Set, Review Evidence Ref Set과 exact Reviewed Source Basis Ref
- Produced `code_reviewed` Policy Element Revision Ref Set
- exact Result Working Revision Ref
- Result Working Revision Ref를 선택하는 Working Policy Document current head view
- Result Current `code_reviewed` Policy Element Revision Ref Set과 Result-basis Code Review Binding Ref Set
- 필요할 때만 owning review/binding 관계에서 파생하는 optional Policy Element-to-Code Review Matrix
- Code Review Finding Registry
- Technical Decision Log
- Evidence Gap Log
- repair이면 P2G-20 Resolution Record Ref

material policy 또는 필수 evidence blocker로 중단된 batch는 성공 산출물 대신 다음 non-current 조사 결과만
남긴다.

- `aborted/non-current` Review Scope/Source Requirement Plan과 Source Observation Ref Set
- Candidate Review Envelope Ref Set과 보존 가능한 immutable Review Evidence Ref Set
- material/evidence finding, affected claim·candidate closure와 limitation
- P2G-10에서 검토할 material-policy question candidate 목록 — canonical Open Policy Question artifact가 아님
- 수행하지 않은 current publication과 필요한 P2G 복귀를 명시한 stage result report
- exact Input Working Revision을 유지하는 Working Policy Document current head view

중단 batch는 current Review Batch Ref, Result Working Revision Ref, formal `code_reviewed` Policy Element
Revision Ref 또는 current Code Review Binding Ref를 산출하지 않는다.

#### 종료 조건

다음 공통 조건을 모두 만족해야 한다.

1. exact Input과 Result Working Revision Ref, reviewed revision, Decision, Review Scope/Source
   Requirement Plan과 source basis를 식별했다.
2. Current Actual, Approved Target, P2G-10에서 승인된 Observable Policy Acceptance, Implementation Delta와 Proposed
   Implementation을 구분했다. Proposed Implementation은 `8.8`의 적용 가능한 구조·소유·연결·실패·전환·검증
   의미와 근거를 제공하며, 후속 상세 설계로 남긴 선택의 범위와 owner를 식별했다.
3. 필요한 runtime/data/store/transport/consumer 경계를 검토했다.
4. 모든 claim에 evidence 또는 명시적인 `unknown`/`N/A`와 이유가 있다.
5. 모든 finding에 `8.1`의 primary classification과 disposition이 있다.
6. 새 `material_policy`가 없다.
7. technical decision은 정책 의미 보존 근거와 함께 기록됐다.
8. 필수 Evidence Gap이 review completeness claim을 침범하지 않는다.
9. out-of-scope finding이 현재 scope를 확장하지 않는다.
10. Produced set과 Result Current set이 구분돼 있고 Result Current set 전체가 Result-basis Code Review Binding Ref Set으로
    완전히 coverage된다.
11. Result Working Revision의 모든 in-scope current Policy Element Revision이 `code_reviewed`다.
12. 모든 binding이 동일한 exact Result Working Revision Ref, exact current element revision,
    published Review Scope/Source Requirement Plan, current source basis와 immutable review evidence를 가리킨다.
13. P2G-20 complete result의 모든 필수 reference, coverage와 상호 일치가 검증된 뒤에만 current로 선택됐다.
14. aborted candidate나 이전 Working Revision에만 묶인 binding을 current 결과로 사용하지 않는다.
15. 모든 applicable Change에 valid authoritative declaration이 존재한다.
16. repair이면 P2G-20이 소유한 모든 applicable active obligation leaf가 current Review Batch Ref/Code Review Binding Ref Set에 의해
    effectively resolved됐다.
17. 모든 current Source Observation이 exact Input Working Revision과 published Review Scope/Source Requirement
    Plan의 requirement member에 exact-bind되며, predecessor observation을 current evidence로 선택하지 않았다.
    반대 방향으로도 Plan의 모든 Requirement ID가 최소 하나의 current Source Observation으로 평가됐다. 각
    observation에는 유효한 disposition이 있고 `not_applicable` 또는 false로 입증된 conditional requirement만
    evidence가 있는 `N/A`로 평가됐다.
18. 모든 reviewed element의 Observable Policy Acceptance가 exact Input current revision의 값과 동일하며,
    P2G-20 결과가 이를 수정하거나 대체하지 않았다.
19. complete batch이면 Working Policy Document current head가 exact Result Working Revision Ref를 선택한다.
    publish 전 aborted되거나 complete result를 만들지 못하면 current head가 exact Input Working Revision
    Ref를 유지한다.

`code_reviewed`는 exact policy revision의 current boundary, delta, Observable Policy Acceptance와 evidence 검토가 완료됐다는
뜻이다. Final Policy Freeze, 구현 완료, compatibility pass, Phase 설계, Goal 작성 또는 구현 승인을 뜻하지
않는다.

#### 되돌림

- 새로운 material policy candidate가 발견되면 위 경계 안에서 materiality, 선택지, observable impact와
  closure를 설명하는 데 필요한 bounded read-only 조사를 수행할 수 있다. material이 확정되면 review를
  완료하지 않고 Policy Impact Closure와 Artifact Invalidation Closure를 기록하며 batch 성공 경로를 abort한다.
  Candidate Review Envelope와 evidence만 보존하고 formal `code_reviewed` revision, Result Working Revision,
  current binding 또는 current Review Batch를 만들지 않는다. Reader Contract의 범위에 P2G-10이 있으면 그
  owner에서 질문을 준비하고 결정 지점에서 멈춘다. 범위 밖이면 이동·정책 선택에 필요한 입력을 함께 요청한다.
- code evidence 때문에 Observable Policy Acceptance를 변경·보완해야 하거나 기존 acceptance가 ambiguous하면
  같은 `material_policy` 반환 규칙을 적용한다. validation seam은 P2G-20이 기록할 수 있지만 정책 acceptance를
  대신 결정하지 않는다.
- 정책 의미는 그대로지만 source/evidence가 바뀌면 `code_reviewed_artifact_revalidation` 또는 affected
  `decided` revision의 fresh candidate review를 사용한다. 기존 review를 current로 복사하지 않는다.
- 필수 evidence가 inaccessible/unknown이면 pass로 추정하지 않고 `8.6`의 blocker/중단 규칙으로 route한다.

#### Dirty workspace 적용

이 단계의 source 조사는 section 8.5를 따른다. dirty source의 관찰 결과를 evidence로 사용할 수는 있지만
baseline과 사용자 변경을 구분할 수 없는 overlap이 reviewed claim을 침범하면 successful Review Batch를
publish하지 않는다.

<!-- P2G-20 END -->

<!-- P2G-30 START -->
### P2G-30 — 최종 정책 폐쇄와 동결

#### 목적

정책 결정과 code-informed 검토가 서로 일치하는지 확인하고 최종 정책 revision을 동결한다.

#### 입력

- exact immutable Working Revision Ref와 current Policy Element Revision map
- 모든 Policy Element의 결정 기록
- current Review Batch Ref와 그 payload의 Produced/Result Current `code_reviewed` Policy Element
  Revision Ref Set
- Batch가 고정한 exact Result-basis Code Review Binding Ref Set, Reviewed Source Basis Ref와 Review Evidence Ref Set
- Open Policy Question, Technical Decision과 Evidence Gap 기록
- applicable active P2G-30 Obligation Key Set과 exact prerequisite Composite Basis

#### 진입 조건

- target Composite Basis가 exact하고 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-10과 P2G-20 owner의 모든 applicable active obligation leaf가 effectively resolved됨
- Review Batch Ref, Result Working Revision Ref와 Code Review Binding Ref Set이 current이고 모든 binding이 동일한 exact
  Result Working Revision Ref를 가리킴
- P2G-30 소유 unresolved active obligation leaf는 이 단계의 repair scope로 허용됨

#### 주요 활동

- 모든 in-scope 요소의 결정과 코드 검토 coverage 확인
- 남아 있는 미결 정책과 결정 충돌 확인
- Observable Policy Acceptance, compatibility, migration 및 알려진 unknown 확인
- 모든 current Policy Element Revision의 intrinsic state가 `code_reviewed`인지 검증
- target Working Revision의 in-scope Current Policy Element Revision Map을 Element ID로 정렬하고 다음 세
  candidate member selection이 정확히 같은지 검증한다. Element ID나 Revision Ref의 누락·중복, out-of-scope
  또는 historical revision 혼입을 허용하지 않는다.

```text
Candidate Freeze Member Policy Element Revision Ref Set
  == target Working Revision의 전체 in-scope current element revision value set
  == Review Batch의 Result Current code_reviewed Policy Element Revision Ref Set
```

- exact current `code_reviewed` Policy Element Revision별 required claim/boundary coverage를 계산
- 각 current Result-basis Code Review Binding이 exact target Working Revision Ref, exact current
  `code_reviewed` revision, Source Basis와 Review Evidence를 가리키는지 검증한다. P2G-30은 새 element
  revision이나 replacement binding을 만들어 논리적 간극을 메우지 않는다.
- Current Code Review Binding Ref Set이 모든 candidate member revision의 required claim과 boundary를 완전히
  coverage하는지 검증. 한 요소에 여러 source/consumer binding을 허용한다.
- 각 binding의 review evidence가 동일한 exact `code_reviewed` revision의 policy meaning, claim과 boundary를
  지지하며 current source basis에 대해 유효한지 확인
- ordered candidate member Policy Element Revision Ref Set에서 각 member의 Observable Policy Acceptance를
  읽어 `(Policy Element Revision Ref -> Acceptance)` projection을 계산하고, 별도 정책 입력이나 P2G-30
  판단으로 acceptance를 추가·변경하지 않는다.
- exact Reviewed Source Basis Ref를 고정
- applicable Change/Obligation Basis와 resolved prerequisite evidence를 순서화된 exact Ref로 고정
- 모든 검증이 성공하면 exact Working Revision Ref, current Review Batch Ref,
  exact `code_reviewed` Policy Element Revision Ref Set, Decision Revision Ref Set, Observable Policy Acceptance Set, Current Code Review Binding Ref Set,
  Reviewed Source Basis Ref, Binding Completeness Evidence, applicable Change/Obligation Basis와 알려진
  unknown으로 candidate Final Freeze payload를 구성하고 producer-issued Final Policy Freeze Ref를 준비한다.
- ordered member set에 대한 candidate `(Final Policy Freeze Ref, Policy Element Revision Ref)` Freeze Membership Set을
  계산한다. 하나의 revision이 기존 또는 후속 Freeze의 member여도 새 membership을 가질 수 있다.
- applicable P2G-30 active leaf가 있으면 candidate Final Policy Freeze Ref, exact prerequisite/result Composite
  Basis를 가리키는 candidate P2G-30 Resolution Record Ref Set을 준비한다. 해당 leaf가 없으면 이 set은 명시적인
  empty다.
- Final Freeze, Freeze Membership Set과 P2G-30 Resolution Record Ref Set의 참조·coverage를 모두 검증한 뒤 다음
  complete P2G-30 result로 publish한다.

```text
P2G-30 Complete Result
  Candidate Final Policy Freeze Ref
  Candidate Freeze Membership Set
  Candidate P2G-30 Resolution Record Ref Set
```

- 필수 구성, reference, coverage와 종료 조건이 모두 검증된 뒤에만 Final Freeze, Membership과 Resolution
  Record를 함께 current 결과로 선택한다. 그 전에는 셋 모두 non-current candidate이고 Freeze Membership은
  성립하지 않는다.
- complete P2G-30 result가 publish되면 Current Composite Basis의 current planning Freeze selection이 candidate
  Final Policy Freeze Ref를 선택한다. candidate validation이 실패하거나 complete result를 만들지 못하면 기존
  current planning Freeze selection을 바꾸지 않는다. 어느 경우에도 기존 Final Freeze bytes와 published
  Membership history는 수정하지 않는다.
- complete result publish 뒤 각 included Binding ID와 Final Policy Freeze ID를 연결하는 Binding Inclusion
  Index/View를 선택적으로 materialize할 수 있다. inclusion의 authoritative source는 Final Freeze의 Current
  Code Review Binding Ref Set이며 이 view의 부재는 Freeze currentness, binding completeness 또는 P2G-30
  종료를 막지 않는다. 이는 Final Freeze나 binding payload에 포함하지 않는다.

#### 산출물

- immutable Final Policy Freeze Ref
- immutable Freeze Membership set
- Current Code Review Binding Ref Set과 Binding Completeness Evidence
- Current Reviewed Source Basis Ref
- optional derived Binding Inclusion Index/View
- P2G-30 Resolution Record Ref Set
- member Policy Element Revision에서 계산한 Observable Policy Acceptance Set projection
- 새 Final Policy Freeze Ref를 선택한 current planning Freeze view
- 남은 unknown과 out-of-scope 목록

#### 종료 조건

- 모든 in-scope current Policy Element Revision의 intrinsic state가 `code_reviewed`
- Candidate Freeze Member Policy Element Revision Ref Set, target Working Revision의 전체 in-scope current
  element revision value set과 Review Batch의 Result Current `code_reviewed` Policy Element Revision Ref Set이
  Element ID/Revision Ref 기준으로 정확히 같음
- Open Policy Question이 없음
- 정책 결정과 코드 검토 결과가 충돌하지 않음
- Observable Policy Acceptance Set이 exact member Policy Element Revision에서 결정적으로 다시 계산되며
  모든 member의 acceptance와 일치함
- 모든 candidate member Policy Element Revision의 required claim과 boundary가 Current Code Review Binding Ref Set으로 완전히
  coverage됨
- 모든 included binding이 exact target Working Revision Ref, exact member revision과 current
  source basis를 가리킴
- complete P2G-30 result의 필수 구성과 reference가 검증돼 Final Policy Freeze, 모든 Freeze Membership과
  applicable P2G-30 Resolution Record Ref Set이 함께 current로 선택됐음. Freeze가 exact Binding Ref Set과
  Reviewed Source Basis Ref를 포함하고 binding 중 어느 것도 Final Freeze ID를 canonical payload에 포함하지
  않음
- published Final Policy Freeze의 lifecycle이 `current`
- complete result가 Current Composite Basis의 current planning Freeze로 exact Final Policy
  Freeze Ref를 선택함. optional Binding Inclusion Index/View의 존재 여부는 이 판정을 제한하지 않음
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-30이 소유한 모든 applicable active obligation leaf가 이 Freeze에 의해 effectively resolved됨

#### 되돌림

- Final Policy Freeze를 생성하기 전에 material question 또는 충돌이 발견되면 target Working Revision을
  수정하지 않고 Policy Impact Closure를 계산해 각 영향 요소에 successor `open` Policy Element Revision이
  필요함을 기록한다. Reader Contract가 P2G-10 복귀를 허용하면 `8.4`에 따라 successor Working Revision과
  필요한 element를 준비하고 `8.6`에 따라 위임 결정 또는 사용자 질문으로 처리한다. 허용하지 않으면 필요한
  이동·결정을 보고한다.
- Final Policy Freeze가 이미 생성된 뒤라면 같은 revision을 재개방하지 않고 `8.4`와 `8.7`의 successor
  revision 경로를 사용한다.
- 코드 evidence만 부족하면 정해진 evidence-gap 처리 결과에 따라 중단하거나 제한된 claim으로 진행한다.

#### Acceptance와 전환 disposition completeness

- Observable Policy Acceptance completeness는 모든 in-scope member Policy Element Revision에 비어 있지 않은
  acceptance가 있고, 성공·거부·실패·경계 중 해당 정책이 구분해야 하는 observable outcome을 모호하지 않게
  판정할 수 있음을 뜻한다. P2G-30은 acceptance를 새로 만들거나 보완하지 않고 member revision에서 projection한다.
- 각 acceptance와 required claim은 Current Code Review Binding Ref Set의 implementation boundary와 validation
  seam으로 coverage돼야 한다. 구현 완료 evidence 자체는 이 단계에서 요구하지 않는다.
- applicable한 compatibility, migration, persistence loss/lifetime, rollout, fallback 또는 rollback 의미에는
  P2G-10의 사용자 직접/위임 결정과 P2G-20의 reviewed boundary가 있어야 한다. 적용되지 않으면 근거 있는 `N/A`, 후속
  확인이 승인된 구현 계획의 일부이면 exact known unknown/owner를 기록한다.
- 위 의미나 current compatibility를 판정하는 데 필요한 evidence가 unknown이고 Freeze의 required claim을
  침범하면 동결하지 않는다. P2G-30이 unknown을 호환 가능 또는 안전으로 바꾸지 않는다.

<!-- P2G-30 END -->

<!-- P2G-40 START -->
### P2G-40 — Work Package, 의존성과 Phase 설계

#### 목적

동결된 정책을 안전하게 구현할 수 있는 Work Package와 실행 순서로 변환한다.

#### 입력

- exact current Final Policy Freeze Ref와 lifecycle disposition
- Freeze가 포함한 Code Review Binding Ref Set, Reviewed Source Basis Ref와 Completeness Evidence의 currentness evidence
- repair overlay가 있으면 exact successor Working Revision Ref와 Freeze compatibility evidence
- 필요할 때만 owning review/binding 관계에서 파생하는 optional Policy Element-to-Code Review Matrix
- implementation delta, ownership과 validation seam
- Final Freeze acceptance와 reviewed validation seam evidence 중 Phase 설계에서 cross-Phase invariant와
  integration acceptance 검증으로 배정할 수 있는 evidence
- applicable active P2G-40 Obligation Key Set과 exact prerequisite Composite Basis

#### 진입 조건

- Final Policy Freeze가 `current`이고 exact Final Policy Freeze Ref가 검증됨
- included Binding, Source Basis와 Completeness Evidence가 current이며 Freeze를 stale로 만드는 finding이 없음
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-40보다 앞선 owner stage의 모든 applicable active obligation leaf가 effectively resolved됨
- P2G-40 소유 unresolved active obligation leaf는 이 단계의 repair scope로 허용됨

#### 주요 활동

- 이번 실행의 exact current Final Policy Freeze Ref와 Exact Prerequisite Composite Basis를 하나로 고정한다.
  applicable Change가 없으면 사람용 Phase Plan의 top-level Repair Basis에서 관련 obligation/resolution/
  supersession set의 `explicit empty`를 한 번만 선언하고 이후 section은 이를 상속한다.
- 구현 작업을 Work Package로 구성한다. 파일 목록이나 넓은 product feature만으로 경계를 정하지 않고,
  authoritative owner, transaction, persistence lifetime, schema/data cutover, producer-consumer contract와
  독립 validation/completion boundary를 검토한다. migration, compatibility, rollout, rollback과 known unknown은
  applicable한 경우에만 기록한다.
- 각 Work Package에는 stable ID, Bound Final Policy Freeze에 상대적인 Covered Stable Element ID Selector Set,
  구현 경계, expected change, prerequisite, validation/completion condition과 applicable unknown을 기록한다.
  exact Policy Element Revision, Decision Revision과 Observable Policy Acceptance는 selector와 Bound Freeze에서
  파생하며 Phase Plan 본문에 같은 목록을 다시 전개하지 않는다. selector는 implementation, supporting과
  validation-only coverage를 구분한다.
- Final Freeze에서 파생되는 cross-Phase invariant와 integration acceptance마다 검증 seam을 다음 중 하나로
  disposition하고 Phase Plan의 Integration Validation Ownership Map에 기록한다.
  - `existing_read_only`: current repository/runtime에 재현 가능한 read-only seam이 있으며 locator, 실행 방법,
    fixture/input과 evidence contract를 식별함
  - `audit_local_read_only`: production source/data를 바꾸지 않는 audit-local probe로 충분하며, probe가 관찰할
    actual, 재현 방법과 temporary-output 경계를 식별함
  - `assigned_execution_node`: durable test/fixture/probe 또는 source change가 필요하여 이를 구현할 exact Final
    Execution Node와 owning Phase를 지정함
- integration acceptance를 검증할 seam이 없는데 구현할 Final Execution Node도 없으면 future GEX-30 작업으로
  미루지 않는다. implementation boundary나 Phase assignment가 불완전한 것으로 판정해 P2G-40을 완료하지 않는다.
- Final Freeze의 각 Policy Element Revision에 stable Element ID selector를 통해 정확히 하나의 primary owning
  Phase를 지정한다. 같은 요소를 돕는 supporting Phase가 있으면 그 relation과 realization·validation 경계를
  명시한다. 별도 1:1 Policy Element-to-Phase 표는 필수가 아니며 ordered Phase Set의 primary selector relation이
  authority다.
- Work Package dependency graph를 작성한다. edge type은 필요에 따라 확장할 수 있는 권장 vocabulary이며
  `policy`, `authority`, `transaction`, `schema_data`, `persistence_lifetime`, `producer_consumer`,
  `migration_compatibility`, `validation`, `release_rollout`, `legacy_removal`, `external_evidence` 등을 예로
  사용할 수 있다. 파일 충돌만으로 runtime dependency를 단정하지 않는다.
- 각 dependency edge에는 prerequisite Work Package, dependent Work Package, 적용한 type, 근거와 그 edge를
  무시했을 때 깨지는 선행 전제, 유효 상태 또는 완료 claim을 기록한다.
- `external_evidence` edge는 Final Freeze가 후행 evidence 확인을 명시적으로 허용하고, evidence 확보 자체가
  승인된 구현 계획의 일부이며, 그 지연이 Phase Plan의 성립이나 Freeze currentness를 훼손하지 않을 때만
  사용한다. 구현 경계나 Freeze 유효성을 먼저 확정해야 하는 evidence gap이면 Phase dependency로 숨기지
  않고 `boundary_or_evidence_gap`으로 판정해 P2G-40 성공 경로를 중단한다. 허용 범위의 P2G-20에서 근거를
  확보·재검증할 수 있으면 계속하고, 접근 불가·새 결정 등 실제 blocker이면 사용자에게 보고한다.
- dependency graph에서 cycle을 포함하는 SCC, 즉 둘 이상의 Work Package를 포함하거나 실질적인 self-loop가
  있는 SCC만 disposition한다. 정상적인 단일 비순환 node에는 SCC disposition을 요구하지 않는다. cyclic SCC가
  하나도 없으면 사람용 Phase Plan에는 empty table을 만들지 않고 Validation Receipt의 count/result로만
  증명한다.
- cyclic SCC는 다음 중 하나로 disposition한다.
  - `coimplement_atomic`: 공유 transaction/cutover/lifetime/producer-consumer boundary 때문에 분리할 수 없어
    하나의 Atomic Group으로 결합한다.
  - `break_by_stable_seam`: 실제 stable contract와 독립적으로 유효한 중간 상태가 있을 때 Work Package 경계를
    재구성하거나 edge를 재분류한다. 기존 edge를 근거 없이 삭제하지 않으며 변경 근거를 남기고 SCC를 다시
    계산해 cycle 해소를 검증한다.
  - `material_policy`: cycle을 끊으려면 사용자 정책 결정 또는 Observable Policy Acceptance 변경이 필요하다.
    P2G-10 owner의 실행 권한과 `8.6`을 확인하여 위임 결정 또는 사용자 질문으로 처리한다.
  - `boundary_or_evidence_gap`: authority, consumer, migration 또는 구현 경계 evidence가 부족하다. P2G-40 성공
    경로는 중단하고 허용된 P2G-20에서 조사·재검증한다. 해소할 수 없는 필수 근거 문제이면 사용자에게 보고한다.
- Work Package를 나눴을 때 competing/missing authority, transaction 단절, reader/writer schema 불일치,
  unreadable persistence, producer-consumer incompatibility, migration loss, 독립 validation 불능 또는
  applicable한 rollback 수행 불능이 생기면 함께 구현해야 하는 Atomic Group 후보로 본다. Atomic Group은
  편의를 위한 큰 묶음이 아니라 유효한 중간 상태를 만들 수 없는 최소 경계다.
- 모든 Work Package를 standalone execution node 또는 정확히 하나의 Atomic Group member로 disposition한다.
  Atomic Group은 서로 겹치거나 중첩될 수 없으며, candidate group이 Work Package를 공유하면 하나의 Atomic
  Group으로 병합한다. Final Execution Node Set은 standalone Work Package Set과 Atomic Group Set의 합이다.
- cyclic SCC disposition과 Atomic Group condensation 뒤의 acyclic dependency graph를 기준으로 Final
  Execution Node를 Phase에 배정하고 순서화한다. 모든 Final Execution Node는 정확히 하나의 Phase에 속하며,
  Atomic Group member를 개별 execution node로 다시 배정하거나 서로 다른 Phase로 분리하지 않는다. graph
  도구, SCC 알고리즘과 탐색 순서는 비규범적이며 AI가 규모와 증거에 맞게 선택할 수 있다.
- 각 Phase 완료 직후를 대상으로 Intermediate-State Audit을 수행한다. applicable한 범위에서 단일 authority,
  runtime/state transition, reader/writer schema, save/load와 persistence lifetime, producer/transport/consumer,
  승인된 compatibility·migration·rollout·rollback, validation seam과 승인되지 않은 fallback 또는 dual path의
  부재를 확인한다. fallback 또는 dual path가 존재하면 Final Freeze의 승인된 정책과 유지·제거 조건에
  일치하는지도 확인한다. 다음 Phase가 수행되지 않으면 현재 Phase의 유효성이나 완료 claim이 깨지는 경우에는
  두 Phase의 경계 또는 Atomic Group을 다시 설계한다.
- Phase 병렬화는 dependency path 부재만으로 승인하지 않는다. 병렬화를 승인할 때만
  authority/transaction/schema/persistence와 producer-consumer cutover의 독립성, file collision 부재 또는
  명시적 merge plan, 독립 validation과 병렬 완료 후 중간 상태의 유효성을 상세 Positive Evidence로 기록한다.
  병렬화를 승인하지 않으면 `Parallelization: none approved` 한 번으로 충분하다. 같은 topological layer라는
  사실은 필요조건일 수 있지만 충분조건은 아니다.
- foundation 성격의 Phase는 stable contract, observable completion, 독립 validation/currentness, 구체적인
  downstream consumer와 Final Freeze의 요구가 있을 때만 둔다. 미래 편의를 위한 추상화만으로 Phase를 만들지
  않는다.
- `8.8`의 공유 계약은 기존 Work Package의 implementation boundary/prerequisite와 Intermediate-State Audit에
  연결한다. 각 계약의 의미 owner, 생산자·소비자, 구현할 node, 검증할 node와 실제 사용 가능 시점을 구분한다.
  권장 설계 초안의 존재를 후속 Phase가 호출할 구현 API의 존재로 간주하지 않는다. 미해결 상세 선택을 허용할
  때도 그 선택이 현재 Phase 경계·의존성·중간 상태를 바꾸지 않는다는 근거가 필요하다.
- 모든 disposition과 중간 상태 감사를 통과한 최종 Phase Plan의 독립 완료 Phase 수를 센다. Phase가 하나면
  `single_phase`, 둘 이상이면 순차·병렬 여부와 관계없이 `multi_phase` 구조 사실을 P2G-50에 전달한다. 한
  Phase 안에는 여러 독립 Work Package가 포함될 수 있다. 실제 Single Goal 또는 Master/Phase Goal의 형식
  결정과 작성은 P2G-50이 소유한다.
- Phase Plan이 exact current Final Policy Freeze Ref와 prerequisite Composite Basis를 고정하게 한다.
- Phase 수리로 downstream closure가 바뀌면 기존 obligation을 수정하지 않고 successor obligation set과
  complete Obligation Supersession Record를 생성한다.
- candidate Phase Plan Ref를 발행하고 사람용 설계 내용이 안정된 뒤 repository 표준 생성기로 exact Bound
  Freeze에 대한 Phase Plan Validation Receipt JSON을 산출한다.

  ```bash
  node core_enhanced/docs/tools/validate_p2g40_receipt.mjs --write-receipt \
    --plan <phase-plan.md> \
    --freeze <final-policy-freeze.md> \
    --receipt <phase-plan-validation-receipt.json>
  ```

  생성기는 selector exact resolution, primary owner coverage, Work Package/Atomic Group partition,
  dependency/SCC/condensation graph, Final Execution Node-to-Phase cardinality, required audit/validation-owner row,
  Goal Structure Fact와 prerequisite/repair basis를 Plan과 Freeze에서 결정적으로 파생한다. target Receipt가
  없으면 생성하고, 이미 존재하면서 계산 결과와 의미상 동일하면 bytes를 바꾸지 않고 `unchanged`로
  통과한다. 서로 다른 immutable Receipt가 이미 있으면 덮어쓰지 않고 실패하므로 successor Receipt Ref와
  path를 발행해야 한다. 생성된 JSON을 수동 편집하지 않는다.
- 생성 직후 같은 exact 세 입력으로 기본 검증 모드를 실행한다.

  ```bash
  node core_enhanced/docs/tools/validate_p2g40_receipt.mjs \
    --plan <phase-plan.md> \
    --freeze <final-policy-freeze.md> \
    --receipt <phase-plan-validation-receipt.json>
  ```

  process exit code가 `0`이고 출력의 `result`가 `passed`, `failures`가 empty일 때만 passed Receipt로 사용한다.
  생성·검증이 실패하면 오류의 authoritative owner부터 구분한다.
  - 현재 P2G-40 candidate Plan의 selector·배정·표현 오류이면 그 candidate를 교정한다. 이미 Ref가 발행돼
    immutable payload가 고정된 Plan이면 새 Ref를 발행하고, published Plan의 교정이면 `8.4`의 successor/repair
    절차를 따른다. 변경된 exact Plan에 대해 새 Receipt를 생성·검증한다.
  - Freeze의 Decision/Acceptance 의미 오류이면 P2G-10, included code/source/binding 근거 오류이면 가장 이른
    P2G-20 등 owner 필요를 보고한다. `8.4`의 invalidation/repair 경계를 적용하고 **입력 Freeze의 bytes,
    member 또는 source fact를 P2G-40에서 수정하지 않는다.** 허용된 non-material 수리는 해당 owner에서
    계속하고, 새 material 선택·범위 확대는 결정 지점에서 멈춘다. 단일 단계 요청의 범위는 넘지 않는다.
  - 도구 결함·미지원 포맷·환경 문제 가능성이면 명령·입력 Ref·진단과 한계를 분리 기록한다. Receipt나 Freeze를
    고쳐 도구의 오판에 맞추거나 검증을 우회하지 않는다. 필요한 도구 교정은 별도 승인 범위다.
  receipt는 Work Package 경계, dependency 이유, intermediate-state validity, 설계 인계 또는 validation seam의
  의미적 타당성을 기계적으로 대신 판단하지 않는다.
- Phase Plan Ref, passed Phase Plan Validation Receipt Ref, P2G-40 Resolution Record Ref Set과 해당 Obligation
  Supersession Record Ref Set의 필수 구성과 상호 참조를 검증한 뒤 complete P2G-40 result로 publish하고 current
  Phase Plan을 선택한다.

#### 축약 표시 규칙

- `single_phase`이면 ordered Phase row와 그 직후 Intermediate-State Audit을 하나의 compact phase row로 합칠 수
  있다. Work Package, dependency, Atomic Group과 validation ownership 검사는 그대로 수행한다.
- cyclic SCC가 없으면 Cyclic SCC Disposition 표를, 병렬화 승인이 없으면 상세 Parallelization Evidence 표를
  만들지 않는다. 각각 Validation Receipt의 `zero cycles`와 Phase Plan의 `none approved`가 그 결과를 소유한다.
- Freeze의 전체 Policy Element Revision/Decision Revision/Acceptance 목록, Work Package별 exact expansion과
  별도 1:1 Policy Element-to-Phase Matrix는 기본 Phase Plan에 반복하지 않는다. 필요하면 selector resolution에서
  optional traceability view를 생성한다.
- `explicit empty` repair field와 동일한 counts/coverage 결과는 사람용 문서의 한 summary에만 표시한다. typed
  machine field의 구분과 stage gate 검증은 Validation Receipt가 유지한다.

#### 산출물

- current ordered Phase Plan Ref
- repository 표준 생성기가 산출하고 기본 검증 모드가 통과한 Phase Plan Validation Receipt JSON과 그 Ref
- Phase Plan canonical payload의 Exact Prerequisite Composite Basis, Bound Final Policy Freeze Ref와 Applicable
  P2G-40 Obligation Key Set
- Phase Plan canonical payload의 Work Package Set, Freeze-scoped Policy Element Selector Coverage, typed Dependency
  Graph, applicable Cyclic SCC Disposition Set, Atomic Group Set, ordered Phase Set, Intermediate-State Audit,
  applicable Parallelization Evidence, Integration Validation Ownership Map, `single_phase|multi_phase` Goal
  Structure Fact와 applicable Known Unknown/Limitations
- 필요할 때만 selector resolution에서 생성한 optional Policy Element-to-Phase traceability view
- P2G-40 Resolution Record Ref와 해당 시 Obligation Supersession Record Ref

#### 종료 조건

- Phase Plan Validation Receipt가 exact Phase Plan Ref와 Bound Final Policy Freeze Ref를 가리키고 `passed`임
- repository 표준 검증기의 process exit code가 `0`, 출력 `result`가 `passed`, `failures`가 empty임
- 모든 Freeze-scoped selector가 Bound Freeze의 exact member revision 하나로 해석되고, Final Policy Freeze의
  모든 Policy Element member revision에 missing/duplicate/extra 없이 primary owning Phase가 있음
- 모든 Work Package가 standalone execution node이거나 정확히 하나의 Atomic Group member이고, Atomic Group이
  서로 겹치거나 중첩되지 않음
- standalone Work Package와 Atomic Group으로 구성된 모든 Final Execution Node가 정확히 하나의 Phase에
  배정됐으며 Atomic Group member가 서로 다른 Phase로 분리되지 않음
- 모든 cyclic SCC에 근거 있는 disposition이 있고, `break_by_stable_seam` 적용 후 다시 계산한 graph 또는
  Atomic Group condensation graph가 acyclic임
- 각 Phase 완료 직후의 applicable 중간 상태와 완료 claim이 유효함
- `8.8`의 cross-Work-Package 공유 계약과 상세 설계 인계가 existing boundary/prerequisite/validation owner에
  연결돼 있고, 아직 구현되지 않은 선행 계약을 사용 가능하다고 가정하지 않았음
- Phase 순서, 완료 조건과 supporting Phase 관계가 명확함
- 모든 applicable cross-Phase invariant와 integration acceptance에 `existing_read_only | audit_local_read_only |
  assigned_execution_node` validation disposition이 있고, durable seam에는 exact owning Final Execution Node와
  Phase가 있음
- 병렬 가능하다고 표시한 모든 Phase 조합에 positive parallelization evidence가 있음
- final ordered Phase Set의 수에 따라 `single_phase|multi_phase` Goal Structure Fact가 결정됐고 P2G-50의 Goal
  작성 권한을 침범하지 않음
- published Phase Plan이 exact current Final Policy Freeze Ref를 가리키며 lifecycle이 `current`
- complete P2G-40 result의 필수 구성, reference와 coverage가 Validation Receipt와 함께 검증된 뒤 current Phase
  Plan으로 선택됨
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-40이 소유한 모든 applicable active obligation leaf가 current Phase Plan으로 effectively resolved됨

#### 되돌림

- dependency 검토에서 새로운 material policy가 발견되면 `8.4`에 따라 Post-Freeze Change, 정확한 Policy
  Impact Closure, Artifact Invalidation Closure, complete initial obligation member set과 Invalidated Artifact
  Disposition Set을 하나의 완전한 Change Obligation Declaration으로 publish한다. declaration이 current로
  선택되면 모든 영향 artifact와 기존 Freeze의 `stale` disposition을 파생하되 bytes와 Membership은 수정하지
  않는다. P2G-10 복귀 권한을 Reader Contract로 확인하고, 범위 안이면 `8.6`에 따라 위임 결정 또는
  사용자 질문으로 처리한다. 범위 밖이면 필요한 이동·결정을 함께 요청한다. declaration의 필수 구성을
  완성하지 못하면 수리하지 않는다.
- 정책 의미는 그대로지만 구현 경계가 달라졌다면 해당 non-material Change의 Policy Impact Closure는
  empty로 기록하고 `8.4`의 complete declaration을 publish한다. current declaration의 Invalidated
  Artifact Disposition Set에서 영향 artifact의 `stale`을 파생한 뒤 가장 이른 owner stage를 판정하고, 그
  단계에서 predecessor Freeze를 가리키는 successor Working Revision을 준비한다. `bounded_pipeline`의 범위
  안이면 필요한 owner에서 수리·재검증을 계속하고, 범위 밖이거나 추가 권한이 필요하면 멈춘다.

#### 조사·설계 방법의 재량

call graph, reference search, typed dependency graph, SCC detection, condensation과 topological layering은 권장
기법이다. 특정 graph 알고리즘, command, 도구, 파일 순서나 edge type 집합을 규범적으로 강제하지 않는다.
강한 제한은 근거 없는 dependency 삭제, material policy의 대리 결정, evidence gap의 Phase dependency 위장,
invalid intermediate state의 승인과 요청 범위를 벗어나는 실행에 적용한다. 범위 안의 검증된 단계 전이는 허용한다.

<!-- P2G-40 END -->

<!-- P2G-50 START -->
### P2G-50 — Single 또는 Master/Phase Goal 작성

#### 목적

current Phase Plan의 구조를 재판단하지 않고 각 Phase를 정확히 하나의 Single/Phase Goal member로 변환한다.
`multi_phase`이면 구현 작업을 소유하지 않는 Master coordination member를 추가하고, 모든 member를 포함한
완전한 Goal Set을 한 번에 publish한다. activation, handoff authorization과 구현은 수행하지 않는다.

#### 입력

- exact current Final Policy Freeze Ref
- exact current ordered Phase Plan Ref
- exact passed Phase Plan Validation Receipt Ref
- ordered Phase Set의 Freeze-scoped primary/supporting Policy Element Selector relation
- Phase별 assigned Final Execution Node와 구현 경계. exact Policy Element Revision, Decision Revision과 Observable
  Policy Acceptance coverage는 Bound Freeze와 validated selector relation에서 파생함
- Phase Plan canonical payload의 Dependency Graph, Atomic Group Set, ordered Phase Set, Intermediate-State Audit,
  applicable Parallelization Evidence, Integration Validation Ownership Map과 `single_phase|multi_phase` Goal
  Structure Fact
- applicable한 user-approved Validation Baseline Reuse Directive. 없으면 `N/A`; 있으면 exact directive ref와
  승인 범위, initial/final full validation owner 및 evidence invalidation 경계를 식별함
- correction이면 exact predecessor Goal Set Ref, predecessor lifecycle/disposition과 exact correction
  scope/evidence, 아니면 `N/A`
- applicable active P2G-50 Obligation Key Set과 exact prerequisite Composite Basis

#### 진입 조건

- Final Policy Freeze와 ordered Phase Plan이 모두 `current`
- Phase Plan Validation Receipt가 exact current Phase Plan Ref와 Final Policy Freeze Ref를 가리키고 `passed`임
- Phase Plan Ref가 exact Final Policy Freeze Ref를 가리키고, 그 canonical payload에 Dependency Graph와 Atomic
  Group Set, Integration Validation Ownership Map 및 final ordered Phase Set에서 계산한 Goal Structure Fact가
  포함됨
- 모든 Final Execution Node가 정확히 하나의 Phase에 배정되고 각 Phase의 policy, decision과 acceptance
  coverage가 완전함
- Integration Validation Ownership Map의 모든 applicable acceptance가 complete disposition을 가지며,
  `assigned_execution_node` member는 exact Final Execution Node와 owning Phase를 가짐
- correction predecessor는 published exact Goal Set Ref이고 current Final Policy Freeze 및 Phase Plan과
  양립함. predecessor가 `current`이면 current Goal Set selection과 exact match해야 한다. predecessor가
  `stale`이면 authoritative Change Obligation Declaration의 Invalidated Artifact Disposition Set과 active
  P2G-50 obligation이 지정한 exact invalidated Goal Set이어야 하며, 이 조건을 만족한 stale 자체는 repair
  entry를 막지 않음
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-50보다 앞선 owner stage의 모든 applicable active obligation leaf가 effectively resolved됨
- P2G-50 소유 unresolved active obligation leaf는 이 단계의 repair scope로 허용됨

#### 경로 선택

- Phase Plan의 Goal Structure Fact가 `single_phase`이고 final ordered Phase Set에 Phase가 정확히 하나이면
  Single Goal을 작성한다. 해당 Phase 안에는 여러 Work Package 또는 Atomic Group이 포함될 수 있다.
- Goal Structure Fact가 `multi_phase`이고 독립 완료 가능한 Phase가 둘 이상이면 순차·병렬 여부와 관계없이
  Master Goal과 각 Phase Goal을 작성한다.
- Goal Structure Fact와 final ordered Phase Set의 수가 다르면 임의로 Goal 구조를 선택하지 않고 P2G-40
  Phase Plan 교정으로 반환한다. 허용된 연속 진행 범위이면 P2G-40에서 교정·재검증하고, 아니면 필요한 요청을 보고한다.

#### Goal Set 공통 내용

- Predecessor Goal Set Ref 또는 `N/A`
- Exact Prerequisite Composite Basis, exact current Phase Plan Ref와 Final Policy Freeze Ref
- Phase Plan에서 가져온 `single_phase|multi_phase` Goal Structure Fact
- stable Goal ID로 정렬한 complete Single/Master/Phase Goal Member Set
- 모든 Phase ID를 정확히 하나의 Single/Phase Goal ID에 연결하는 Phase-to-Goal Member Mapping. Master Goal
  ID는 이 mapping의 target이 아니다.
- Applicable P2G-50 Obligation Key Set
- Goal Quality Contract Version: `2`
- `Authoring Status: Authored`
- `Activation Status: Not Activated`
- `Implementation Authorized: No`

applicable한 user-approved validation reuse directive가 있으면 Goal Set member보다 앞선 공통 payload에
`Shared Validation Baseline Reuse Contract`를 둔다.

- 첫 executable Single/Phase Goal은 production mutation 전에 전체 validation command set을 한 번 실행해
  initial full baseline을 고정한다. Master Goal은 실행 작업이나 baseline 생성을 소유하지 않는다.
- baseline evidence key는 최소한 command/test selector, fixture/input, relevant source/data/test/config,
  toolchain과 lock/dependency identity, 실행 결과와 owning Goal을 포함한다.
- Phase 시작 자체는 baseline invalidation이나 full validation 재실행 사유가 아니다. exact relevant input
  identity와 evidence contract가 유지되면 이전 성공 evidence를 재사용한다.
- production source/data/config가 바뀌면 test source가 그대로여도 영향받는 validation seam baseline은
  invalid다. 변경 영향 map에 포함된 focused validation을 실행해 current target에 다시 bind한다.
- test/fixture/probe가 바뀌면 그 validation seam baseline을 invalid 처리하고 변경된 검증을 current target에
  non-zero로 실행해 새 baseline을 고정한다. shared harness, schema, persistence, transport 또는 dependency
  변경은 영향 map이 가리키는 baseline set 전체를 invalid 처리한다.
- invalidation되지 않은 seam은 다시 실행하지 않는다. “test 파일이 바뀌지 않음”만으로 source 변경 뒤의
  evidence를 재사용하거나 모든 validation을 생략하지 않는다.
- 마지막 executable Phase Goal은 모든 구현과 focused rebaseline이 끝난 current integrated target에서 전체
  validation command set을 한 번 실행해 final full baseline을 고정한다. 실패, 실행 뒤 relevant input 변경 또는
  incomplete/zero execution은 성공 1회로 세지 않으며 필요한 재실행을 금지하지 않는다.
- normal linear execution의 full validation은 initial 1회와 final 1회이며, 중간 Phase 시작마다 반복하지 않는다.
  Phase-local completion은 영향받은 focused seam과 assigned integration validation evidence로 판정한다.

Goal Set은 P2G-60의 durable handoff와 authorization boundary를 소유하지 않는다. handoff까지 요청된
`bounded_pipeline`이면 P2G-50 exit 후 P2G-60의 entry를 확인해 진행한다. Goal 작성까지만 요청됐으면 종료한다.
어느 경우에도 Goal Set 자체가 handoff 수행 권한, activation 또는 구현 권한을 부여하지 않는다.

#### Single/Phase Goal member 내용

- Goal ID, `single|phase` kind와 담당 Phase ID
- Phase에 배정된 standalone Work Package/Atomic Group으로 이루어진 Final Execution Node Set
- Covered Policy Element Revision Ref Set, Covered Decision Revision Ref Set과 해당 Policy Element Revision이
  소유한 derived Observable Policy Acceptance coverage
- 목표와 contract-visible result
- 해결할 user-visible problem과 각각 독립적으로 판정 가능한 user-visible completion condition set
- in-scope와 out-of-scope
- reviewed current behavior와 implementation delta
- Goal-local source of truth와 authoritative owning layer
- Goal-local invariant와 그 exact Final Freeze/Phase Plan source
- 구현할 code/data/schema/store/transport/consumer boundary
- consumer verification boundary. workspace에서 확인한 consumer, canonical 문서에만 명시된 external consumer,
  존재는 알려졌지만 접근할 수 없는 consumer와 존재·배포 상태가 unknown인 consumer를 구분하고, required
  consumer를 확인하지 못했다면 완료 가능한 것처럼 작성하지 않음
- 선행 Goal dependency와 goal-local execution prerequisite
- validation 계획과 completion matrix
- section 아래의 `Validation Dimension Matrix`. 각 dimension은 `required | N/A` 중 하나이며 `N/A`는 Goal
  actual과 승인 policy에 근거한 구체적 이유를 가짐
- structural completion, legacy/fallback disposition. 단일 authority, superseded path 제거, 승인되지 않은
  dual path/fallback 부재, current live path에 필요하지 않은 미래 전용 abstraction 부재와 test/fixture 전용
  production behavior 부재를 completion condition에 연결함
- 변경된 책임과 가장 가까운 canonical documentation obligation 또는 근거 있는 `N/A`
- inherited Shared Validation Baseline Reuse Contract, 이 Goal이 바꾸는 input-to-seam impact map,
  reusable/invalidated baseline 판정과 focused rerun/rebaseline plan
- 이 Phase에 배정된 durable integration validation seam이 있으면 acceptance, seam과 implementation/completion
  coverage
- applicable한 migration, compatibility, rollout과 rollback 요구사항
- goal-local known unknown, blocker와 중단 조건
- 위 existing member 항목 안에 `8.8`의 코드 기반 상세 설계 인계 내용을 배치함. 새 독립 Design artifact나
  quality-contract top-level field를 강제하지 않으며, 적용되는 내용은 existing boundary/prerequisite/
  invariant/validation/unknown 항목에서 누락 없이 확인할 수 있어야 함
- applicable한 risk와 reviewer requirement 및 그 근거. reviewer를 지정·승인하거나 Goal Gate를 실행하지
  않는다.

##### Validation Dimension Matrix

Goal Quality Contract Version `2`의 모든 Single/Phase Goal member는 다음 dimension을 정확히 한 번
disposition한다.

| Dimension | 판정 대상 |
| --- | --- |
| `success` | 실제 사용자-visible 정상 완료 경로 |
| `invalid_input` | invalid schema, command, state 또는 data의 명시적 rejection과 무변경 보장 |
| `failure_rollback` | 부분 실패, 원자성, rollback과 중간 상태 비노출 |
| `retry_idempotency` | retry, duplicate, dedupe, exactly-once 또는 재호출 의미 |
| `ordering_concurrency` | message/event ordering, 같은 boundary, 경쟁 transition 또는 동시성 |
| `boundary_empty_max` | empty, missing, 최소·최대, limit과 overflow/underflow 경계 |
| `disconnect_restart_persistence` | disconnect, reconnect, process restart, save/load와 persistence lifetime |
| `live_data` | production loader가 읽는 실제 data/RON과 invalid-data startup rejection |
| `producer_consumer` | mock끼리의 일치가 아닌 실제 producer shape와 접근 가능한 consumer의 왕복 |
| `legacy_fallback_absence` | 제거 대상 legacy, duplicate authority, fallback, dual-read/write와 schema alias 부재 |

`required` row는 validation seam/evidence contract와 completion requirement를 기록한다. 반드시 모든 row를
별도 test로 만들라는 뜻은 아니며, current source inspection, strict loader audit, focused/integration test 또는
durable read-only seam 중 해당 claim을 직접 증명하는 방법을 선택할 수 있다. `N/A` row는 단순히 범위 밖이라는
표시가 아니라 현재 Goal의 source-of-truth, state transition과 consumer boundary에 그 dimension이 적용되지 않는
이유를 기록한다.

Happy path 하나, compile 성공, snapshot 기대값 갱신 또는 mock-only 일치만으로 applicable failure/rollback,
ordering, persistence나 consumer dimension을 대신할 수 없다. Matrix를 작성하는 과정에서 applicable
dimension의 승인 의미가 Final Freeze에 없으면 P2G-10, durable validation owner나 Phase boundary가 없으면
P2G-40 필요를 보고하고 P2G-50을 완료하지 않는다.

#### Master Goal member 내용

- Goal ID와 `master` kind
- complete Phase Goal ID Set과 Phase Plan에서 파생된 순차·병렬 관계
- Phase Goal 사이 dependency와 coordination condition
- Final Freeze와 Phase Plan에서 파생한 cross-Phase invariant 및 exact source
- Final Freeze와 Phase Plan에서 파생한 integration acceptance 및 exact source
- integration acceptance별 `existing_read_only | audit_local_read_only | assigned_execution_node` validation
  disposition, seam locator/evidence contract 또는 owning Phase Goal과 Final Execution Node
- initial/final full baseline owner, Phase별 input-to-seam invalidation 전달과 reusable evidence coordination.
  Master는 이를 mapping할 뿐 validation command 실행이나 durable test 구현을 소유하지 않음
- 전체 완료 조건과 applicable integration risk/reviewer requirement 및 그 근거

Master Goal은 담당 Phase ID, assigned Final Execution Node, code/data change, Phase-local validation 또는 구현
작업을 소유하지 않는다. Master의 integration validation mapping은 Phase Plan의 existing/audit-local seam 또는
Phase Goal이 소유한 Final Execution Node를 참조할 뿐 구현 ownership을 Master로 옮기지 않는다. cross-Phase
invariant와 integration acceptance를 새로 정의하지 않는다. 필요한 의미가 Final Freeze에 없으면 P2G-10,
필요한 dependency·Phase 경계 또는 durable seam owner가 없으면 P2G-40으로 반환한다. 기술적인 교정은 승인된
연속 진행 범위에서 계속하며, 새 정책은 `8.6`에 따라 판정한다. 위임되지 않은 정책·범위 선택에서는 멈춘다.
Phase Plan에 owner와 disposition이 있지만 Goal member의 validation/completion mapping만 누락됐으면 P2G-50
교정 범위다.

#### Goal member identity와 교정

- Goal ID는 Goal Set 내부의 stable member key이고 독립 Exact Artifact Ref가 아니다. exact member 문맥은
  `(Goal Set Ref, Goal ID)`로 식별한다.
- Phase 구조가 동일한 P2G-50 교정은 Predecessor Goal Set Ref를 가진 successor Goal Set을 만들고 Phase ID,
  Phase-to-Goal Member Mapping과 대응 Goal ID를 기본적으로 유지한다. member key collision, corruption 또는
  identity ambiguity 때문에 변경해야 하면 exact reason과 old/new member-key disposition을 correction evidence에
  남긴다. predecessor Goal Set은 수정·삭제하지 않는다.
- Phase 구조, assigned Final Execution Node 또는 Phase-to-Goal cardinality가 바뀌면 P2G-50 교정으로 숨기지
  않고 P2G-40 owner에서 처리한다. 진행 권한은 Reader Contract에 따르며 P2G-50이 Phase를 직접 고치지 않는다.
- P2G에서는 execution attempt 의미의 successor Goal ID를 만들지 않는다.

#### 완전한 작성과 publish

- 모든 Goal member와 Phase-to-Goal Member Mapping을 포함한 complete Goal Set canonical payload를 먼저
  구성한다. `single_phase`는 Single Goal member 하나만, `multi_phase`는 Master Goal member 정확히 하나와
  모든 Phase의 Phase Goal member를 포함한다.
- 일부 Phase 또는 일부 Goal member만 포함한 Goal Set은 candidate로도 current publish하지 않는다.
- candidate Goal Set을 구성한 뒤 repository 표준 구조 검증기를 exact current Freeze, Phase Plan, passed
  P2G-40 Receipt와 Goal Set에 대해 실행한다.

  ```bash
  node core_enhanced/docs/tools/validate_p2g50_goal_set.mjs \
    --freeze <final-policy-freeze.md> \
    --plan <phase-plan.md> \
    --receipt <phase-plan-validation-receipt.json> \
    --quality-contract 2 \
    --goal-set <goal-set.md>
  ```

  검증기는 upstream Ref/hash/result, Goal 구조, Phase-to-Goal 및 Final Execution Node cardinality, Freeze
  PER/Decision/Acceptance traceability, member-local coverage, integration ownership, applicable baseline owner와
  authoring/activation/authorization 상태 및 Goal Quality Contract v2 필드/Validation Dimension disposition을
  기계적으로 검사한다. process exit code가 `0`, 출력 `result`가
  `passed`, `failures`가 empty일 때만 candidate를 complete result에 사용할 수 있다. 이 검증은 Goal scope,
  implementation boundary, dependency 이유, risk 또는 completion plan의 의미적 타당성을 대신 판단하지 않으며
  game test나 production mutation을 수행하지 않는다. 별도 P2G-50 Validation Receipt artifact는 만들지 않는다.
  명시적 `--quality-contract 1`은 이 계약 도입 전에 발행된 immutable historical Goal Set의 구조를 재검증하기
  위한 호환 모드다. Flag는 생략할 수 없고 새 P2G-50 candidate를 publish할 때는 반드시 version `2` flag와
  payload를 사용하며 historical mode로 새 quality requirement를 우회할 수 없다.
  `8.8`의 설계 내용은 existing field의 의미적 검토 기준이다. 구조 검증기의 passed만으로 그 내용을 검토했다고
  주장하지 않는다. 이 개정만을 근거로 존재하지 않는 새 flag/version을 호출하지 않으며, 새로운 machine
  필수 field나 enum이 필요해지면 계약·template·validator·fixture를 함께 변경하고 검증해야 한다.
- Goal Set Ref, P2G-50 Resolution Record Ref Set과 해당 Obligation Supersession Record Ref Set을 하나의 P2G-50
  complete result로 검증한다.
- 필수 구성과 종료 조건이 모두 검증된 경우에만 새 Goal Set을 current로 선택한다.
- validation이 실패하거나 complete result를 만들지 못하면 candidate Goal Set을 current로 선택하지 않고 predecessor selection과
  lifecycle/disposition을 authoritative declaration과 기존 disposition 그대로 유지한다. 정상 current
  predecessor는 기존 current selection을 유지하고, stale predecessor는 stale로 남으며 current로 복원하지
  않는다.

#### 산출물

- current complete Goal Set Ref
- repository 표준 Goal Set 구조 검증 결과: validator version, `passed`, 핵심 count와 empty failure summary
- Goal Set canonical payload의 Goal Structure Fact, complete Goal Member Set, Phase-to-Goal Member Mapping과
  Authoring/Activation/Implementation Authorization Status 및 applicable Shared Validation Baseline Reuse Contract
- `single_phase`이면 Single Goal member 하나, `multi_phase`이면 Master coordination member 하나와 모든 Phase
  Goal member
- Goal Authoring Summary
- 별도 문서가 필요할 때만 만드는 optional 정책·Phase·Goal traceability matrix; 현행 검증기용 Goal Set 내부 파생 표현은 아래 template 계약을 따름
- P2G-50 Resolution Record Ref와 해당 시 Obligation Supersession Record Ref

#### 종료 조건

- final ordered Phase Set의 모든 Phase가 정확히 하나의 Single/Phase Goal member에 연결되고 누락·중복·extra
  또는 historical Phase mapping이 없음
- repository 표준 Goal Set 구조 검증기의 process exit code가 `0`, 출력 `result`가 `passed`, `failures`가 empty임
- `single_phase` Goal Set에는 Single Goal member가 정확히 하나이고 Master/Phase member가 없음
- `multi_phase` Goal Set에는 Master Goal member가 정확히 하나이고 모든 Phase에 해당하는 Phase Goal member가
  정확히 하나씩 있음
- 모든 Single/Phase Goal member의 Phase, Final Execution Node, scope, dependency, policy/decision/acceptance
  coverage, implementation boundary, validation과 applicable migration·risk가 Phase Plan 및 Final Freeze와
  일치함
- Goal Quality Contract Version이 `2`이고 모든 Single/Phase Goal member가 user-visible problem/completion,
  source of truth/owner, Goal-local invariant, consumer verification boundary, structural completion/legacy
  disposition과 canonical documentation obligation을 가짐
- 모든 Single/Phase Goal member의 Validation Dimension Matrix가 10개 dimension을 missing/duplicate/extra 없이
  disposition하며, `required`에는 direct evidence contract와 completion requirement가 있고 `N/A`에는
  applicability를 부정하는 구체적 근거가 있음. applicable negative/edge dimension을 happy path나 broad test
  이름만으로 대체하지 않음
- applicable Shared Validation Baseline Reuse Contract가 Goal Set common payload에 완전하며 initial/final full
  validation owner는 executable Single/Phase Goal이고, 각 Phase Goal의 impact map과 focused rebaseline plan이
  Phase 시작만을 이유로 full validation을 반복하지 않으면서 changed production/test input을 빠짐없이 invalid 처리함
- 모든 Single/Phase Goal의 existing boundary/prerequisite/invariant/validation/unknown 항목에 `8.8`의 적용 가능한
  코드 조사·상세 설계·준비 점검 인계가 있고, “구현자가 추후 전부 설계”라는 문구로 reviewed boundary를 대체하지 않았음
- Master Goal이 구현 작업을 소유하지 않고, coordination relation, cross-Phase invariant, integration
  acceptance와 전체 완료 조건이 exact Final Freeze/Phase Plan source에서 파생되며, 모든 integration
  acceptance가 read-only seam 또는 owning Phase Goal/Final Execution Node에 완전히 mapping됨
- Goal member key가 Goal Set 안에서 unique하고 각 exact member가 `(Goal Set Ref, Goal ID)`로 식별됨
- Goal Set이 `Authored`, `Not Activated`, `Implementation Authorized: No`이며 durable handoff 또는 P2G-60
  authorization boundary를 소유하지 않음
- published Goal Set이 exact current Phase Plan Ref와 Final Policy Freeze Ref를 가리키며 lifecycle이 `current`
- complete P2G-50 result의 필수 구성, reference와 coverage가 검증된 뒤 current Goal Set으로 선택됨
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-50이 소유한 모든 applicable active obligation leaf가 current Goal Set으로 effectively resolved됨

#### 되돌림

- Goal 작성 중 새로운 material policy가 발견되면 `8.4`에 따라 Post-Freeze Change, 두 closure, complete
  initial obligation member set과 Invalidated Artifact Disposition Set을 하나의 Change Obligation Declaration으로
  완전하게 publish한다. declaration이 current로 선택되면 영향 artifact와 기존 Freeze의 `stale`
  disposition을 파생하되 bytes와 Membership은 보존한다. `P2G-10` successor revision 필요를 보고한 뒤
  Reader Contract에 따라 P2G-10에서 `8.6`의 위임 결정/사용자 질문으로 처리하거나 필요한 이동 권한을 요청한다. 새 material 결정 전에는
  영향을 받는 성공 경로를 재개하지 않는다. declaration의 필수 구성을 완성하지 못하면 수리를 시작하지 않는다.
- Phase boundary나 dependency만 잘못된 경우에도 영향받은 Phase/Goal artifact와 필요한 P2G-40/50 obligation을
  complete declaration으로 publish한다. derived `stale` disposition을 기록하고, 승인된 연속 진행 범위이면
  P2G-40에서 재검토·수리한다. 새 정책 의미는 `8.6`에 따라 처리하며 승인 범위 밖이면 필요한 입력을 보고한다.
- reviewed current behavior, implementation delta 또는 consumer boundary evidence가 부족하거나 달라졌지만
  policy/Phase 의미를 아직 바꿀 수 없다면 P2G-20으로 반환한다. 안전하게 해소 가능한 근거 문제는 허용 범위에서
  조사·재검증하고, 확인할 수 없는 필수 사실이나 새 material 선택이면 멈춘다.
- Goal-local 문구, validation plan 또는 applicable risk/reviewer requirement만 불완전하고 Phase 구조가
  동일하면 P2G-50 successor Goal Set 교정 범위로 처리할 수 있다.

#### 권한 경계

P2G-50은 Goal authoring만 소유한다. repository baseline freeze, dirty ownership 판정, authorization grant,
Goal 실행 target 선택, implementation mutation, validation과 completion audit는 후속
[Goal Execution Pipeline](./goal_execution_pipeline.md)의 책임이며 이 단계에서 수행하거나 승인하지 않는다.

<!-- P2G-50 END -->

<!-- P2G-60 START -->
### P2G-60 — 구현 전 Handoff

#### 목적

current Final Policy Freeze, Phase Plan과 Goal Set의 exact 인수 관계, 남은 gap·blocker와 P2G 권한 경계를
durable Handoff Artifact에 기록한다. 기존 정책, Phase 또는 Goal 내용을 수정·복제하지 않으며 Goal activation,
implementation authorization과 production mutation을 수행하지 않는다.

#### 입력

- exact current Composite Basis
- exact current Final Policy Freeze Ref
- exact current Phase Plan Ref
- exact current Goal Set Ref
- 남은 Evidence Gap, 구현 전 blocker와 알려진 limitation
- rewrite이면 exact predecessor Handoff Artifact Ref 또는 `N/A`
- applicable active P2G-60 Obligation Key Set

#### 진입 조건

- Freeze, Phase Plan과 Goal Set이 모두 current이고 exact prerequisite chain으로 연결됨
- exact current Composite Basis가 Freeze, Phase Plan과 Goal Set Ref를 선택하며 stale 또는 historical artifact가
  입력에 없음
- Goal Set이 `Authored`, `Not Activated`, `Implementation Authorized: No`임
- Final Freeze의 모든 in-scope Policy Element Revision이 Phase Plan의 primary Phase를 거쳐 정확히 하나의
  Single/Phase Goal member까지 추적됨
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-60보다 앞선 owner stage의 모든 applicable active obligation leaf가 effectively resolved됨
- P2G-60 소유 unresolved active obligation leaf는 durable handoff 재작성 scope로 허용됨

#### 주요 활동

- exact current Final Policy Freeze, Phase Plan과 Goal Set Ref 및 Result Composite Basis를 고정하고 prerequisite
  chain과 currentness를 확인한다. Handoff Artifact는 upstream member payload를 복사하지 않고 exact owning
  artifact Ref를 가리킨다.
- Final Freeze의 in-scope Policy Element Revision에서 Phase Plan primary Phase와 Goal Set의 Phase-to-Goal Member
  Mapping까지 traceability가 빠짐없이 이어지는지 확인한다. Master Goal은 구현 owner로 계산하지 않는다.
- upstream artifact에 이미 기록된 remaining Evidence Gap, 구현 전 blocker, known limitation과 out-of-scope,
  `8.8`의 미완료 상세 설계·공유 계약 준비 조건을 누락하거나 확정 사실로 바꾸지 않고 요약한다. 새 registry, blocker 상태 모델 또는 별도 evidence schema를
  만들지 않는다.
- P2G가 정책 동결, Phase 설계와 Goal 작성을 전달했다는 범위와 Goal activation, 구현 승인, production
  mutation 및 completion 판정을 수행하지 않았다는 authorization boundary를 기록한다. Handoff Artifact의
  `current`는 구현 준비나 권한을 뜻하지 않는다.
- exact refs, remaining summary와 scope/authorization boundary를 포함한 complete durable Handoff Artifact
  canonical payload를 구성한다.
- P2G-60 obligation이 있으면 Handoff Artifact를 가리키는 Resolution Record를 생성한다.
- Handoff Artifact Ref와 P2G-60 Resolution Record Ref Set의 필수 구성과 상호 참조를 검증한 뒤 complete
  P2G-60 result로 publish한다. complete result만 current Handoff로 선택한다.

#### 산출물

- current Handoff Artifact Ref
- exact Result Composite Basis와 upstream Final Policy Freeze, Phase Plan, Goal Set Ref
- remaining Evidence Gap, blocker, limitation과 out-of-scope summary
- P2G Scope/Authorization Boundary
- P2G-60 Resolution Record Ref Set: explicit empty / record refs

#### 종료 조건

- Handoff Artifact가 exact current Final Freeze, Phase Plan과 Goal Set으로 구성된 Result Composite Basis를 가리킴
- prerequisite chain과 lifecycle/current selection 검증이 통과함
- 모든 in-scope Policy Element Revision이 primary Phase와 정확히 하나의 Single/Phase Goal member까지 추적되며
  Master Goal이 구현 owner로 계산되지 않음
- 알려진 Evidence Gap, blocker, limitation과 out-of-scope가 누락되거나 확정 사실로 바뀌지 않음
- P2G Scope/Authorization Boundary가 Goal activation, implementation authorization, production mutation 또는
  completion을 승인하지 않음
- 이 파이프라인 안에서는 production mutation이 발생하지 않음
- Handoff Artifact가 exact current Freeze, Phase Plan과 Goal Set을 가리키며 lifecycle이 `current`
- complete P2G-60 result의 필수 구성과 reference가 검증된 뒤 current Handoff로 선택됨
- 모든 applicable Change에 valid authoritative declaration이 존재함
- P2G-60이 소유한 모든 applicable active obligation leaf가 current Handoff Artifact로 effectively resolved됨

#### 되돌림과 중단

- upstream 정책, code review, Freeze, Phase 또는 Goal artifact의 결함은 해당 owner로 반환한다. 허용된
  non-material 수리이면 계속하고, 새 결정·권한·해소 불가능한 필수 근거 문제이면 `8.6`에 따라 멈춘다.
- exact upstream basis가 그대로이고 Handoff summary나 authorization boundary만 잘못됐다면 predecessor를
  보존한 P2G-60 successor Handoff로 교정할 수 있다.
- P2G-60 종료 뒤에도 [GEX-00](./goal_execution_pipeline.md), Goal 선택 또는 mutation을 자동 시작하지 않는다.

<!-- P2G-60 END -->

## 8. 모든 단계에 공통으로 적용되는 규칙

다음 규칙은 특정 단계 하나가 아니라 파이프라인 전체에 적용한다.

### 승인 범위 안의 계속 진행과 단계 권한

**진행 권한과 단계 유효성은 별개다.** 사용자가 허용한 Topic·범위·종료 결과 안에서 필요한 단계를 계속
수행할 수 있지만, 각각의 entry/exit·owner·불변성·coverage 검사는 생략할 수 없다. 단계 결과가 완성됐다는
사실 자체가 새로운 범위의 권한을 만들지도 않는다.

Reader Contract의 `single_stage | bounded_pipeline`이 요청 경계를 소유한다. 다음 원칙을 모든 section에 적용한다.

- 단일 단계 요청이면 그 단계에서 종료한다. 연속 진행 요청이면 필요한 선행·후속·non-material 수리를
  각 owner 단계에서 수행하고 **단계 경계만을 이유로 새 승인을 요구하지 않는다.**
- 승인된 정책의 표현·참조·구조 오류와 검증 실패는 결과를 실제로 완성하는 작업의 일부다. 범위 안에서
  원인을 조사해 수정·재검증한다. `8.6`의 사용자 선택이 필요한 정책, 범위 확대, 사용자 checkpoint, 미승인 위험·권한 또는
  안전한 방법으로 해소할 수 없는 필수 근거 문제에서만 사용자 개입을 요청한다.
- material finding은 영향받은 성공 batch를 중단시키고 P2G-10으로 반환한다. 이는 사용자 답변 대기와
  별개다. 기존 결정이 있으면 근거를 연결하고, 없으면 `8.6`에 따라 위임된 통상적 정책은 결정·기록하며
  사용자 선택이 필요한 정책만 질문한다. owner와 필수 단계 검증은 생략하지 않는다.
- 질문에 대한 답변은 중단된 기존 연속 진행 범위에서 반영·재개할 수 있다. 종료 결과·scope·예산·권한이
  여전히 유효한지 확인한다. 완료된 요청의 권한을 새 Topic이나 새로운 구현 작업으로 이월하지 않는다.
- 지침에 맞춰 일을 끝내기 위한 자체 검토는 기본 활동이다. 사용자 검토나 독립 reviewer의 실제 승인이
  필수라고 지정된 경우에는 이를 자기 검토로 대체하지 않는다. 검토 계획을 실행된 검토로 보고하지 않는다.
- P2G-50에서 Goal을 작성하거나 P2G-60에서 인계해도 Goal activation, production mutation과 구현 완료를
  승인하지 않는다. 이 문서의 진행 권한은 P2G 안에서만 유효하다.

실행 범위는 기존 Working Policy Document view 또는 작업 기록에 한 번 간결하게 기록한다. 초기화 전에는
외부 응답/기록으로 충분하다. 새 Execution Contract artifact·Ref·canonical 정책 필드는 만들지 않는다.

```text
요청 근거 / 대상 Topic·승인 scope
진행 방식: single_stage | bounded_pipeline
종료 결과: 지정 단계/산출물
허용 범위: 필요한 P2G 단계·문서 쓰기·안전한 조사/검증; 명시적 제한
현재 단계 / 마지막 유효 결과 / 중단·재개 조건(해당할 때)
```

단계마다 위 내용을 전부 복사하지 않는다. 중요한 전환·새 근거·실패·교정·결정만 기존 실행 기록에 남기고,
사용자에게는 요청 완료·실제 결정 필요·중요 blocker에서 요약한다. 다른 단계로 이동할 때 현재 scope와
artifact basis가 유효한지 확인하되, 이미 검증한 무관한 자료를 기계적으로 재검사하지 않는다.

### 8.0 Artifact Identity와 Exact Reference

P2G의 단계 인수에는 producer가 발행한 typed `Exact Artifact Ref`를 사용한다. Ref는 artifact 종류와 stable
Artifact ID를 반드시 포함하고, producer가 같은 구현 안의 무결성 확인에 사용하는 Content Digest를 선택적으로
포함할 수 있다.

```text
Exact Artifact Ref :=
  Artifact Type
  Artifact ID
  Content Digest 또는 N/A
```

각 artifact의 typed alias를 사용한다. section 5.1 Artifact Catalog의 `Canonical artifact name`과 `Typed exact
identity`가 이름 mapping의 유일한 owner다. 아래는 자주 쓰는 예이며, 다른 독립 artifact도
`{Artifact Type} Ref` 형식으로 같은 구조를 사용한다.

```text
Working Revision Ref
Policy Element Revision Ref
Policy Intake Snapshot Ref
Review Batch Ref
Code Review Binding Ref
Reviewed Source Basis Ref
Final Policy Freeze Ref
Phase Plan Ref
Phase Plan Validation Receipt Ref
Goal Set Ref
Handoff Artifact Ref
```

단계 입력·출력과 dependency는 typed Exact Artifact Ref로 표현한다. producer가 발행한 Ref를 후속 consumer가
그대로 보존하고 exact-match하는 것이 P2G 계약이다.

- Artifact ID, Ref와 optional Content Digest는 자신이 식별하는 payload에 포함하지 않는다. identity를 계산하는
  reference dependency에는 순환이 없어야 한다.
- section 9 template가 artifact 자신의 `... Ref`를 payload field와 함께 표시해도 그 Ref는 identity envelope
  metadata이며 canonical payload member가 아니다.
- member set은 owner artifact 안에서 stable key로 구분한다. producer가 안정적인 로컬 표현을 위해 정렬하거나
  digest를 사용할 수 있지만, 다른 실행자가 같은 byte sequence나 digest를 재생성할 의무는 없다.
- `... Ref Set`은 독립 Set artifact를 뜻하지 않는다. 별도 owner/consumer가 명시되지 않으면 상위 artifact의
  Exact Artifact Ref member list다.
- effective view, materialized registry, reverse index와 status projection은 authoritative artifact에서
  파생하며 별도 Exact Artifact Ref를 만들지 않는다.
- `Basis`는 owner 단계가 요구하는 ordered Exact Artifact Ref 집합과 필요한 authoritative field reference다.
  Basis 자체가 독립 immutable artifact로 publish되지 않으면 별도 digest를 만들지 않는다.
- 외부 repository revision, file content identifier와 deployed version은 외부 source identity다. P2G Content
  Digest와 혼동하지 않고 해당 source가 정의한 exact identifier 형식을 보존한다.

같은 단계 결과의 사람용 문서에서는 authoritative field나 set을 한 번만 선언한다.

- `absent`와 `explicit empty`의 의미 차이는 유지한다. 다만 한 owner field에서 `explicit empty`를 선언했다면
  entry summary, 본문, Complete Stage Result와 exit summary가 같은 값을 반복 전개하지 않고 그 field를
  참조하거나 생략한다.
- 서로 다른 typed set이 모두 empty이고 하나의 전제에서 결정적으로 파생되면 사람용 view는 예를 들어
  `Repair Basis: no applicable Change; obligation/resolution/supersession sets empty`처럼 한 번에 축약할 수 있다.
  machine-readable canonical payload와 validation은 typed field의 구분을 유지한다.
- exact Ref 목록을 owner artifact가 이미 고정했다면 consumer의 사람용 view는 owner Ref와 stable selector만
  기록할 수 있다. exact expansion과 duplicate/missing/extra 검사는 validation receipt나 자동 검사 결과가
  소유하며, 같은 목록을 설명용 표와 검증용 표에 반복하지 않는다.
- 축약된 표시가 identity, currentness 또는 completeness를 약화하지 않는다. 자동 검증 결과가 없거나
  실패했으면 사람이 목록을 재기록해도 stage gate를 통과한 것으로 간주하지 않는다.

Content Digest를 사용하는 구현에서는 동일 구현 안의 내용 무결성 확인에만 사용한다. hash algorithm,
serialization, Unicode normalization과 byte-level 재현은 이 문서가 강제하지 않는다. consumer는 producer가
발행한 Ref를 임의로 다시 계산하거나 다른 지문으로 교체하지 않는다. 승인, 진실성, completeness와
currentness는 digest가 아니라 사용자 직접/위임 decision, evidence와 단계 exit condition으로 판정한다.

### 8.1 정책 발견 사항 분류와 결정 권한

모든 발견 사항은 다음 중 하나의 primary classification과 disposition을 가져야 한다.

| 분류 | 판정 기준 | 결정 주체와 처리 |
| --- | --- | --- |
| `material_policy` | 사용자나 consumer가 경험·의존하는 의미가 달라짐 | P2G-10이 소유한다. `8.6`의 통상적 정책은 위임 범위에서 AI가 결정·기록하고, 사용자 선택이 필요한 경우에만 Open Policy Question으로 질문한다. |
| `technical_decision` | 승인된 정책 의미를 보존하는 내부 realization 선택 | AI가 실제 근거, 선택과 결과를 Technical Decision Log에 기록한다. |
| `evidence_gap` | 필요한 사실을 접근하거나 신뢰성 있게 판정할 수 없음 | `unknown`과 영향받는 claim을 기록한다. 필요한 결정이나 다음 단계의 전제이면 해소 전까지 진행하지 않는다. |
| `implementation_finding` | 정책 변경 없이 구현 경계·난이도·작업 계획에만 영향을 줌 | Code Review Finding 또는 후속 Work Package 입력으로 기록한다. |
| `out_of_scope` | 현재 Policy Topic과 in-scope contract에 속하지 않음 | 별도로 기록하고 route하되 현재 scope나 완료 조건을 자동 확장하지 않는다. |

한 finding이 여러 성격을 가진 것처럼 보이면 사용자-visible 의미를 바꾸는지 먼저 판정한다. 의미가 바뀌면
`material_policy`다. 필요한 사실이 없어 이 판정 자체를 신뢰할 수 없으면 우선 `evidence_gap`으로 기록하고,
evidence가 확보된 뒤 다시 분류한다. In-scope 정책을 성립시키는 데 필수인 항목은 단순히
`out_of_scope`로 내려놓을 수 없다.

#### Material Policy의 의미와 결정 권한

다음 중 하나를 바꾸는 선택은 material policy다. **material 여부는 의미의 영향 분류이고, 통상적 여부는
결정 권한의 분류다.** material이라는 이유만으로 사용자 답변을 요구하지 않는다. `8.6`의 기준·위임 근거를
충족하면 AI가 P2G-10에서 결정하고, 충족하지 못해 사용자 선택이 필요하면 질문한다.

- gameplay, balance와 UX
- public contract와 observable behavior
- identity, 의미 또는 consumer-visible 처리 순서
- persistence lifetime과 data loss
- compatibility와 migration behavior
- rollout, fallback 또는 이전 버전 처리

DTO, schema, message ordering처럼 구현 세부사항으로 보이는 선택도 identity, consumer-visible meaning/order,
compatibility, migration loss, persistence lifetime 또는 rollout behavior를 바꾸면 `material_policy`다.

#### AI가 결정할 수 있는 Technical Decision

승인된 정책 의미를 보존한다는 근거가 있을 때 다음은 AI가 결정할 수 있다.

- module, type과 file 배치
- private data layout과 내부 encoding
- 내부 algorithm과 realization 방식
- Work Package 구성
- dependency와 구현 순서
- validation 방법

정책 의미 보존 여부가 불명확하면 technical decision으로 확정하지 않고 `material_policy` 또는
`evidence_gap`으로 다시 분류한다.

#### 승인 경계

다음 결정은 서로를 대신하지 않는다.

```text
정책 승인
  != Goal 작성 요청
  != 구현 시작 또는 production mutation 승인
  != 구현 완료 판정
```

사용자 직접 결정 또는 위임에 따른 AI 결정은 해당 Decision Revision Ref의 target meaning만 확정한다.
통상적 정책의 위임도 Scope·Goal·구현 권한을 확대하지 않는다. 단독 결정만으로 Goal 작성,
구현 시작, 코드·데이터 변경 또는 완료 판정을 수행할 수 없다. 이미 Goal 작성까지 승인된 `bounded_pipeline`이
질문 때문에 중단된 경우에는 그 답변을 반영하고 기존 권한으로 재개할 수 있다. 새로운 구현 권한이 생기는 것은 아니다.

#### Unknown과 범위 안전

- `unknown`은 current, compatible, safe 또는 pass가 아니다.
- 접근하지 못한 consumer나 환경의 동작을 추측해 정책 질문을 닫지 않는다.
- 범위 밖 finding은 기록하고 별도 작업으로 route할 수 있지만 현재 scope를 자동 확장하지 않는다.
- AI의 추천, 위임된 AI 결정, 사용자 직접 결정, 현재 사실과 구현 제안은 서로 구분해 기록한다.

이 권한 경계는 P2G 내부의 policy closure와 authoring만 소유한다. Goal 실행 요청, 구현 상태와 완료 판정은
[Goal Execution Pipeline](./goal_execution_pipeline.md)이 별도로 소유한다.

### 8.2 Source와 Evidence

모든 review와 계획에서 source claim을 다음처럼 분리한다.

| Claim | Owner와 의미 | 기본 처리 |
| --- | --- | --- |
| Workspace Actual | exact checkout의 reachable code/data/store/consumer와 scoped dirty state | repository current claim에 필수 |
| Released Actual | 식별된 build/publish revision의 내용 | release/compatibility claim에 필요하며 확인하지 못하면 `unknown` |
| Deployed Actual | 식별된 환경의 revision/config/data/consumer set | live/rollout claim에 필요하며 확인하지 못하면 `unknown` |
| Approved Target | 사용자 직접 결정 또는 `8.6`의 위임 근거를 갖춘 exact Decision과 current Policy Element Revision | target meaning의 유일한 owner |
| Observable Policy Acceptance | target policy realization을 외부에서 판정할 P2G-10 승인 기준 | validation seam과 별도로 기록 |
| Implementation Delta | Current Actual과 Approved Target 사이의 필요한 변경 | evidence 기반 review 결과 |
| Proposed Implementation | 아직 실행하지 않은 내부 구현 선택과 경계 | actual 또는 approved policy로 표현 금지 |

Workspace Actual로 Released/Deployed Actual을 대신하거나, Proposed Implementation을 current behavior로 표현하지
않는다. P2G-20 Review Scope/Source Requirement Plan에서 Released/Deployed requirement가 `not_applicable`이거나
immutable conditional condition이 false이면 `N/A`와 evidence를 기록하고, 적용되지만 확인할 수 없으면
`unknown`이다.

#### Source requirement와 observation 경계

Source requirement와 그 requirement에 대한 실제 관찰 결과는 P2G-20이 소유하는 서로 다른 derived artifact다.
어느 것도 Working Revision policy basis에 포함하지 않는다.

```text
P2G-20 Review Scope/Source Requirement Plan — Derived Artifact
  Review Scope/Source Requirement Plan Ref
  Bound Input Working Revision Ref
  Requirement ID
  Exact Policy Element Revision과 required claim/boundary
  Source type/role
  Required identity/location constraint
  Applicability/Necessity: required | conditional | not_applicable
  Applicability condition
  Policy/Scope Basis Reference
  Validation requirement

Source Observation/Evidence Registry — P2G-20 Derived Artifact
  Source Observation Ref
  Evaluated Working Revision Ref
  Review Scope/Source Requirement Plan Ref
  Requirement ID
  Observed source identity/location/version
  Disposition
  Observation evidence
  Limitation 또는 N/A reason
  Observation timestamp/version
```

Plan은 exact Input Working Revision의 current Policy Element, Decision과 Observable Policy Acceptance에서 required claim/boundary를
도출한다. Plan의 canonical field가 바뀌면 같은 Input basis에 대한 새 immutable plan 또는 새 P2G-20 Review Batch가
필요하지만 그 사실만으로 successor Working Revision을 만들지 않는다. plan 변경이 승인 scope, Policy Element
statement, Decision 또는 Observable Policy Acceptance 의미가 잘못됐음을 드러내면 material finding으로 분류하고 P2G-20 batch를
abort한 뒤 P2G-10 복귀 필요를 보고한다.

각 Source Observation은 자신이 평가한 exact Input Working Revision Ref와 published Review
Scope/Source Requirement Plan Ref를 함께 가리키는 immutable envelope다. current observation은 다음
조건을 모두 만족할 때만 target Review Batch에서 선택할 수 있다.

```text
is_current_source_observation(observation, target_review_batch) :=
  observation.Evaluated Working Revision Ref
    == target_review_batch.Input Working Revision Ref
  and observation.Review Scope/Source Requirement Plan Ref
    == target_review_batch.Review Scope/Source Requirement Plan Ref
  and observation.Requirement ID가 그 plan의 exact member임
  and observation이 요구하는 source/evidence currentness를 만족함
```

Plan requirement의 역방향 coverage도 별도로 검증한다.

```text
is_plan_requirement_evaluated(requirement, target_review_batch) :=
  exists current Source Observation observation:
    observation.Review Scope/Source Requirement Plan Ref
      == target_review_batch.Review Scope/Source Requirement Plan Ref
    and observation.Requirement ID == requirement.Requirement ID
    and observation이 exact target Input Working Revision에 bind됨
    and observation.Disposition이 유효함
```

target Review Batch의 모든 Plan Requirement ID가 위 판정을 만족해야 한다. 하나의 requirement에 여러 source나
consumer가 필요하면 최소 한 row의 존재만으로 claim coverage가 완료된 것으로 보지 않고 plan이 요구한 전체
observation을 확인한다. `not_applicable` 또는 exact Input basis에서 false로 입증된 `conditional` requirement만
evidence가 있는 `N/A` observation으로 평가할 수 있다. `required` 또는 true/unknown인 `conditional`
requirement에는 `N/A`를 사용할 수 없다.

predecessor Working Revision의 raw evidence가 내용상 재사용 가능해도 그 observation row 자체를 successor에서
current로 선택하지 않는다. raw Evidence ID는 보존·재사용할 수 있지만, target Review Batch의 Input Working
Revision과 plan requirement를 다시 평가한 새 immutable Source Observation envelope를 만들어야 한다.

`N/A`는 plan requirement applicability의 관찰 결과이지 policy basis 변경 명령이 아니다. current plan
requirement가 `not_applicable`이거나, `conditional` condition이 exact Input basis에서 false라고 evidence로
평가된 경우에만 observation registry에 기록할 수 있다. `required`를 N/A로 만들거나 plan의
applicability/condition을 observation만으로 바꿀 수 없다. plan만 잘못됐으면 새 candidate plan/batch를 만들고,
정책 의미나 승인 scope가 잘못됐으면 material finding으로 route한다.

#### Source disposition

각 required repository, data/store, server, consumer, release/deploy source는 다음 중 하나를 가진다.

| Disposition | 의미 |
| --- | --- |
| `confirmed` | exact source와 observation evidence로 claim을 확인함 |
| `inaccessible` | required source를 식별했지만 접근할 수 없음 |
| `explicitly_excluded` | 승인된 scope상 제외됐고 제외가 현재 claim을 침범하지 않음 |
| `unknown` | source 또는 결과를 신뢰성 있게 판정할 수 없음 |
| `N/A` | current requirement가 `not_applicable`이거나 immutable conditional condition이 current basis에서 false임을 evidence와 함께 표시함 |

`inaccessible`, `unknown`과 `N/A`는 서로 대체할 수 없다. `required` requirement 또는 condition이 true/unknown인
`conditional` requirement에 `N/A`를 기록해 applicability gate를 우회할 수 없다. Required source의
inaccessible/unknown이 current, target delta 또는 acceptance 판단을 침범하면 해당 claim은 완료되지 않는다.

#### Evidence reference

Durable evidence는 최소 다음을 식별한다.

```text
Review Evidence Ref
Repository/environment identity
Path, artifact 또는 endpoint
Exact external revision, content identifier 또는 temporal version
Observation command/method
Observed result
Timestamp — 시간에 따라 바뀌는 사실인 경우
Supported claim
Limitations 또는 environment restriction
```

- 파일, type, schema 또는 test의 존재만으로 runtime reachability를 증명하지 않는다.
- test의 zero execution, skip, unknown 또는 실행 불가는 pass가 아니다.
- current, compatible, safe와 complete 같은 claim은 이를 직접 지지하는 evidence가 있어야 한다.
- confidence와 uncertainty는 evidence result와 별도로 기록하며 높은 confidence로 missing evidence를 대체하지
  않는다.
- 동일 evidence를 successor에서 재사용하려면 source identity/currentness와 changed premise로부터의 독립성을
  positive evidence로 확인하고 `8.4`의 binding 규칙을 통과해야 한다.

이 source/evidence 계약은 P2G의 read-only policy/code review 범위만 소유한다. Goal 실행 상태, publication,
배포 또는 운영 currentness를 추론하지 않는다.

### 8.3 Traceability

Traceability는 별도 canonical artifact를 추가하는 요구가 아니라 owner artifact의 exact Ref/member relation을
따라 current policy meaning이 downstream 결과에 빠짐없이 전달됐는지 확인하는 규칙이다.

최소 정방향 경로는 다음과 같다.

```text
Policy Element Revision
  -> current Decision Revision + owned Observable Policy Acceptance
  -> Code Review Binding + Review Batch
  -> Final Policy Freeze Membership
  -> Work Package / Atomic Group coverage
  -> primary Phase
  -> exactly one Single/Phase Goal member
  -> P2G-60 Handoff가 선택한 exact Goal Set
```

- 각 relation은 exact owner artifact와 stable member key를 가리킨다. Phase Plan의 Freeze-scoped selector는 exact
  Bound Final Policy Freeze Ref와 stable Element ID의 결합으로 해석한다. label이나 문구 유사성으로 연결하지
  않는다.
- Policy Element Revision에서 Handoff까지 정방향으로 누락 없이 도달해야 하고, downstream member에서 upstream
  Policy Element/Decision/Acceptance와 evidence를 역방향으로 확인할 수 있어야 한다.
- Master Goal은 coordination owner이지 Policy Element의 primary implementation owner가 아니다.
- 하나의 Policy Element가 여러 Work Package나 supporting Phase에 연결될 수 있지만 primary Phase와 최종
  Single/Phase Goal mapping은 하나여야 한다. supporting relation은 primary ownership을 대체하지 않는다.
- current traceability view는 current Freeze, Phase Plan, Goal Set과 Handoff에서 계산한다. historical/stale
  artifact의 edge를 current coverage에 섞지 않는다.
- 별도 `Policy Element-to-Code Review Matrix`, `Policy Element-to-Phase Matrix`와 정책·Phase·Goal Matrix는
  optional view다. view 파일의 부재만으로 entry/exit를 막지 않는다. required traceability는 owning artifact의
  relation에서 직접 검증하며, 그 관계 자체의 누락은 계속 실패다.
- Phase의 authoritative ownership은 ordered Phase Set의 Freeze-scoped selector relation, Goal 연결은
  Phase-to-Goal Member Mapping이다. 현행 P2G-50 검증기가 Goal Set 문서 안의 `Policy–Phase–Goal Traceability`
  표를 읽는 경우 이 표는 해당 관계에서 생성하는 **검증기 입력용 파생 표현**으로 제공한다. 별도 prerequisite
  artifact나 새 정책 authority가 아니며 독립적으로 편집해 의미를 바꾸지 않는다. 해당 도구 계약이 바뀌기
  전에는 이 입력 표현을 생략한 채 도구 검증을 통과했다고 주장할 수 없다.
- relation 누락·모순은 이를 소유한 가장 이른 P2G 단계로 반환한다. Reader Contract의 승인 범위 안이면
  그 owner에서 수리·재검증하고 진행한다. current하지 않은 입력으로 후속 단계를 강행하지 않는다.

### 8.4 변경과 재검토

변경은 기존 결정과 산출물을 소급 수정하지 않고 새 evidence와 disposition으로 처리한다.

#### 승인된 implementation delta와 premise change의 경계

- P2G의 Post-Freeze Change는 approved policy meaning, reviewed current behavior/boundary, Freeze completeness,
  Phase dependency 또는 Goal scope의 전제를 바꾸거나 거짓으로 만드는 사건이다.
- accepted P2G-60 Handoff와 exact Goal member가 이미 승인한 implementation delta를 GEX-10이 수행해 source
  bytes와 Workspace Actual이 달라지는 것은 예상된 realization이며, 그 사실만으로 Post-Freeze Change를 만들거나
  Freeze/Phase Plan/Goal Set/Handoff를 stale 처리하지 않는다.
- 완료된 predecessor Phase의 expected result도 다음 Phase의 execution basis 일부다. 다음 Phase를 위해 P2G-20
  review나 P2G-60 Handoff를 자동 재작성하지 않는다.
- 구현 중 current behavior, consumer, migration 또는 validation evidence가 기존 review premise가 실제로
  불완전·오류였음을 드러내거나 Goal 밖 의미/작업이 필요해지면 예상 delta가 아니다. GEX finding을 exact
  evidence로 받아 이 section의 Change/closure와 가장 이른 P2G owner를 판정한다.
- 구현 이후 사용자가 policy, Phase 또는 Goal 구조를 다시 계획해 달라고 요청하면 completed/partial execution
  evidence를 보존한 successor P2G 경로를 사용한다. GEX가 P2G artifact를 직접 고치거나 자동 복귀하지 않는다.

#### Freeze 이전 변경

- 아직 Final Policy Freeze가 없어도 기존 Policy Element Revision의 내용이나 상태를 변경하지 않는다.
- 정책 의미가 바뀌는 affected current element는 predecessor를 가리키는 successor Policy Element Revision을
  `open`으로 시작한다. P2G-10의 의미 보존 Decision 재분할에 따른 reference-only successor는 그 절의 별도
  계약을 적용하며, 의미가 같은 것까지 새 material question으로 만들지 않는다.
- 기존 Working Revision을 수정하지 않는다. 변경된 decision, question, closure 또는 current element map을
  고정하는 immutable successor Working Revision을 만들고 Working Policy Document의 Current Working
  Checkpoint로 그 successor를 선택한다. `open` question이 남은 checkpoint는 P2G-10 complete result가 아니다.
  변경 전 policy basis와 review evidence는 predecessor history로 보존한다.
- Policy Impact Closure에 포함된 element만 successor `open` revision을 만든다. Artifact Invalidation Closure에만
  포함된 element의 정책 revision은 재개방하지 않는다.
- Code Review, binding 또는 다른 derived artifact의 추가·disposition 변경만으로는 Working Revision을 만들지
  않는다. 이들은 policy basis 밖의 Derived Artifact Registry에 기록한다.

#### 두 종류의 영향 범위

변경은 다음 두 closure를 별도로 계산하고 기록한다.

| Closure | 시작점과 edge | 결과와 효과 |
| --- | --- | --- |
| Policy Impact Closure | direct material finding에서 시작해 Policy Element 사이의 semantic premise/meaning dependency만 추적 | 재결정이 필요한 Element ID 집합. 각 요소에 successor `open` revision을 만든다. |
| Artifact Invalidation Closure | changed premise 또는 changed Policy Element Revision에서 traceability edge를 따라 Code Review, Work Package, Atomic Group, Phase Plan, Goal과 Handoff로 진행 | current 근거를 잃는 artifact 집합. 각 artifact를 `stale` 처리한다. |

Policy Impact Closure는 artifact 공동 소유 관계로 확장하지 않는다. 두 Policy Element가 같은 Work Package,
Atomic Group, Phase 또는 Goal을 공유한다는 사실만으로 다른 요소를 재개방하지 않는다. 다만 공유 artifact는
변경된 요소에 의존하므로 Artifact Invalidation Closure에 포함될 수 있다.

각 closure에는 direct member, transitive member, 사용한 edge와 제외 근거를 기록한다. Policy dependency를
확정할 수 없으면 `evidence_gap`으로 중단하며 편의상 다른 요소를 재개방하지 않는다. Artifact dependency를
분리할 수 없으면 안전하게 포함할 수 있는 가장 가까운 aggregate artifact 전체를 invalidation closure에
포함한다.

Post-Freeze Change Record는 origin basis, changed premise와 두 closure를 고정한 immutable event다.
`unresolved|resolved|superseded`를 record 안의 mutable status로 저장하지 않는다. 상태는 아래 obligation,
resolution과 supersession을 target basis에 대조해 파생한다.

#### Composite Basis

currentness와 resolution은 단일 artifact lineage가 아니라 다음 exact component의 조합으로 판정한다.

```text
Composite Basis
  Working Revision Ref
  Final Policy Freeze Ref 또는 N/A
  Phase Plan Ref 또는 N/A
  Goal Set Ref 또는 N/A
  Authoritative Change Obligation Declaration Ref Set
```

각 단계는 Target Basis Projection이 요구하는 predecessor component를 exact하게 참조해야 한다. 단,
downstream-only change의 successor Working Revision처럼 기존 Freeze의 policy meaning을 바꾸지 않는 overlay
basis는 positive compatibility evidence와 함께 current Freeze를 유지할 수 있다. Composite Basis는 모든
component가 동시에 생성됐다고 가정하지 않고 component별 predecessor 관계, exact binding과 명시된
compatibility를 검증하며, 한 branch에서 해결됐다는 사실을 다른 branch에 전역 적용하지 않는다.

#### Applicable Change 판정

Change applicability는 target Composite Basis의 Declaration Ref Set에 들어 있는지를 보고 정의하지 않는다.
그렇게 하면 누락된 declaration이 검사 대상에서 사라질 수 있다. 먼저 Post-Freeze Change의 origin과 target
lineage를 판정하고, 그 결과에 맞춰 declaration set의 완전성을 별도로 검증한다.

```text
is_applicable_change(change, target_basis) :=
  change.Origin Composite Basis가 target_basis와 같거나
  target_basis의 declared predecessor/change lineage에 속하거나
  target_basis가 positive overlay compatibility evidence로 그 origin을 명시적으로 계승하고,
  target_basis가 그 Change와 무관한 disjoint branch라는 evidence가 없음

applicable_changes(target_basis) :=
  is_applicable_change(change, target_basis) = true인 모든 Post-Freeze Change
```

이때 검색할 source population과 종료 근거를 명시한다. 관련 Topic의 origin·predecessor/change lineage와
명시적으로 계승한 overlay에 속한 Post-Freeze Change 원문을 열거하고, 확인한 저장 위치/인덱스·범위·누락 가능성을
기록한다. target Declaration Ref Set이나 선택적 Post-Freeze Change Log에 보이는 항목만을 전체 population으로
간주하지 않는다. 완전성을 확인할 수 없으면 applicability는 `INVALID`이며 빈 집합으로 처리하지 않는다.
기존 owning artifact 저장소와 조회 도구를 사용할 수 있고 별도 전역 Change Registry를 새 필수 artifact로
만들 필요는 없다.

- `declared predecessor/change lineage`에는 Working Revision과 downstream artifact의 predecessor link,
  successor Working Revision이 고정한 Post-Freeze Change/Declaration basis 및 stage-relevant Composite Basis
  projection을 사용한다. target 단계에 필요하지 않아 `N/A`인 downstream component 하나만으로 Change를
  non-applicable로 판정하지 않는다.
- positive overlay compatibility는 exact origin component, 유지하는 policy meaning과 적용 범위를 식별해야
  한다. 단순히 successor가 존재하거나 artifact를 보존했다는 사실은 compatibility evidence가 아니다.
- target lineage와 disjoint한 branch의 Change는 applicable하지 않다. ancestry 또는 overlay compatibility를
  확정할 수 없으면 편의상 제외하지 않고 applicability 판정을 `INVALID`로 두며 stage gate를 통과시키지
  않는다.
- Change가 effectively resolved됐더라도 lineage에서 사라지지 않는다. applicable 여부와 unresolved 여부는
  별도 판정이며, applicable한 resolved Change도 active leaf resolution 검증 대상이다.

각 applicable Change에 대해 valid Declaration Supersession chain을 따라 target basis와 양립하는 current
declaration leaf를 정확히 하나 선택한다.

```text
expected_authoritative_declaration_set(target_basis) :=
  { authoritative current declaration leaf(change, target_basis)
    for change in applicable_changes(target_basis) }

target_basis.Authoritative Change Obligation Declaration Ref Set
  == expected_authoritative_declaration_set(target_basis)
```

applicable Change의 declaration이 없거나 leaf가 둘 이상이거나 declaration set에 누락·extra·다른 branch의
Ref가 있으면 Composite Basis와 stage gate는 `INVALID`다. 이 set equality와 applicability evidence는 stage
entry와 exit에서 함께 검증한다.

#### Change Obligation Declaration

Post-Freeze Change, 두 closure와 최초 obligation set은 다음 declaration payload에서 함께 선언한다.

```text
Change Obligation Declaration Canonical Payload
  Change ID
  Post-Freeze Change Record Ref
  Origin Composite Basis
  Policy Impact Closure
  Artifact Invalidation Closure
  Complete Initial Obligation Member Set
  Stage Coverage Evidence
  Invalidated Artifact Disposition Set
```

최초 Resolution Obligation은 독립 Exact-Ref artifact가 아니라 declaration 안의 immutable member다.

```text
Resolution Obligation Member
  Obligation ID
  Owner Stage
  Origin Composite Basis
  Internal Prerequisite Obligation ID Set
  External Prerequisite Obligation Key Set 또는 empty
  Target Basis Requirements/Projection
  Artifact Invalidation Closure Selector
  Required Resolution Artifact Type

Obligation Container := Change Obligation Declaration | Obligation Supersession Record
Obligation Key := (Container Ref, Obligation ID)
```

declaration은 referenced Post-Freeze Change Record, 두 closure, complete initial obligation member set과
invalidation set을 하나의 완전한 결과로 publish한다. declaration Ref가 발행된 뒤 각 최초 obligation은
`(Declaration Ref, Obligation ID)`로 식별된다. 따라서 declaration payload가 아직 존재하지 않는 obligation
Ref를 참조하거나 obligation이 자신을 담은 declaration Ref를 다시 identity payload에 포함하는 순환은 없다.
Origin Composite Basis는 이 Change와 declaration이 생기기 직전의 exact basis이며, 지금 작성 중인
Change Obligation Declaration Ref를 포함하지 않는다.

`Invalidated Artifact Disposition Set`은 artifact Exact Artifact Ref, derived
`stale` disposition, invalidation reason과 Change ID를 고정한다. 이 set에 포함된 artifact는 declaration publish
시점부터 논리적으로 stale이며 Artifact Disposition Registry 갱신 성공 여부와 무관하다.

- initial obligation set은 두 closure가 요구하는 모든 owner stage를 빠짐없이 coverage해야 한다. Stage
  Coverage Evidence가 closure member/edge를 obligation 또는 명시적인 no-repair disposition에 대응시킨다.
- obligation이 실제로 필요 없는 Change도 유효할 수 있지만, empty set과 그 이유를 declaration이 명시적으로
  publish해야 한다. declaration 부재와 유효한 empty declaration은 서로 다르다.
- 같은 container 안의 prerequisite는 Obligation ID로만 참조한다. 다른 container의 predecessor가 꼭 필요한
  successor obligation만 완전한 Obligation Key를 사용한다.
- External Prerequisite Obligation Key는 이미 발행된 predecessor container만 가리킨다. 현재 작성 중인
  container의 member끼리는 반드시 Internal Prerequisite Obligation ID Set을 사용하며 자기 Container Ref를
  payload 안에서 미리 참조하지 않는다.
- external prerequisite도 동일한 Change의 obligation만 참조할 수 있다. 정상 P2G 흐름에 필요하지 않은
  cross-Change obligation dependency는 만들지 않는다.
- declaration 필수 구성이 완전하게 검증되고 current declaration으로 선택되기 전에는 closure의 일부나
  obligation member를 current prerequisite로 사용하지 않는다.

```text
is_valid_authoritative_declaration(change, basis) :=
  같은 (Change ID, Origin Composite Basis)에서 current로 선택된 declaration이고
  Change/closure/initial obligation/invalidation member와 exact reference가 모두 일치하고
  Stage Coverage Evidence가 closure 전체를 증명하며
  prerequisite graph가 strict stage order와 acyclicity를 만족함
```

이 predicate가 false이면 `active_leaves`는 empty가 아니라 `INVALID`다.

#### Resolution Obligation과 prerequisite graph

Change Obligation Declaration은 Artifact Invalidation Closure가 요구하는 수리를 단계별 immutable obligation
member로 분해한다. 미래 artifact의 exact ID는 아직 존재하지 않을 수 있으므로 obligation은 target artifact
ID가 아니라 projection을 소유한다. Change ID와 authoritative declaration은 Obligation Key의 container에서
확인하며 member payload에 중복하지 않는다.

필요한 stage obligation과 resolution artifact는 다음과 같다.

| Owner Stage | Resolution artifact |
| --- | --- |
| `P2G-10` | current successor Working Revision Ref/Decision Revision Ref Set |
| `P2G-20` | current Review Batch Ref/Result-basis Code Review Binding Ref Set |
| `P2G-30` | current successor Final Policy Freeze |
| `P2G-40` | current successor Phase Plan |
| `P2G-50` | current successor Goal Set |
| `P2G-60` | current durable Handoff Artifact |

변경이 영향을 주는 단계의 obligation만 만들고 prerequisite edge로 순서를 표현한다. 예를 들어 material
change가 기존 Goal까지 영향을 주면 `O10 → O20 → O30 → O40 → O50 → O60`의 필요한 부분을 만든다.
Phase-only change는 O40과 실제로 invalidated된 뒤 단계 obligation만 가지며 current Freeze를 다시 만드는 O30을
요구하지 않는다.

stage 순서는 `P2G-10 < P2G-20 < P2G-30 < P2G-40 < P2G-50 < P2G-60`이다. 모든 prerequisite edge
`P → O`는 다음을 만족해야 한다.

```text
owner_stage(P) < owner_stage(O)
```

같은 stage obligation 사이의 prerequisite edge는 허용하지 않는다. 함께 해결해야 하면 하나의 obligation 또는
동일 stage 결과로 합친다. graph는 acyclic이어야 하며 strict stage order와 acyclicity를 모두 검증해야
declaration을 current로 선택할 수 있다. Obligation Supersession이 만드는 successor member set과 새
prerequisite edge에도 같은 규칙을 적용한다.

#### 완전한 단계 결과와 current selection

이 문서에서 `원자적`, `함께 publish` 또는 `complete result`는 저장소 transaction, crash recovery, CAS나
동시성 protocol을 요구하는 말이 아니다. 단계의 필수 결과가 모두 검증되기 전에는 부분 결과를 current
prerequisite로 사용하지 않는다는 의미 경계다.

```text
Complete Stage Result
  Required Result Artifact/Relation Set
  Applicable Resolution Record Set
  Applicable Obligation Supersession Record Set
  Required Cross-reference/Coverage Evidence
```

- candidate record, revision과 evidence는 수행 중 개별적으로 작성할 수 있다.
- P2G-10은 사용자 논의를 durable하게 재개하기 위해 자체 reference가 일치하는 incomplete Working Revision을
  Current Working Checkpoint로 publish·선택할 수 있다. 이는 document head 선택일 뿐 complete-stage result나
  후속 단계 prerequisite의 current selection이 아니다.
- 필수 구성, exact reference, coverage와 해당 단계 exit condition이 모두 검증된 뒤에만 결과를 complete-stage
  result로 판정하고 후속 단계 prerequisite의 current selection에 사용한다.
- 완료 전의 candidate는 보존할 수 있지만 후속 단계의 current prerequisite로 사용하지 않는다.
- complete result를 만들지 못하면 후속 prerequisite의 기존 current selection을 유지한다. P2G-10의
  incomplete checkpoint가 document current head로 선택되는 것과는 별개이며, 이미 authoritative declaration으로
  stale인 predecessor를 current로 복원한다는 뜻은 아니다. 특히 P2G-10 incomplete successor checkpoint가
  current head이면 이전 complete checkpoint를 current prerequisite로 유지하지 않는다.
- P2G-10의 성공 경계는 latest Current Working Checkpoint가 모든 P2G-10 exit condition을 만족해
  `is_p2g10_complete = true`가 되는 시점이다. repair라면 같은 성공 결과에 해당 Resolution Record도 함께
  effective해져야 한다.
- P2G-20은 Review Batch, P2G-30은 Final Freeze와 Membership Set, P2G-40은 Phase Plan과 passed Phase Plan
  Validation Receipt를 각 단계의 complete result 중심 artifact로 사용한다. 별도의 범용 publish envelope나
  저장 성공 관찰 artifact는 만들지 않는다.

동일한 `(Change ID, Origin Composite Basis)`에서는 하나의 declaration만 current로 선택한다. 다른 내용으로
교체해야 하면 기존 declaration을 수정하지 않고 다음 immutable supersession 관계와 완전한 successor
declaration을 publish한다.

```text
Declaration Supersession Record
  Predecessor Change Obligation Declaration Ref
  Successor Change Obligation Declaration Ref
  Change ID
  Origin Composite Basis
  Closure/Stage Coverage Preservation Evidence
```

replacement는 predecessor가 해당 basis의 current declaration이고 successor의 complete initial obligation
member set, strict stage order와 coverage 보존이 검증될 때만 current selection을 바꾼다. supersession chain은
분기하거나 순환할 수 없다. 정상 실행을 위해 저장소 수준 CAS, 경쟁 write 조정이나 recovery 절차를 계약화하지
않는다.

#### Resolution Record와 basis-relative 판정

owner 단계의 complete result는 다음 immutable record로 실제 exact basis를 확정한다.

```text
Resolution Record Canonical Payload
  Obligation Key
  Owner Stage
  Exact Resolved Prerequisite Basis
  Exact Result Composite Basis
  Resolution Artifact Ref
  Resolution Evidence
```

Obligation Key의 Container Ref에서 Change와 authoritative declaration context를 확인한다. Resolution artifact가
current이고, exact prerequisite가 같은 branch의 current ancestry에 있으며,
Target Basis Requirements/Projection과 closure coverage를 만족하고, 이후 change에 의해 다시 invalidated되지
않았을 때만 해당 obligation은 target basis에서 `effectively resolved`다.

owner stage에 applicable obligation이 있으면 result artifact와 필요한 Resolution Record를 같은 complete stage
result로 검증한 뒤 current로 선택한다. result artifact만 current가 되거나 Resolution Record만 effective해지는
부분 성공은 허용하지 않는다.

```text
is_effectively_resolved(obligation_key, target_basis) :=
  valid Resolution Record가 target_basis의 component lineage와 exact하게 양립하고
  active obligation leaf의 closure를 완전히 해결하며
  그 resolution result가 이후 change로 다시 invalidated되지 않음

is_effectively_resolved(change, target_basis) :=
  valid authoritative Change Obligation Declaration이 존재하고
  target_basis에 applicable한 모든 active obligation leaf가 effectively resolved
```

어느 한 branch에 Resolution Record가 존재한다는 사실만으로 다른 target basis의 obligation을 해결된 것으로
판정하지 않는다. declaration이 없거나 invalid이면 빈 obligation set에 대한 vacuous success를
허용하지 않고 Change resolution 판정 자체를 `INVALID`로 반환한다.

#### Obligation Supersession

Phase 구조 변경처럼 기존 downstream closure를 다시 계산해야 하면 기존 obligation의 closure를 수정하지 않고
집합 단위 immutable supersession을 사용한다.

```text
Obligation Supersession Record Canonical Payload
  Change ID
  Authoritative Change Obligation Declaration Ref
  Predecessor Obligation Key Set
  Successor Obligation Member Set
  Closure Coverage Evidence
  Applicable Composite Basis
```

- successor obligation은 record 안의 immutable member이며 record Ref가 발행된 뒤
  `(Obligation Supersession Record Ref, Obligation ID)`로 식별한다.
- predecessor와 successor는 모두 같은 Change에 속해야 하며 cross-Change obligation dependency를 만들지 않는다.
- record 전체의 필수 구성, reference, coverage와 obligation graph의 acyclicity가 검증된 뒤에만 effective하다.
- predecessor closure 전체가 successor obligation set으로 coverage된다는 evidence가 있어야 한다.
- 1:N, N:1과 N:M을 허용하지만 부분 작성된 record나 successor 하나의 존재만으로 predecessor를 inactive로
  만들지 않는다.
- successor obligation set의 모든 prerequisite edge가 strict stage order를 만족해야 한다. 같은 stage edge나
  뒤 stage에서 앞 stage로 향하는 edge가 있으면 record를 effective하게 만들 수 없다.
- 위 조건을 만족하고 Applicable Composite Basis가 target lineage와 양립할 때만 predecessor set을
  `validly superseded`로 판정한다.
- 현재 수리 과정이 downstream obligation의 closure를 바꾸면 새 successor obligation set과 이 record를
  만들며 기존 obligation bytes와 closure member set은 보존한다.

target basis에서 validly superseded된 predecessor를 따라 successor edge를 끝까지 진행했을 때 남는 obligation을
`active obligation leaf`라 한다. Resolution은 obligation을 삭제하지 않고 그 active leaf를 해결한다. Change의
effective status는 이 active leaf set 전체를 기준으로 계산한다.

```text
active_leaves(change, target_basis) :=
  valid authoritative declaration이 없으면 INVALID
  유효한 declaration이면 valid supersession을 적용한 applicable leaf set
```

유효한 declaration이 Complete Initial Obligation Member Set을 명시적으로 empty로 선언하고 Stage Coverage Evidence가
이를 정당화한 경우에만 정상적인 empty leaf set을 반환한다. `validly superseded`는 active leaf set을 계산하는
중간 판정이며 stage gate의 최종 대안 조건이 아니다.

#### 단계 entry와 exit/current

Resolution Obligation은 실행 권한을 새로 만들지 않는다. 필요한 owner 단계가 단일 요청 또는 승인된 연속
진행 범위에 들어 있는지는 Reader Contract로 판단한다. 선택된 P2G-X에는 다음 추가 gate를 적용한다.

```text
P2G-X entry:
  applicable_changes(target_basis) 판정이 INVALID가 아니고
  Authoritative Change Obligation Declaration Ref Set이 expected set과 exact-match하며
  모든 applicable Change에 valid authoritative declaration이 존재하고
  X보다 앞선 owner stage의 모든 applicable active obligation leaf가 effectively resolved

P2G-X exit/current:
  applicable_changes(result_basis) 판정이 INVALID가 아니고
  Authoritative Change Obligation Declaration Ref Set이 expected set과 exact-match하며
  모든 applicable Change에 valid authoritative declaration이 존재하고
  해당 단계의 일반 exit condition을 만족하고
  X가 소유한 모든 applicable active obligation leaf가 effectively resolved
```

현재 단계가 소유한 unresolved obligation은 entry를 막지 않고 repair scope가 된다. 일부만 수리한 결과는
current로 publish할 수 없다. 뒤 단계 obligation만 가진 Change라도 valid declaration 검사는 entry와 exit에서
생략하지 않는다. 뒤 단계 active leaf 자체는 현재 단계 진입을 막지 않는다. 요소 단위 재검증의 안전
판정은 target Working Revision에 applicable하지만 아직 change 전체로서 effectively resolved되지 않은 모든
active Policy Impact Closure의 합집합을 사용한다. 판정 evidence에는 evaluated Change/Obligation/Closure Ref
set과 stable key로 정렬한 canonical union member set을 기록한다.

#### Freeze 이후 공통 successor 규칙

- published Final Policy Freeze 이후 발생한 material/non-material change는 모두 immutable successor Working
  Revision을 사용한다. predecessor Working Revision, 기존 Freeze와 Membership은 수정하지 않는다. 기존
  Freeze의 current planning basis 재사용 여부는 아래 Freeze currentness 규칙으로 별도 판정한다.
- finding이 P2G-10 이외의 단계에서 발견됐거나 현재 요청이 동결 정책 변경을 수행하는 명시적 P2G-10 실행
  요청이 아니라면, 발견 단계는 Post-Freeze Change, 두 closure, complete initial obligation member set과 Invalidated
  Artifact Disposition Set을 완전한 Change Obligation Declaration으로 publish한다. declaration이 current로
  선택된 뒤 derived lifecycle disposition과 필요한 P2G owner를 기록한다. 허용된 non-material 수리는
  계속하고, 새로운 material 결정 또는 범위·권한 문제가 있으면 그 경계에서 멈춘다. declaration의 필수
  구성을 완성하지 못하면 수리를 시작하지 않는다.
- 현재 사용자 요청이 동결된 정책의 exact scope/policy basis 변경을 수행하는 명시적 P2G-10 실행 요청이면,
  같은 요청은 P2G-10 entry preparation의 Change Obligation Declaration publish와 이어지는 successor intake/
  Working Revision 준비 및 P2G-10 본활동을 함께 승인한다. 이는 선택된 P2G-10 내부의 진입 준비이며 단계 간
  자동 권한 확대가 아니다. P2G-20 이후의 수행 여부는 원래 승인된 종료 결과와 실제 prerequisite에 따른다.
- 필요한 최초 P2G 복귀가 Reader Contract의 승인 범위 안에 있으면 해당 단계의 entry preparation으로
  successor Working Revision을 만든다. 범위 밖이면 먼저 이동 권한을 확인한다. successor는 predecessor Working
  Revision Ref, predecessor Final Policy Freeze Ref와 change/closure basis의 ordered exact Ref를 고정한다.
- 이미 successor Working Revision head가 있는 상태에서 policy basis가 더 바뀌면 그 head를 수정하지 않고
  다시 immutable successor Working Revision을 만든다.
- material은 의미 재결정을 위해 affected Policy Element를 `open`으로 재개방한다. 일반 non-material
  source/plan 변경은 element를 그대로 보존한다. 단, 아래 의미 보존 Decision 재분할의 reference-only 전이는
  P2G-10이 별도 계약에 따라 successor `decided` revision을 만드는 예외다. 세 경우 모두 원본은 수정하지 않는다.

#### Freeze 이후 material 변경

Final Policy Freeze가 생성된 뒤 material question이나 결정 변경이 발생하면 다음 순서를 따른다.

1. finding, changed premise와 direct affected Policy Element를 candidate Post-Freeze Change에 기록한다.
2. Policy Element semantic dependency를 따라 Policy Impact Closure를 계산한다.
3. traceability dependency를 따라 별도의 Artifact Invalidation Closure를 계산하고 complete initial
   obligation member set, Stage Coverage Evidence와 Invalidated Artifact Disposition Set을 계산한다.
4. Change, 두 closure, obligation member set과 invalidation set을 하나의 완전한 Change Obligation
   Declaration으로 publish한다. 필수 구성과 coverage가 검증되기 전에는 current declaration으로 선택하거나
   disposition·복귀 작업을 확정하지 않는다.
5. current declaration은 기존 Freeze, Policy Element Revision과 artifact bytes를 수정하지 않는다.
6. member Policy Element, Decision 또는 Acceptance meaning이 무효화됐으므로 declaration의 invalidation set에서
   predecessor Freeze를 즉시 `stale`로 파생한다. 이 판정은 사용자 단계 이동 승인이나 successor Working
   Revision 생성까지 기다리지 않는다.
7. declaration의 Artifact Invalidation Closure에 속한 artifact는 `stale`이며 current basis, Goal authoring
   또는 activation 근거로 사용하는 것을 중단한다.
8. invalidation closure 밖의 predecessor-bound artifact는 `preserved`로 기록한다. 이는 bytes 보존과 잠재적
   재사용만 뜻하며 current pipeline basis가 아님을 명시한다.
9. P2G-10 복귀 필요, declaration Exact Artifact Ref와 두 closure를 기록한다. 승인 범위가 P2G-10을 포함하면
   그 owner에서 `8.6`의 위임 결정 또는 사용자 질문으로 처리한다. 포함하지 않으면 필요한 이동·결정을 함께
   요청한다. 연속 진행 권한만으로 정책 결정 권한을 추정하거나 추천안을 승인으로 간주하지 않는다.
10. P2G-10 실행이 승인 범위에 있으면 current declaration을 entry basis로 사용해 successor Working Revision을
    생성하고 미결 질문을 checkpoint로 보존할 수 있다. 결정이 필요한 요소를 `decided`로 닫는 것은 실제
    사용자 답변 또는 `8.6`의 통상적 정책 위임 승인·조건 충족·exact AI 결정 근거가 있을 때만 가능하다.
    Policy Impact Closure의 element만 successor `open` revision을 만들고 current element map에서 선택하며,
    나머지 element는 predecessor Freeze의 exact member revision을 그대로 선택한다. valid current
    declaration이 없으면 계속하지 않는다.

#### Freeze 이후 non-material 변경

**Decision 재분할의 제한적 예외:** 승인 의미를 보존하면서 Decision current-selection과 element의 Decision Ref를
바꾸는 경우 source/boundary-only 변경이 아니다. 의미가 유지되므로 Policy Impact Closure는 empty일 수 있지만,
P2G-10이 소유한 reference 재구성·coverage 확인과 그 결과에 대한 P2G-20 binding 검토가 필요하다. affected
Decision/Element reference에 의존하는 Freeze와 downstream의 Artifact Invalidation Closure를 계산하고,
complete declaration에 필요한 P2G-10/20/30 및 영향받은 뒤 단계 의무를 배정한다. Reader Contract가 그 수리를
포함하면 P2G-10 “Decision 재분할”의 원자적 current-selection을 수행하고, 포함하지 않으면 필요한 권한을 요청한다. 새로운 의미가 섞이면 이 예외를 쓰지 않고
material 규칙으로 처리한다. 아래의 element 무변경 규칙은 이 예외가 아닌 일반 non-material 변경에 적용한다.

- 정책 의미가 그대로이고 code evidence 또는 구현 경계만 바뀌었다면 Final Policy Freeze, Freeze Membership과
  해당 member Policy Element Revision bytes를 변경하지 않는다. 기존 Freeze의 Code Review Binding Ref Set에도 새 binding을
  추가하지 않는다.
- 해당 non-material Change의 Policy Impact Closure는 empty다. 이는 target basis에서 아직 effectively
  resolved되지 않은 다른 Change들의 Policy Impact Closure가 없다는 뜻이 아니다.
- 변경된 evidence에 의존하는 Code Review, Binding, Source Basis, Completeness Evidence, Work Package, Phase,
  Goal과 Handoff의 Artifact Invalidation Closure, complete initial obligation member set과 Invalidated Artifact
  Disposition Set을 계산해 완전한 Change Obligation Declaration으로 publish한다. 필수 구성과 coverage가
  검증되기 전에는 current declaration으로 선택하지 않는다.
- current declaration의 invalidation이 Final Freeze에 포함된 Binding, Source Basis 또는 Binding
  Completeness Evidence를 침범하면 predecessor Freeze도 즉시 `stale`로 파생된다. Phase/Goal 같은 downstream
  artifact만 침범하면 predecessor Freeze는 current를 유지할 수 있다. 어느 경우에도 Freeze bytes와
  Membership은 변경하지 않는다.
- Artifact Invalidation Closure에서 가장 이른 owner stage를 선택한다. Binding/Source review이면 P2G-20,
  Phase-only이면 P2G-40, Goal-only이면 P2G-50이 될 수 있다. 승인된 `bounded_pipeline` 범위면 추가 단계 이동
  허락 없이 수리·재검증하고, 범위 밖이면 필요한 권한을 요청한다. 공통 successor 규칙을 따라 revision을 만들되
  predecessor Freeze의 exact member revision map을 그대로 상속한다. Policy Element Revision을 새로 만들거나
  재개방하지 않는다.
- 최초 stale artifact를 소유한 P2G 단계의 종료 조건을 fresh evidence로 다시 통과해야 current로 복원할 수
  있다. 단순 재확인이나 기존 결론의 복사는 재검토가 아니다.
- P2G-20 obligation이 있으면 Review Batch는 reviewed revision이 이 successor Working Revision current
  element map의 exact revision인지 확인하고, Result-basis Code Review Binding Ref Set을 publish한다. P2G-30 obligation도
  있으면 권한이 확인된 P2G-30에서 successor Freeze를 만들어야 한다. 연속 진행 범위 안이면 별도 요청은 불필요하다.
- successor Working Revision을 생성했다는 사실만으로 predecessor Freeze disposition을 바꾸지 않는다.

#### Artifact lifecycle disposition과 successor reuse

Artifact lifecycle disposition과 successor reuse status는 원본 artifact bytes와 별개인 두 축이다.
Lifecycle의 authoritative stale 판정은 current Change Obligation Declaration의 Invalidated Artifact
Disposition Set에서 파생한다. 아래 registry와 표는 그 결과를 표시하는 view다.

| Disposition | 의미 |
| --- | --- |
| `current` | 현재 Working/Freeze basis와 일치해 후속 단계의 근거로 사용할 수 있음 |
| `stale` | 전제 변경의 영향을 받아 재검토 전에는 current 근거로 사용할 수 없음 |
| `preserved` | invalidation closure 밖이라 bytes와 결론을 보존했지만 predecessor에만 bind돼 현재 pipeline basis는 아님 |
| `superseded` | 검증된 successor artifact가 current로 선택돼 더 이상 current가 아님 |
| `historical` | 이력·감사 근거로만 보존되며 current 작업 대상으로 선택되지 않음 |

| Successor Reuse Status | 의미 |
| --- | --- |
| `N/A` | successor rebind 상황이 아님 |
| `not_assessed` | successor에서 내용 재사용 가능성을 아직 검토하지 않음 |
| `reusable_unbound` | 내용상 재사용 가능하다는 positive evidence가 있지만 successor basis에 아직 bind되지 않음 |
| `rejected` | successor premise와 맞지 않아 내용 재사용 불가 |
| `rebound_current` | owner 단계가 fresh revalidation을 완료하고 successor Working Revision 또는 Freeze에 다시 bind함 |

- predecessor artifact는 stale이 됐다는 이유로 삭제하거나 내용을 수정하지 않는다.
- applicable Change의 declaration이 missing 또는 invalid이면 관련 origin basis의 currentness를 확정하지
  않고 해당 단계 진행을 막는다. Artifact Disposition Registry에 기존 `current` row가 남아 있어도 이를
  근거로 진행하지 않는다.
- successor가 검증되기 전에는 predecessor를 `superseded`로 표시하지 않는다.
- Artifact Invalidation Closure 밖이라는 사실만으로 `current`가 유지되지 않는다. Stale predecessor Freeze에
  bind된 artifact는 우선 `preserved/not_assessed`다.
- prerequisite basis가 계속 current이고 successor overlay와의 양립이 positive evidence로 확인되면 closure 밖
  artifact는 기존 exact binding으로 `current`를 유지할 수 있다.
- 영향받지 않았다는 positive dependency evidence가 있으면 `preserved/reusable_unbound`로 올릴 수 있지만,
  여전히 P2G exit, Phase/Goal authoring 또는 activation의 current input으로 사용할 수 없다.
- `stale` 또는 `preserved` artifact를 current pipeline basis로 복원하려면 owner 단계의 exit condition을 fresh
  evidence로 다시 통과하고, successor basis에 대한 별도 binding record를 만든 뒤
  `current/rebound_current`로 기록해야 한다.
- 원본 artifact bytes를 수정하는 대신 revalidation/binding record가 predecessor artifact, successor Working
  Revision 또는 Freeze와 evidence를 연결한다.
- durable Handoff Artifact는 exact Result Composite Basis의 Freeze, Phase Plan 또는 Goal Set 중 하나라도 stale,
  superseded 또는 binding mismatch가 되면 즉시 `stale`다. 새 current Handoff는 P2G-60을 다시 통과해야 한다.
- P2G 단계 이동의 권한은 disposition에서 파생하지 않고 Reader Contract의 기존 요청 범위에서 확인한다.
  범위 안의 수리는 계속할 수 있지만 disposition/currentness 검증을 생략할 수 없다.

### 8.5 Dirty Workspace Safety

P2G는 policy/plan artifact를 작성할 수 있지만 production code/data/store 구현을 수정하지 않는 read-only
조사 파이프라인이다.

- source 조사 전에 관련 repository identity와 status/diff를 필요한 범위에서 확인한다.
- 기존 dirty work와 untracked file은 사용자 또는 다른 작업의 변경으로 간주한다. 되돌리기, 삭제, 재포맷,
  stash, reset 또는 현재 P2G 결과로의 귀속을 하지 않는다.
- current behavior claim은 조사한 exact source revision과 relevant dirty state를 함께 식별한다. clean HEAD의
  내용과 dirty working tree actual을 섞어 하나의 current source처럼 표현하지 않는다.
- dirty overlap이 있어도 read-only로 owner, current behavior와 claim boundary를 구분할 수 있으면 limitation과
  attribution을 evidence에 남기고 검토할 수 있다.
- 사용자 변경과 baseline을 구분할 수 없는 overlap이 required claim, code review binding 또는 Phase boundary를
  침범하면 `evidence_gap` 또는 ownership blocker로 중단한다.
- P2G가 만든 임시 조사 산출물만 정리할 수 있다. 기존 logs, generated output, test artifact 또는 unrelated
  file을 조사 편의를 이유로 삭제하지 않는다.
- 사용자 요청의 Topic·종료 결과에 필요한 P2G artifact만 각 owner 단계 범위에서 생성·교정한다. 불변
  artifact는 successor로 처리한다. 다른 Topic·Goal·canonical 문서나 도구 소스를 정합성 명목으로 자동 변경하지 않는다.

#### 변경 영향에 비례한 읽기·진단·검증

필수 근거와 실제 결과는 유지하되, 같은 문서·명령을 수정할 때마다 반복하는 절차는 만들지 않는다.

- 읽기 깊이는 현재 claim과 변경 위험에 맞춘다. 기존에 읽은 source·계약·근거가 같은 범위에서 유효하면
  재사용하고, 관련 내용·환경·의존성·가정이 바뀌거나 근거가 불명확한 부분만 확장한다.
- 일반 조회·오타 수정·지침 편집에서는 그 변경을 확인하는 직접 비교·링크·구조 검증부터 선택한다. 실제
  게임 실행 의미나 loader/schema를 바꾸지 않았다면 무관한 전체 게임 테스트를 자동 요구하지 않는다.
- 정식 P2G 단계의 필수 structure/coverage validator는 생략하지 않는다. 동일 exact 입력·도구·환경에서
  유효한 실행 증거는 재사용할 수 있지만, Plan/Freeze/Goal/Receipt 중 관련 입력이 바뀌면 다시 검증한다.
- 로컬 검사라는 이름만으로 안전하다고 가정하지 않는다. 실제 명령·fixture·접근 경계로 production 데이터,
  운영 credentials·서비스, 외부 비용과 파괴적 side effect가 없음을 확인한다. 확인한 환경이 그대로면 매 호출마다
  같은 확인을 반복하지 않는다. 승인 범위의 안전한 진단·artifact 검증·관련 재검증에는 별도 단계별 허락이 필요 없다.
- P2G의 read-only는 production source/data/store를 고치지 않는다는 뜻이다. 검증기의 지정 output·일회용
  fixture·자신이 만든 cache/artifact는 허용된 작업 경계에서 만들 수 있다. 운영 데이터를 테스트 fixture로 쓰지 않는다.
- 검증 실패가 현재 문서·후보 입력의 오류면 해당 owner에서 고치고 영향받은 검증을 다시 한다. production
  코드 결함이나 validator 소스 변경이 필요하면 evidence와 분리해 남긴다. P2G 진행 권한만으로 구현하지 않는다.
- 근거 재사용은 변경되지 않은 입력·방법·환경·claim에 한한다. 새 source 변경을 과거 테스트로 덮거나 재사용한
  결과를 이번 실행으로 기록하지 않는다. 영향 분리가 어려우면 합리적인 범위로 검증을 넓힌다.
- 전체 검증의 주기는 적용되는 계약과 변경 위험으로 정한다. P2G-50의 명시적 Shared Validation Baseline Reuse
  Contract가 있으면 그것을 따르며, 그 안의 initial/final 전체 검증을 사소한 편집까지 강제하는 일반 규칙으로
  확대하지 않는다. 반대로 이미 요구된 최종 통합 검증을 focused 검사로 대체하지 않는다.

### 8.6 Blocker와 중단 조건

Blocker는 required exit claim을 신뢰성 있게 닫을 수 없는 조건이다. **모든 실패·모르는 점·단계 전환이 사용자
blocker는 아니다.** 승인 범위 안의 안전한 조사·오류 교정·재검증으로 해소할 수 있으면 먼저 그 일을 수행한다.

| 상황 | 처리 |
| --- | --- |
| 정보가 아직 부족하지만 접근 가능한 관련 source가 있음 | 범위를 좁혀 추가 조사. 바로 새 승인을 요청하지 않음 |
| 같은 범위의 문서 오류·누락·참조·Plan 배정 오류 | owner에서 candidate 교정 또는 불변 successor 수리 후 재검증 |
| 승인 의미를 유지하는 내부 구조·검증 방법 선택 | 근거를 기록하고 기술적으로 결정 |
| 새로운 `material_policy`, scope conflict 또는 승인 의미의 충돌로 사용자 의논 필요 | 주 작업은 일반 채팅의 최종 답변으로 질문 후 대기. 실행 중 작업의 중단·계속 여부는 아래 독립성 조건 적용 |
| 필수 source·consumer·도구·권한에 접근할 수 없고 안전한 대체 경로도 없음 | `evidence_gap`/접근 제한을 보고. 무관한 근거로 성공을 대체하지 않음 |
| production mutation·배포·미승인 비용·파괴적 변경이 필요 | P2G 밖의 권한 경계. 별도 요청 없이 수행하지 않음 |
| 사용자 지정 checkpoint·시간/비용/도구 한계 또는 종료 결과 달성 | 결과·미완료·재개 조건을 구분해 종료 |

`ownership conflict`는 조사로 owner를 확인할 수 있는 기술 문제인지, 실제 사용자-visible 정책 충돌인지 먼저
분류한다. 정책 의미는 P2G-10, code/source evidence는 P2G-20, execution grouping/dependency는 P2G-40 owner다.

#### 통상적 정책과 사용자 선택이 필요한 정책

**정책 결정이 필요하다는 사실과 사용자 의논이 필요하다는 사실을 구분한다.** 통상적 정책은 승인된 기능을
완성하기 위해 기존 합의 또는 객체의 소유·정체성·수명에서 기본 동작을 도출할 수 있는 결정이다. 기술적인
선택에만 한정하지 않으며, gameplay나 수명에 관한 `material_policy`도 아래 조건을 만족할 수 있다.

사용자가 이 기준에 따른 통상적 정책 결정을 위임한 범위에서, 다음 다섯 조건을 **모두** 만족하면 AI가
결정·기록하고 계속한다. 지침 파일의 존재나 다른 프로젝트의 관행만으로 위임 또는 기본값을 추정하지 않는다.

| 조건 | 확인할 근거 |
| --- | --- |
| 기존 합의 유지 | 명시된 정책·예외·사용자 의도를 변경하지 않음 |
| 기본 동작의 도출 근거 | 동일한 상황의 기존 정책을 적용하거나, 확인된 소유자·객체 정체성·수명 규칙으로 선택을 설명할 수 있음 |
| 별도 게임 규칙을 추가하지 않음 | 새로운 보상·비용·확률·상한·환불·상속·이전 예외를 만들어야 하는 결정이 아님 |
| 해당 기능 안의 영향 | 다른 객체의 자원이나 독립적으로 보존해야 할 진행·획득 내역을 변경하지 않음 |
| 기존 사용자 데이터 보존 | 기존 저장 데이터의 삭제·일괄 변환·재해석 또는 이미 획득한 권리의 변경을 요구하지 않음 |

소유·정체성·수명 자체가 아직 미정이면 그 전제를 임의로 정해 통상적이라고 판정하지 않는다. 기존 코드의
placeholder나 우연한 동작도 승인된 기본값의 근거가 아니다. 여러 선택이 남아 기존 원칙으로 기본 동작을
도출할 수 없고 사용자가 원하는 게임 경험을 선택해야 한다면 사용자 의논 대상으로 분류한다.

예를 들어 장비 사본에만 귀속된 Run 자원이라는 전제가 승인돼 있다면, 장착 해제·직원 이전·강화로 같은
사본의 정체성이 유지될 때 자원도 유지하고, 판매·분해로 사본이 소멸할 때 그 사본 전용 자원도 소멸시키는
것은 통상적 결정이다. 환불·다른 장비로 상속하는 별도 규칙은 추가하지 않는다. 이는 승인된 게임 동작의
일부이며 개인 저장 데이터 삭제 권한이 아니다. 별도 영속 재산·보상·진행 기록까지 소멸시키는 근거로도 쓰지 않는다.
자원의 장비별/직원별/팀 공유 여부, 판매 시 보상 환산, 사망·후퇴의 손실과 회복 기회 또는 기존 합의의 예외는
기존 결정으로 이미 정해지지 않았다면 사용자 선택이 필요한 예다.

판정과 기록은 기존 owner 문서에서 다음처럼 처리한다. 새 canonical artifact·분류 enum·필수 승인 양식이나
validator 필드를 만들지 않는다.

1. 관련 기존 결정과 코드·문서 사실을 확인한다. 기술적 사실을 아직 모른다는 이유만으로 정책 질문을 하지
   않는다. 필요한 사실을 현재 권한 안에서 조사하되 해소할 수 없는 필수 근거 문제는 `evidence_gap`으로 남긴다.
2. 통상적 정책이면 실제 사용자 위임 위치·범위, 다섯 조건의 충족 근거, 선택한 exact 동작과 영향을 기존
   Finding Triage Log의 disposition/evidence 또는 연결된 설명에 간결히 기록한다. 그 항목을 통상적 정책으로
   판정한 AI 결정을 사용자 직접 답변이나 일반 `technical_decision`으로 바꾸어 기록하지 않는다.
3. 정식 정책 의미·acceptance는 P2G-10의 기존 Decision Revision과 Policy Element Revision에 반영한다.
   기존 `Exact user approval ref`에는 실제 위임 승인 근거를 연결하고, exact AI 결정과 위임 근거의 관계를
   위 설명에서 추적한다. 사용자 직접 결정과 위임 결정은 구분하며, 없는 사용자 답변을 만들지 않는다.
   사용자 질문이 필요하지 않았으면 Open Policy Question을 형식적으로 만들지 않고 해당 Ref Set은 empty로 둔다.
4. 통상적 정책이어도 다른 단계가 P2G-10의 결정을 대신하지 않는다. material finding이 나온 검토 batch는
   성공으로 닫지 않고 원래 owner로 반환한다. 동결 후라면 `8.4`의 Change·closure·immutable successor·
   후속 검토·동결·인계 절차를 지킨다. 현재 실행 범위가 필요한 owner 단계를 포함하면 별도 사용자 질문 없이
   진행하고, 포함하지 않으면 필요한 범위·권한만 요청한다. 통상적 판정 자체로 단계 실행 권한이 생기지는 않는다.
5. 사용자 선택이 필요한 정책은 이유·선택지·영향을 질문하고 아래 대기 규칙을 적용한다. 무응답·추천안·
   기본 선택을 승인으로 처리하지 않는다. 사용자 의논 필요가 확정되기 전 사실 조사와, 확정 후의 실행 중단을
   구별한다. 필요한 질문을 한 뒤 다른 정책을 조사하며 주 작업을 계속하지 않는다.

#### 정책 의논 시 주 작업 대기와 실행 중인 독립 작업

위 기준으로 사용자와 의논이 필요하다고 판정한 정책이 발생하면 **주 작업 담당자는 진행을 임시중단하고, 일반 채팅의 최종 답변으로
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

`inaccessible`·`unknown`을 `N/A`, safe, compatible 또는 pass로 바꾸지 않는다. 이미 이용 가능한 사실·승인으로
답할 수 있는 질문은 재질문하지 않고 근거를 연결한다. 새 입력이 필요한 독립 질문은 가능한 한 함께 정리하여
묻되, 논의 중 선택되지 않은 답을 미리 채우지 않는다.

실패가 남았다는 이유만으로 종료하지 않지만, 새로운 근거 없이 동일 작업을 반복하는 무한 재시도도 하지 않는다.
시도한 방법과 결과를 보고 합리적인 대체 경로가 소진됐거나 요청의 한계에 도달하면 마지막 유효 상태를 보존한다.

중단 보고에는 현재 단계·승인된 종료 결과, 실제 원인, affected claim/artifact, 확인한 근거·한계, 필요한 결정/
권한/자료와 재개 지점을 적는다. `single_stage`에서 다른 단계가 필요하면 범위 밖임을 알린다. `bounded_pipeline`
안의 수리면 이 중단 보고를 단계 이동 승인 요구로 대신하지 않고 진행한다.

정책 답변·자료 제공 후에는 Reader Contract에 따라 기존 요청을 재개할 수 있다. 새 Topic, 범위 확대 또는 GEX
실행은 별도 권한이다. 세션이 끊겼거나 실행 환경이 바뀌면 범위·basis·관련 근거의 유효성을 확인한 뒤 재개한다.

### 8.7 문서 Revision과 Freeze

#### Revision 종류

| Revision | 의미 |
| --- | --- |
| Working Revision | 정책 basis와 predecessor를 고정한 immutable snapshot |
| Policy Element Revision | 한 Policy Element의 statement, decision, Observable Policy Acceptance, intrinsic status와 predecessor를 고정한 immutable revision |
| Final Policy Freeze | exact Working Revision Ref, code-reviewed member Policy Element Revision Ref Set, Current Code Review Binding Ref Set과 Reviewed Source Basis Ref를 complete result로 고정한 immutable artifact |

Working Policy Document는 mutable view이고 current Working Revision head를 선택한다. Working Revision 자체는
producer-issued Working Revision Ref를 가지며 생성 후 내용을 바꾸지 않는다. optional Content Digest는 같은
구현 안의 무결성 확인에만 사용할 수 있다. policy basis가 바뀔 때마다 기존 revision을 predecessor로
가리키는 successor Working Revision을 만든다.

Working Revision의 canonical policy-basis payload는 다음으로 한정한다.

```text
Predecessor Working Revision Ref 또는 N/A
Predecessor Final Policy Freeze Ref 또는 N/A
Approved metadata와 scope
Policy Intake Snapshot Ref
Current Policy Element Revision map
Decision Revision Ref Set
Decision Repartition Member Set — 이 transition에 재분할이 없으면 생략
Open Policy Question Ref Set
Post-Freeze Change/Closure와 authoritative Declaration reference basis
```

producer는 위 payload를 고정한 Working Revision Ref를 발행한다. 후속 consumer는 이를 재계산하지 않고
exact-match한다.

`Approved metadata와 scope`에는 stable Working Policy Document ID/identity, Policy Topic, Requested Result,
target repository identity, exact
User-Approved in-scope/out-of-scope set과 approval reference를 포함한다. AI Proposed Scope와 pending scope item은
successful Working Revision에 포함하지 않는다. scope 승인이 없으면 revision을 publish하지 않는다.

`Policy Intake Snapshot Ref`는 P2G-00이 initial로, 또는 P2G-10이 user-provided raw policy input 변경 시
successor로 고정한 raw policy draft content 또는 exact content reference, user-provided source locator,
user-stated constraint와 capture limitation의 immutable set을 가리킨다. snapshot의 stable metadata는 Working
Policy Document ID, Snapshot Kind `initial|successor`와 Predecessor Policy Intake Snapshot Ref 또는 N/A를
포함한다. canonical row는 `(Intake Item ID, Intake Kind, Exact Content 또는 Content Reference, User-Provided
Locator, User-Stated Constraint, Capture Limitation)`이며 Intake Item ID 순으로 정렬한다.

producer는 stable document/intake metadata와 Intake Item ID로 구분된 immutable row list를 고정한 Policy
Intake Snapshot Ref를 발행한다. P2G-00과 P2G-10의 snapshot 생성 활동은 snapshot 내용을 정책 요소로
해석하지 않는다. 후속
user-provided raw input basis 자체가 바뀌면 P2G-10이 원본 snapshot을 수정하지 않고 predecessor를 가리키는
새 immutable successor snapshot을 만들며, 이를 current policy basis로 선택하려면 successor Working Revision이
필요하다. scope만 바뀌고 raw input이 동일하면 새 snapshot을 만들지 않는다. P2G-10의 candidate disposition,
P2G-20의 source plan/observation 또는 derived evidence 변경만으로 snapshot을 바꾸지 않는다.

Policy Element Candidate/Disposition Record, P2G-20 Review Scope/Source Requirement Plan, Source
Observation/Evidence Registry, Candidate Review Envelope, current Review Batch, Code Review artifact, Code Review Binding,
optional Binding Inclusion Index/View, Change Obligation Declaration, embedded Resolution Obligation member,
Resolution Record, Declaration/Obligation Supersession Record, Freeze Membership
materialized view, Phase Plan, Phase Plan Validation Receipt, Goal Set과 Handoff Artifact는 Derived Artifact
Registry에 둔다. 이 registry의
내용은 Working Revision canonical payload에 포함되지 않는다. Derived artifact는 필요하면 Working Revision
Ref를 참조하며, selected Policy Element Revision이 가진 immutable evidence reference는 그 element
revision identity의 일부로만 취급한다. Derived artifact 생성이나 registry 표시 갱신 때문에 Working Revision을
다시 만들지 않으므로 Binding과 Working Revision 사이의 자기참조가 발생하지 않는다.

AI Proposed/Pending Scope는 승인 전 협의 상태이며 successful Working Policy Document artifact가 아니다. P2G
Stage Result Report도 기본적으로 외부 사용자 응답이다. 별도 durable audit artifact로 유지하기로 명시적으로
결정한 경우에만 derived registry에 stable identity와 lifecycle을 부여한다.

Policy Element Revision의 정책 payload는 최소한 다음 의미를 고정한다.

```text
Stable Element ID
Predecessor Policy Element Revision Ref 또는 N/A
Policy statement
Decision Revision Ref 또는 N/A
Observable Policy Acceptance 또는 N/A(open에만 허용)
Intrinsic status: open | decided | code_reviewed
```

Policy Element의 statement, decision, Observable Policy Acceptance 또는 intrinsic status가 바뀌면 기존
revision을 수정하지 않고 successor revision을 만든다. `decided|code_reviewed`에는 비어 있지 않은 Observable
Policy Acceptance가 필요하다. P2G-20이 `decided`를 `code_reviewed` successor로 만들 때 statement, Decision
Revision Ref와 Observable Policy Acceptance의 정책 의미를 exact하게 보존한다. 이를 바꿔야 하면 P2G-20
결과를 publish하지 않고 `material_policy`로 P2G-10에 반환한다.

successor Working Revision의 current element map이 요소별 current Policy Element Revision을 선택한다. 질문을
닫은 P2G-10 Result Working Revision은 Initial Decision이면 새 Ref를 current set에 추가한다. 일반 Successor는
transition 직전 Input Working Checkpoint의 current predecessor를 제외하고 successor를 함께 선택한다. 영향
없는 Decision은 보존하고 해결된 Question은 current set에서 제외한다.

Decision 재분할은 P2G-10의 “Decision 재분할” 절이 소유한다. Result Working Revision은 해당 transition의
`Decision Repartition Member Set`, source 제거·result 전체 추가와 affected element의 Decision 재결속을 함께
고정한다. 새 result는 별도 lineage의 initial Decision이며 source는 일반 predecessor가 아니다. 같은 P2G-10에서
후속 의미 변경을 수행하면 새 checkpoint를 Input으로 일반 successor를 만든다.

current에서 선택한 active lineage에만 정확히 하나의 leaf를 요구한다. 재분할 source lineage는 이 ancestry에서
current member가 0개인 history로 남으며 다시 추가할 수 없다. 의미 보존·coverage·참조 검증 없이 current
Decision을 삭제하거나 source와 result를 함께 선택하지 않는다. 과거 artifact와 Freeze Membership은 보존한다.

Final Policy Freeze는 최소한 exact Working Revision Ref, current Review Batch Ref,
ordered exact `code_reviewed` Policy Element Revision Ref Set, Decision Revision Ref Set, Observable Policy Acceptance Set, Current Code Review
Binding Ref Set, Reviewed Source Basis Ref, Binding Completeness Evidence, applicable Change/Obligation Basis,
resolved prerequisite evidence와 known unknown을 payload로 고정한다. producer는 필수 구성을 모두 검증한 뒤
Final Policy Freeze Ref를 발행한다.

`Observable Policy Acceptance Set`은 P2G-30이 새로 작성하는 정책 artifact가 아니라 ordered member Policy
Element Revision Ref Set에서 계산한 다음 projection이다.

```text
Observable Policy Acceptance Set :=
  ordered projection(
    member Policy Element Revision Ref
      -> member revision의 Observable Policy Acceptance
  )
```

Acceptance의 authoritative owner는 member Policy Element Revision이다. P2G-30은 이 projection이 모든 exact
member revision과 일치하는지만 검증하고 변경하지 않는다. mismatch가 있으면 candidate Final Freeze는
invalid이며 별도 Acceptance artifact나 Content Digest를 만들지 않는다.

#### Freeze Membership

Freeze Membership은 다음 immutable 다대다 관계다.

```text
Freeze Membership := (Final Policy Freeze Ref, Policy Element Revision Ref)
```

P2G-30 검증 중인 candidate는 Final Policy Freeze가 아니며 membership을 만들지 않는다. Final Freeze payload,
ordered member revision set과 applicable P2G-30 Resolution Record Ref Set의 필수 구성과 상호 참조를 모두
검증한 complete P2G-30 result가 publish될 때만 각
`(Final Policy Freeze Ref, Policy Element Revision Ref)` membership이 성립한다. 그 전에는 Freeze,
Membership과 Resolution Record가 모두 non-current candidate다.

```text
is_frozen(revision) :=
  exists published Final Policy Freeze F:
    (F.Ref, revision.Ref) ∈ Freeze Membership
```

`is_frozen`은 revision의 intrinsic status가 아니라 파생 판정이다. 하나의 unchanged revision은 여러 Final
Policy Freeze에 포함될 수 있다. published membership은 삭제하거나 다른 revision으로 바꾸지 않는다. Final
Policy Freeze가 current planning basis에서 `stale`, `superseded` 또는 `historical`이 되어도 membership과
역사적 `is_frozen` 판정은 유지한다.

#### Code Review Binding과 Freeze 편입

Code Review Binding의 canonical payload는 다음 입력만 가진다.

```text
Working Revision Ref
Policy Element Revision Ref
Code Review Artifact Ref
P2G-20 Review Scope/Source Requirement Plan Ref
Reviewed Source Basis Ref
Review Evidence Ref set
Covered Required Claim/Boundary set
Evaluated applicable active Change/Obligation/Closure Ref set and canonical union member set when applicable
```

producer는 위 payload를 고정한 Code Review Binding Ref를 발행한다. Code Review Binding은 Final Policy Freeze
Ref를 payload에 포함하지 않는다. Freeze-eligible
binding은 exact target Working Revision Ref와 그 revision의 current element map에 있는 exact
`code_reviewed` Policy Element Revision을 가리킨다. Final Policy Freeze가 ordered Current Code Review Binding
Set을 포함한다. P2G-30은 `code_reviewed` revision을 별도 `frozen` revision으로 바꾸거나 binding을 새 identity에
재결속하지 않는다.

Binding Completeness는 element마다 binding 하나가 있다는 뜻이 아니다. 다음 predicate를 만족해야 한다.

```text
every Final Policy Freeze candidate member Policy Element Revision의
every required claim and boundary가
Current Code Review Binding Ref Set 중 하나 이상의 current binding으로 coverage되고,
각 binding은 exact target Working Revision Ref,
exact candidate member revision, published P2G-20 Review Scope/Source Requirement Plan과 current source basis를
가리킨다.
```

하나의 element가 여러 repository, source 또는 consumer를 가지면 binding도 여러 개일 수 있다. Coverage가
중복되거나 분할되면 claim/boundary별 owner binding과 completeness evidence가 이를 명시해야 한다.

Final Policy Freeze의 Current Code Review Binding Ref Set이 binding inclusion의 authoritative relation을
소유한다. 필요하면 다음 optional derived reverse-index row를 materialize할 수 있다.

```text
Binding Inclusion Index/View Row
Code Review Binding Ref
Final Policy Freeze Ref
Inclusion observation/evidence
```

이 index/view는 binding canonical payload와 Final Freeze content 밖에 있으며, 없어도 Final Freeze의 included
binding relation과 completeness는 그대로 판정할 수 있다.
Freeze 이후 revalidation은 새 binding을 만들지만 기존 Freeze와 Code Review Binding Ref Set을 수정하지 않는다. 그 binding이
P2G-20 Review Batch의 Result-basis Code Review Binding Ref Set으로 published되고 currentness와 completeness 검증을 통과하면
후속 P2G-30이 만드는 successor Freeze의 Code Review Binding Ref Set에 포함할 수 있다.

#### Successor Working Revision

published Freeze 이후 material 또는 non-material change가 발견돼도 Final Policy Freeze, Freeze Membership과
member Policy Element Revision을 수정하지 않는다.

1. finding이 발견된 현재 P2G 단계에서 완전한 Change Obligation Declaration을 publish하고 current로 선택한 뒤
   그 Invalidated Artifact Disposition Set에서 lifecycle을 파생한다. valid current declaration을 만들지
   못하면 수리 단계로 진행하지 않는다.
2. current declaration의 complete obligation member set으로 최초 owner를 정하고 Reader Contract의 권한을
   확인한다. 허용된 non-material 수리와 `8.6`의 위임 결정은 해당 owner에서 계속한다. 사용자 선택이 필요한
   정책·범위·권한에서는 멈춘다.
3. owner 단계의 entry preparation으로 predecessor Working Revision과 predecessor Freeze를 가리키는
   immutable successor Working Revision을 생성한다. material 의미의 결정 전에는 이를 complete로 취급하지 않는다.
4. material이면 Policy Impact Closure의 요소에만 predecessor member revision을 가리키는 successor Policy
   Element Revision을 만들고 `open`으로 시작한다.
5. 일반 non-material이면 모든 predecessor member revision을 successor current element map에 exact하게 상속하고
   element를 재개방하지 않는다. Decision 재분할에 필요한 reference-only successor는 `8.4`의 제한적 예외와
   P2G-10 계약만 허용하며, 나머지 element와 과거 Freeze Membership은 그대로 보존한다.
6. material change에서도 Policy Impact Closure 밖의 요소는 predecessor Freeze의 exact member revision을
   successor current element map에 그대로 선택한다. 같은 Phase/Goal을 공유한다는 이유만으로 복제하거나
   다시 결정하지 않는다.
7. predecessor-bound artifact는 invalidation closure 안이면 `stale`다. closure 밖 artifact는 prerequisite
   basis가 계속 current이고 successor overlay와 양립한다는 positive evidence가 있으면 `current`를 유지할 수
   있다. prerequisite Freeze가 stale이면 `preserved`이며, successor Working Revision 생성만으로 disposition을
   일괄 변경하지 않는다.
8. `code_reviewed_artifact_revalidation`은 reviewed revision이 planned Result Working Revision의 current
   element map과 exact match일 때만 허용한다. 이미 더 새로운 successor revision이 current인 과거 member
   revision은 재검증 대상으로 선택할 수 없다.
9. successor는 필요한 P2G 단계를 각각 검증해 통과한다. 단계 실행 권한은 단일 요청 또는 기존 연속 진행
   범위에서 확인한다. P2G-30 obligation이 있을 때만 새 Freeze를 만들며, downstream-only repair는 current
   predecessor Freeze와 새 Phase/Goal/Handoff artifact를 사용할 수 있다. 요청 끝점 밖의 결과를 자동 생성하지 않는다.

Successor Working Revision 생성 자체는 predecessor Freeze disposition을 바꾸지 않는다. member Policy
Element/Decision/Acceptance meaning 또는 included Binding/Source Basis/Completeness가 invalidated됐으면 발견
즉시 `stale`이고, Phase/Goal-only invalidation이면 `current`를 유지할 수 있다. Successor Final Policy Freeze가
P2G-30을 통과해 current로 선택된 뒤에만 별도 disposition record로
`predecessor -> superseded_by:successor` 관계를 기록할 수 있다. 원본 predecessor bytes, Membership과 역사적
결정은 그대로 보존한다.

#### Downstream artifact binding

Code Review와 binding은 자신이 근거로 삼은 exact Working Revision Ref를 명시한다. Work Package,
Phase Plan, Goal과 Handoff는 owner 단계가 요구하는 exact Composite Basis와 prerequisite artifact Exact Artifact Ref를
명시한다. Policy Impact Closure의 element가 successor revision에서 재개방되면 Artifact Invalidation Closure
안의 artifact는 stale이며 다음 용도로 사용할 수 없다.

- P2G 단계 exit 근거
- current Phase Plan 또는 Goal authoring basis
- GEX-00 실행 인수 또는 GEX-10 구현 요청의 current upstream basis

Artifact Invalidation Closure 밖의 predecessor artifact도 predecessor Freeze가 stale이면 `preserved`일 뿐이다.
내용상 재사용 가능성을 확인하면 `reusable_unbound`로 기록할 수 있지만 P2G 단계 입력으로 실제 사용할 수는
없다. 해당 artifact owner 단계에서 영향 없음 또는 새 정책과의 일치를 재검증하고 successor Working Revision
Ref에 binding record를 만든 뒤에만 `current/rebound_current`가 된다. P2G-30 이후 단계에서
사용하려면 생성된 successor Final Policy Freeze Ref와의 binding도 exact해야 한다.

이 보존·invalidation 규칙은 P2G artifact lifecycle만 소유한다. 실행 중 accepted Handoff와 완료된 predecessor
Phase evidence의 관계는 Goal Execution Pipeline section 6.1이 소유하며, P2G가 별도 실행 상태 기계를 만들지
않는다.

<!-- P2G-DESIGN-HANDOFF START -->
### 8.8 코드 기반 설계 인계와 구현자 재량

이 section은 기존 Code Review Artifact, Phase Plan과 Goal member에 담을 **설계 내용의 깊이**를 정의한다.
새 P2G 단계·독립 Exact-Ref artifact·GEX Gate를 만들지 않는다. 상세 설계의 실제 작성·코드 수정·시험 실행은
후속 실행 파이프라인의 권한 안에서 수행한다. P2G는 그 작업을 인계할 뿐 실행하지 않는다.

#### 세 수준의 구분

| 수준 | 의미 | owner와 변경 기준 |
| --- | --- | --- |
| 승인 정책 | 사용자·consumer가 의존할 의미와 Observable Policy Acceptance | P2G-10의 Decision/Element가 소유. 구현 편의로 변경하지 않음 |
| 권장 구조 | 승인 의미를 실현할 책임·상태·인터페이스·컴파일/실행 방식과 선택 이유 | P2G-20의 Proposed Implementation/Technical Decision. 사실 근거와 대안을 함께 기록 |
| 코드 기반 상세 설계 | 실행 시점 코드와 선행 구현에 맞춘 정확한 타입·함수·상태 전이·검증·전환 | 구현자가 선택된 Goal scope에서 조사·작성. 공유 계약·정책 변경은 해당 owner로 반환 |

사용자가 권장안을 읽고 동의했다는 사실만으로 모든 내부 표현을 gameplay 정책으로 고정하지 않는다.
반대로 권장 구조라는 이름을 붙여 의미상 필수인 상태 수명·취소 결과·데이터 손실 규칙을 재량으로 바꾸지도
않는다. 분류와 결정 권한은 `8.1`이 소유한다.

#### P2G-20 — code-informed 구조 제안의 최소 깊이

`Proposed Implementation`은 목적의 재서술만으로 채우지 않는다. 해당 변경에 적용되는 항목을 다음처럼
설명하고, 적용되지 않는 항목은 짧은 이유와 함께 제외할 수 있다.

| 항목 | 설명할 내용 |
| --- | --- |
| 책임·의존 | authoritative owner, producer/consumer, 공통 계층과 도메인 경계 |
| 데이터와 상태 | 작성 정의·컴파일 정의·실행 상태의 구분, identity와 owner/lifetime |
| 요청과 결과 | 입력 자격, 식별·상관관계, 접수·진행·완료·실패·취소 결과의 의미 |
| 순서와 원자성 | 읽는 snapshot, 판정·변경·후속 사건의 순서, commit/rollback과 중간 상태 |
| 기존 코드 전환 | 재사용·확장·교체·삭제할 경계와 실제 코드 근거, coexistence/cutover 필요 여부 |
| 대안과 제약 | 채택 이유, 주요 대안의 비용·위험, 적용하면 안 되는 단순화 |
| 검증 연결 | 최소 정상·거부·경계 예제, 실제 validation seam, 기대 상태/관찰 결과 |
| 남은 상세 선택 | 아직 결정하지 않은 내부 선택, 필요한 추가 조사, 구현 owner와 완료 전 조건 |

모든 파일·함수·코드 행을 먼저 고정하거나 모든 구조를 prototype으로 구현할 필요는 없다. 다만 무엇을 나중에
정해도 current policy closure, 검토 claim과 다음 Phase 경계가 유지되는지 설명해야 한다. owner·consumer·수명·
실행 결과처럼 Phase 분해 자체를 좌우하는 선택을 “구현 때 결정”으로 미루고 검토를 닫지 않는다.

설계 예시와 의사코드는 `proposal`로 표시한다. 존재하지 않는 API·RON 문법을 현재 지원 기능처럼 쓰지 않는다.
공통 내용은 Code Review Artifact에서 한 번 소유하고 후속 문서는 exact artifact/member/section locator를
가리킨다. 필요한 경우 사람용 요약을 두되 source와 별개의 가변 계약으로 취급하지 않는다. 의도만 같은 로드맵 문구를 code evidence로 사용하지 않는다.

#### P2G-40 — 공유 계약과 유효한 중간 상태

Work Package 사이의 연결에는 해당되는 경우 다음을 existing boundary/prerequisite/validation 항목에 포함한다.

- 계약의 의미 owner와 producer/consumer, 계약의 source artifact/section.
- 확정해야 하는 입력·결과·상태·취소 의미와 내부 구현자가 선택할 수 있는 범위.
- 계약을 구현하는 Final Execution Node, 그 구현을 검증하는 node와 소비자 Phase.
- 현재 존재하는 코드 계약인지, 검토된 설계안인지, 선행 Phase가 구현할 결과인지의 구분.
- 소비자 구현을 시작할 수 있는 조건과, 선행 Phase가 멈춰도 유효한 중간 상태라는 근거.

별도 Contract Registry나 Design Phase를 항상 요구하지 않는다. 기존 문서 안의 local member ID·section으로
충분하면 재사용한다. 이미 필수 API·상태·저장 전환이 함께 있어야만 유효하다면 서로 독립 Phase라고 표시하지
않고 `7/P2G-40`의 stable seam 또는 Atomic Group 기준으로 재분할한다.

#### P2G-50 — 기존 Goal 필드에 상세 설계 인계를 배치

| 기존 Goal member 필드 | 인계할 설계 내용 |
| --- | --- |
| Reviewed Current Behavior / Implementation Delta | 조사 기준 source, 실제 현재 경계와 변경 목표, 추가 확인이 필요한 가정 |
| Goal-Local Source of Truth / Owning Layer; Goal-Local Invariant Set / Source Refs | 책임과 반드시 보존할 의미, 공유 계약의 exact 근거 |
| Code/Data/Schema/Store/Transport/Consumer Boundary | 권장 구조, 재사용·교체 지점, 관련 호출·상태·저장 경로를 조사할 질문 |
| Goal-Local Dependencies / Execution Prerequisites | 아직 없는 선행 구현, 필요한 상세 설계와 구현 전 준비 점검 조건 |
| Validation Plan / Completion Matrix | 설계를 시험할 입력·예상 순서·결과·실패 경계와 실제 검증 수단 |
| Goal-Local Known Unknown / Blocker / Stop Condition | 미정 정책·근거 부족·설계 변경 때의 중단 및 반환 경계 |
| Applicable Risk / Reviewer Requirement and Evidence | 공유 계약 변경·원자적 전환 등 위험에 비례한 설계 검토 요구 |

인계할 구현자 작업 흐름은 다음과 같다. 이 표기는 기존 GEX의 실행 기록 안에 구현할 활동이지 새 전역
상태 기계·별도 승인 단계 또는 이 문서가 발행하는 Gate가 아니다.

```text
현재 실행 기준과 선행 구현 확인
  → 관련 producer부터 state/result/persistence/consumer/test까지 코드 조사
  → 정확한 상세 설계
  → 설계와 공유 계약의 준비 점검
  → 승인 범위의 구현
  → 실제 검증·필요한 설계 수정
  → 완료 근거와 독립 검토
```

상세 설계에는 applicable한 정확한 타입·입출력·상태 key·수명·처리 순서·오류·취소·전환·변경 symbol·시험 기대값이
있어야 한다. 단순 helper 변경에는 간단한 설계 근거로 충분하며, 모든 Goal에 거대한 설계 문서나 추가 reviewer를
강제하지 않는다. 파일명보다 현재 GEX 문서의 owner와 evidence가 중요하다.

준비 점검에서는 producer/consumer 일치, 다른 상태에 대한 부작용, 취소·늦은 결과·재시도, 데이터/저장 전환,
사용 가능한 선행 계약, 미정 정책과 테스트로 판정 가능한 완료 기준을 확인한다. 실제 점검자·시점·승인은
현재 GEX 권한 계약을 따른다. 준비 점검을 새 사용자 승인 요청으로 자동 변환하지 않는다. 명시적으로 요구된
독립 검토·human checkpoint는 유지하고, 문서 작성 완료나 점검 계획을 실제 검토로 보고하지 않는다.

Goal의 기존 완료·검증·중단 필드에는 **첫 구현 이후의 실행·관찰·수정·관련 재검증까지** 필요한 활동을
포함한다. 구현 요청이 해당 범위를 승인했고 현재 GEX도 허용하면 첫 패치나 첫 테스트 실패만으로 멈추지 않고
완료 조건까지 반복한다. 이 문장은 P2G가 구현 권한을 주거나 현행 GEX의 필수 경계를 덮어쓰는 규칙이 아니다.
로컬 fixture의 격리·명령 side effect·예산을 실제로 확인한 실행만 승인된 로컬 작업으로 취급한다.

#### 명시적 단계검토 요청의 인계

단계검토는 사용자가 “단계검토 과정을 포함해줘”처럼 실제로 요청했을 때만 설계한다. 단순 언급·예시·모델 선택을
opt-in으로 해석하지 않는다. 기본은 선택 Goal의 내부 구현·자체 검증을 마친 뒤 최종 감사 인계에서 종료하는
GEX 흐름이며, 모든 WP가 독립적으로 컴파일된다고 가정하지 않는다. 실행 원본은 현재 배치의
`core_enhanced/docs/goal_execution_pipeline(1).md` Reader Contract와 §6.4다.

- P2G-20은 실제 코드 의존성·공유 타입·schema/store 전환을 근거로 빌드 가능한 중간 경계 후보와 검증 방법을 제안한다.
- P2G-40은 후보의 포함 작업·선행/남은 의존과 유효한 중간 상태를 기존 boundary/prerequisite/validation에 연결한다.
  WP·Atomic Group·Phase를 검토 횟수에 맞추어 임의 분할하지 않는다. 안전한 중간 경계가 없으면 최소 결합 범위와
  이유를 명시한다. Goal 전체가 최소 단위일 수도 있으며 이 경우 중간 중단 대신 최종 구현 인계만 가능함을 알린다.
- P2G-50은 기존 Stop Condition·Reviewer Requirement·Validation Plan 또는 그 설계 locator에 사용자 요청,
  적용 범위, checkpoint 후보·빌드/시험·전후 고정·검토 후 다음 구현 요청 대기를 연결한다. 새 필수 Goal 필드를 만들지 않는다.
- P2G-60은 Remaining Summary/인계 기록에 선택 여부·근거와 해당 설계 locator를 연결한다. 계획 단계에서는 후보와
  실행 시 확인할 조건을 인계할 뿐 실제 빌드 통과·구현 전/후 snapshot·중간 검토 PASS를 미리 발행하지 않는다.

계획·검토·상세 구현 계획의 인계 요건을 마치면 P2G-60에서 최종 응답으로 종료하고 별도 구현 요청을 기다린다.
사용자가 계획 단계에서 고성능 모델을 쓰고 구현 단계에서 다른 모델을 쓰는 선택을 지원하지만 실제 모델 전환은 수행하지 않는다.
뒤늦은 단계검토 요청은 GEX 실행 PLAN에 남은 작업의 기술 계획으로 반영할 수 있다. 정식 정책·Phase/Goal 경계를
바꾸지 않는 한 frozen P2G를 재작성하지 않는다. 해당 경계가 달라질 때만 실제 owner와 successor 절차를 따른다.
부분 검토는 Goal 완료 감사가 아니며, 검토 PASS·모델 교체만으로 다음 구현을 시작하지 않는다.

#### 설계 변경 판단

- 승인 의미·공유 계약을 유지하는 내부 변경은 근거와 검증을 남기는 `technical_decision`이다. 타입·함수·파일
  이름이나 동등한 자료구조 변경 자체로 P2G 전체를 다시 수행하지 않는다.
- shared interface의 consumer-visible 의미, 상태 소유·수명, 실패·취소·전환 의미가 달라지면 `8.1`로
  classification하고, `8.4`에서 실제로 영향받는 가장 이른 owner·closure를 판정한다.
- 코드 조사에서 reviewed premise가 틀렸거나 Phase 분할이 유효하지 않음이 드러나면 `implementation_finding`
  또는 `evidence_gap`일 수 있다. 이를 모든 gameplay의 재승인으로 확대하거나 구현자가 조용히 우회하지 않는다.
- accepted Goal의 delta와 완료된 predecessor가 만든 예상 변화는 정상 실행이다. `8.4`의 expected realization
  규칙을 유지한다. 다만 실제 변경이 예상 결과·불변식과 맞는지는 실행 evidence로 확인한다.

예: 비동기 행동을 호출하는 작업에서 “실행 요청 성공”만 적지 않는다. 요청 접수와 발사·흐름 완료·효과 해결을
구분하고, 실행 handle, 상태 종료 시 취소 범위, 방출된 자식의 존속, 늦은 결과 무효화를 설계 인계에 넣는다.
이 선택이 기존 승인 정책 안의 표현이면 기술 설계로, 기존 플레이 결과를 바꾸면 해당 정책 질문으로 처리한다.

#### 도구 검사와 의미 검토

이 개정은 Goal Quality Contract `2`의 기존 top-level field, 10개 Validation Dimension과 CLI flag를 유지한다.
위 내용은 existing field의 실질적인 작성·인계 요건이다. 현재 구조 검증기의 검사가 이 의미를 자동으로
판정한다고 주장하지 않는다. structure passed와 별도로 설계 내용의 의미 검토 근거를 stage result에 남긴다.

이번 Decision 재분할은 Working Revision에 applicable embedded member를 추가한다. 이는 Goal Quality Contract
`2`의 field 변경과는 별개다. Working Revision을 직렬화·검증하는 companion 도구가 있으면 그 도구가 새 member와
current-selection을 이해하는지 실제로 대조해야 한다. 이 문서 변경만으로 해당 도구가 갱신·검증됐다고 보지 않는다.
새로운 필수 machine field/enum/quality version이 필요하면 문서·template·validator·fixture·소비자 검토를 같은
변경 범위에서 다룬다. 문서만 고치고 도구를 갱신했다고 보고하거나 존재하지 않는 새 flag를 호출하지 않는다.
상세 설계 인계 기록은 기존 Goal scope와 validation 계약의 일부로 GEX가 소비한다. 현행 GEX가 허용하지 않는
새 승인·Gate를 이 문서만으로 추가하지 않는다. 이 r2는 P2G의 요청당 진행 방식만 바꾼다. 단계 전환을 외부에서
막는 runner·상위 AGENTS·소비자가 있으면 실제 규칙과 대조하여 별도 변경 범위를 명시한다. 이 문서 파일만
교체했다고 그 도구나 GEX의 권한 정책까지 갱신됐다고 보고하지 않는다.
<!-- P2G-DESIGN-HANDOFF END -->

<!-- P2G-ROADMAP-USE START -->
### 8.9 상위 로드맵을 범위별로 Policy Topic과 Goal로 전환

로드맵은 이 파이프라인의 입력·계획 자료이지, 이미 code-reviewed된 정책·Final Freeze·Phase Plan·Goal Set이
아니다. 로드맵의 항목 ID와 완료 표시는 P2G/GEX artifact identity와 실행 증거를 대신하지 않는다.

#### 한 번에 선택할 경계

- 로드맵 항목 하나를 참고하되, 선택 Topic은 승인할 의미와 실제 변경 경계를 기준으로 제안한다. 로드맵
  항목 하나가 여러 정책·Goal로 나뉘거나 여러 항목의 일부가 하나의 Topic/Atomic Group으로 묶일 수 있다.
- Topic은 현재 의미·완료 기준을 닫을 수 있을 만큼 작아야 하지만, 필수 producer/consumer·저장·장착·등록
  전환을 범위 밖으로 밀어내 유효하지 않은 중간 상태를 만드는 크기여서는 안 된다.
- 전체 장기 로드맵을 한 Topic에 넣고 모든 미래 세부를 동시에 결정할 필요는 없다. 반대로 selected in-scope
  의미에 필요한 질문은 “후속 로드맵 항목”이라는 이유로 deferred하지 않는다.
- Topic 간 공유 정책·계약은 원래 owner의 exact source와 적용 범위를 참조한다. 이미 승인된 의미를 매번 다시
  질문하거나 서로 다른 local copy로 독립 변경하지 않는다. current 기준과의 일치·변경 영향은 여전히 확인한다.
  다른 Topic의 source를 참고했다는 이유로 서로의 Working Revision current set이나 Change obligation을 합치지
  않는다. local scope의 Decision/Element와 외부 선행 계약 source의 역할을 구분한다.
- 선행 계약이 문서 제안인지, frozen policy인지, 실제 완료된 구현인지 구분한다. 나중에 구현할 결과를 현재
  존재하는 API나 current evidence처럼 사용하지 않는다. 계획상 dependency는 P2G-40에서 검증할 수 있지만,
  단계 성립 자체를 좌우하는 current 근거 부족은 Phase dependency로 숨길 수 없다.

#### 권장 진행 관계

```text
로드맵의 다음 후보와 관련 공통 계약 확인
  → 사용자 승인 scope의 Policy Topic 선택
  → 승인한 종료 결과까지 필요한 P2G 단계의 결과를 순서대로 검증·확보
  → exact Handoff와 선택 Goal을 후속 GEX에 인수
  → 별도 실행 권한 아래 구현·검증·완료 감사
  → 실제 완료 evidence와 남은 위험을 반영해 다음 로드맵 후보 판단
```

위 화살표만으로 실행 권한이 생기지는 않는다. 사용자가 선택한 Topic을 Goal/Handoff까지 요청했다면
`bounded_pipeline`으로 필요한 P2G 단계를 이어가고, 특정 단계만 요청했다면 거기서 멈춘다. 이미 current인
prerequisite는 재사용한다. 다음 로드맵 항목·다른 Topic을 임의로 선택하거나 구현까지 연장하지 않는다.
P2G-60 이후의 Goal 선택·구현·검증·완료 감사는 별도 권한 아래 GEX의 책임이다.

하나의 Topic이 여러 Phase Goal로 변환됐다면 각 Phase를 실행할 때마다 P2G-00부터 다시 수행하지 않는다.
accepted Handoff, exact Goal과 완료된 선행 Phase의 evidence를 사용한다. 예상 implementation delta의 실현은
재동결 사유가 아니며, 정책 의미·review premise·Phase 경계가 실제로 달라졌을 때만 owner 경로로 반환한다.

#### 상태와 근거의 분리

로드맵 항목을 “완료”라고 체크하는 것만으로 하위 Goal을 완료 처리하지 않는다. 필요한 경우 기존 상위 문서에
다음 관계를 조회용으로 표시할 수 있다. 새 필수 Registry, 별도 Exact Ref 또는 전역 상태 모델은 요구하지 않는다.

| 로드맵 항목·선택 범위 | owning Policy Topic/Working Revision | current Freeze·Phase Plan·Goal Set/Handoff | GEX 실행·완료 evidence | 남은 의존·미정 |
| --- | --- | --- | --- | --- |

상위 로드맵의 coverage는 연결된 전체 필수 Goal과 통합 검증의 완료 evidence에서 계산한다. 일부 Phase 성공,
문서 동결 또는 Goal 작성 완료를 제품 구현 완료로 승격하지 않는다.

예: “사본별 성장”은 인벤토리 배열만 바꾸는 단위가 아닐 수 있다. 같은 사본을 장착·성장·지급·저장하는
reader/writer가 함께 바뀌어야 정상 상태를 유지한다면 그 경계를 함께 검토한다. 반면 사용자 화면의 추가 편집
편의가 그 의미 성립에 필수가 아니라면 별도 후속 작업일 수 있다. 나눌지 묶을지는 코드 근거와 Intermediate-State
Audit로 결정하며, 로드맵 번호의 개수에 맞춰 Goal을 강제로 생성하지 않는다.
<!-- P2G-ROADMAP-USE END -->

## 9. Working Policy Document 템플릿 골격

이 section은 section 5.1 Artifact Catalog에 등록된 artifact, embedded member와 view의 작성 schema만 제공한다.
여기의 heading이나 표를 근거로 Catalog에 없는 독립 artifact 또는 Exact Ref를 새로 만들지 않는다. 이름, owner,
종류나 Typed exact identity가 충돌하면 5.1을 먼저 수정한 뒤 이 template를 맞춘다.

### Document Metadata

- Working Policy Document ID/Identity:
- Policy Topic:
- Target Documentation Root:
- Policy Directory Name:
- Current Physical Path: `<Target Documentation Root>/goals/<Policy Directory Name>/WORKING_POLICY[.<locale>].md`
- Current Working Revision Ref / Current Working Checkpoint Ref:
- P2G-10 Completion Predicate: true / false / N/A
- P2G-10 Completion Evidence: exact exit validation / N/A
- Predecessor Working Revision Ref: N/A / exact value
- Revision Kind: Initial / Successor
- Predecessor Final Policy Freeze Ref: N/A / exact value
- Requested Result:
- Target Project/Repository:
- User-Approved Scope Ref:
- In Scope:
- Out of Scope:
- Policy Intake Snapshot Ref:
- Current Stage:
- Last User Stage Request: 실행 범위를 승인한 원래 요청 및 해당 후속 답변
- Progression Mode: single_stage / bounded_pipeline
- Authorized Terminal Result and Scope: 요청 근거와 종료 artifact/단계, Topic·in/out 범위
- Authorized Actions / Explicit Checkpoints / Limits: 필요한 문서·안전한 검사, 사용자 정지·예산 조건
- Stage Selection Basis: Explicit Single Stage / Authorized Endpoint Prerequisite / Owner-Bound Repair / Resumed Request
- Unrequested or Deferred Stages: 범위 밖·실제 blocker에 의해 수행하지 않은 단계
- Continuation / Stop / Resume Basis: 필요한 경우만 기록

위 실행 방식·권한·재개 값은 사용자 요청을 가리키는 작업 view다. Working Revision의 canonical payload나
새 Exact-Ref artifact가 아니며 이 값만 바꾸어 scope·정책·실행 승인을 생성하지 않는다.

### User-Approved Scope — Working Revision Payload/View

| Scope Item ID | Approved disposition: in / out | Exact user approval ref | Effective Working Revision |
| --- | --- | --- | --- |

AI proposed 또는 pending scope는 이 표에 넣지 않는다. 승인 전에는 외부 협의 응답으로만 유지하고
Working Revision을 publish하지 않는다. `explicitly_excluded` source disposition은 승인된 out-of-scope item과
exact approval reference가 있을 때만 사용할 수 있다.

이 표는 별도 P2G-00 artifact가 아니라 Working Revision의 `Approved metadata와 scope` payload를 읽기 쉽게
투영한 view다.

### Policy Intake Snapshot

- Policy Intake Snapshot Ref:
- Working Policy Document ID/Identity:
- Snapshot Kind: initial / successor
- Predecessor Policy Intake Snapshot Ref: N/A / exact value
- Created By Stage: P2G-00 initial / P2G-10 successor raw-input change

| Intake Item ID | Intake Kind | Exact Content 또는 Content Reference | User-Provided Locator | User-Stated Constraint | Capture Limitation |
| --- | --- | --- | --- | --- | --- |

필수 member field는 위 여섯 row field와 stable document/intake metadata(Working Policy Document
ID/Identity, Snapshot Kind, Predecessor Policy Intake Snapshot Ref 또는 N/A)다. producer는 Intake Item ID로
구분된 immutable row list를 고정한 Policy Intake Snapshot Ref를 발행한다. row는 생성 후 수정하지 않는다.
capture limitation은 bootstrap 시점에
원문을 확보하지 못한 사실만 기록하며 current source evidence를 대신하지 않는다.
P2G-00과 P2G-10의 snapshot 생성 활동은 이 표에서 Policy Element Candidate, policy meaning, source necessity
또는 disposition을 만들지 않는다. P2G-10 successor는 user-provided raw policy input 자체가 바뀔 때만 만들며
scope 또는 derived evidence만 바뀌면 만들지 않는다.

### Candidate Policy Element Disposition Record

- Produced by P2G-10 Candidate Policy Element Disposition Record Ref Set: explicit empty / record ref set

| Candidate Policy Element Disposition Record Ref | Candidate ID | Evaluated Input Working Revision Ref | Approved Target Scope/Change Request Ref 또는 N/A | Evaluated Policy Intake Snapshot Ref | Exact intake/source reference | Predecessor Candidate Policy Element Disposition Record Ref | P2G-10 result | Evidence/decision ref | Promoted Element ID/initial revision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |

P2G-10은 Policy Intake Snapshot과 current policy 자료에서 Candidate ID와 exact source/reference를 처음 만들고,
각 검토한 candidate에 대해 `promoted|duplicate|out_of_scope|not_a_policy_element` 중 하나의 immutable
disposition record를 만든다. `promoted`일 때만 stable Element ID와 initial `open` revision을 기록한다.
`out_of_scope`는 P2G-10 Evaluation Basis에 승인된 target scope 변경이 있으면 그 target scope를 기준으로
판정한다. scope 변경이 없으면 Input Working Revision의 current User-Approved Scope를 사용한다. 어느
경우든 exact approval reference가 있을 때만 사용할 수 있다. 후속 P2G-10에서 disposition을 바꾸면 기존
record를 수정하지 않고 이를 가리키는 successor disposition을 만든다.
세 evaluation column은 함께 해당 record의 절차상 `P2G-10 Evaluation Basis`를 나타내며 별도 artifact를
만들지 않는다. P2G-00은 Candidate ID, disposition record 또는 explicit empty set을 만들지
않는다.

### Candidate Policy Element Disposition Effective View

- Current Disposition View: absent / materialized view

| Candidate ID | Effective current Candidate Policy Element Disposition Record Ref | Result | Promoted Element ID/initial revision | Projection evidence |
| --- | --- | --- | --- | --- |

이 표는 immutable disposition record와 successor chain에서 계산한 mutable/materialized view다. 최초 P2G-10
입력에서는 view가 absent여도 정상이며 record 부재를 missing prerequisite로 판정하지 않는다. P2G-10 결과의
Current Disposition View는 Candidate ID 순으로 effective record를 선택한 projection이며 독립 Artifact ID나
Exact Artifact Ref를 갖지 않는다. 새 record가 선택되면 view만 다시 계산한다.

### P2G-20 Review Scope/Source Requirement Plan — Derived Artifact

- Review Scope/Source Requirement Plan Ref:
- Bound Input Working Revision Ref:

| Requirement ID | Policy Element Revision Ref | Required claim/boundary | Source type/role | Required identity/location constraint | Applicability/Necessity | Applicability condition | Policy/Scope Basis Ref | Validation requirement |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |

Applicability/Necessity는 `required|conditional|not_applicable` 중 하나다. 이 plan은 P2G-20이 exact Input Working
Revision의 current Policy Element claim/boundary에서 도출하는 derived artifact다. Review Scope/Source
producer는 Bound Input Working Revision Ref와 Requirement ID로 구분된 immutable row list를 고정한
Requirement Plan Ref를 발행한다. 아래 observation field는 plan payload에서 제외한다. plan 변경만으로 successor Working Revision을
만들지 않는다.

### Source Observation/Evidence Registry — P2G-20 Derived Artifact

| Source Observation Ref | Evaluated Input Working Revision Ref | Review Scope/Source Requirement Plan Ref | Requirement ID | Observed source identity/location/version | Disposition | Observation evidence | Limitation 또는 N/A reason | Observation time/version |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |

Disposition은 `confirmed|inaccessible|explicitly_excluded|unknown|N/A` 중 하나다. 이 registry는 Working Revision
canonical payload 밖의 P2G-20 derived artifact다. `N/A` row는 referenced plan requirement가 `not_applicable`이거나 immutable
`conditional` condition이 exact Input basis에서 false임을 evidence로 입증할 때만 유효하다. target Review
Batch의 Input Working Revision 및 published Plan과 exact-match하지 않는 row는 current observation으로 선택하지
않는다.

### P2G-00 Result Report — External by Default

- Selected Stage: P2G-00
- Performed Stage Work: initialization / entry-check-only / none
- Execution Status: completed / not-run-entry-failed / blocked
- Entry Result: eligible_new_document / same_identity_same_path_change / same_identity_same_path_current / path_occupied_by_different_identity / identity_already_exists_at_other_path / identity_or_path_indeterminate / nonconforming_policy_workspace_path / multiple_conflicts / other_failure
- Requested Result:
- Target Project/Repository Identity:
- Target Documentation Root:
- Policy Directory Name:
- Required Path Convention: `<Target Documentation Root>/goals/<Policy Directory Name>/WORKING_POLICY[.<locale>].md`
- User-Approved Initial Scope Ref: exact ref / N/A
- Requested Document Identity:
- Existing Document Identity: N/A / exact identity / unknown
- Requested Physical Path:
- Path Occupancy: unoccupied / occupied by same identity / occupied by different identity / unknown
- Existing Path for Requested Identity: N/A / exact path / unknown
- Identity Comparison Evidence:
- Path Occupancy Evidence:
- Exit Conditions Satisfied:
- Created Working Policy Document: exact path/identity / No
- Policy Intake Snapshot Ref: exact value / N/A
- Initial Working Revision Ref: exact value / N/A
- Unperformed Later Stages:
- Recommended Next Stage and Reason:
- New User Request Required: Yes / No — 실행 범위와 실제 결정·권한 필요에서 판정
- Continuation Disposition: next-authorized-stage / terminal-result / user-checkpoint / blocked
- Resume Requirement: 해당할 때 필요한 결정·자료·권한
- Existing Document No-Mutation Evidence: N/A / evidence

### Working Revision

- Working Revision Ref:
- Predecessor Working Revision Ref: N/A / exact value
- Predecessor Final Policy Freeze Ref: N/A / exact value
- Approved Metadata/Scope:
- Policy Intake Snapshot Ref:
- Current Policy Element Revision Map:
- Decision Revision Ref Set:
- Decision Repartition Member Set: 이번 transition의 member / 재분할 없음이면 생략
- Open Policy Question Ref Set:
- Post-Freeze Change/Closure/Authoritative Declaration Basis:

Working Policy Document view는 각 instance에 `Checkpoint Selection: current|historical`과 derived
`P2G-10 Completion: true|false`를 표시할 수 있다. 두 값은 Working Revision payload가 아니며 current head
selection과 P2G-10 exit condition에서 파생한다. `current/false` checkpoint는 정상적인 미결 작업 상태지만
P2G-20 prerequisite가 아니다.

아래 Code Review, Binding, Freeze Membership, Phase와 Goal 영역은 Working Revision 밖의 Derived Artifact
Registry다. 이 영역의 변경만으로 위 Working Revision Policy Basis를 수정하거나 새 Working Revision Ref를
계산하지 않는다.

### Current Composite Basis

- Working Revision Ref:
- Final Policy Freeze Ref: N/A / exact value
- Current Planning Freeze Selection: selected / none / blocked-by-invalid-prerequisite
- Phase Plan Ref: N/A / exact value
- Goal Set Ref: N/A / exact value
- Derived Applicable Post-Freeze Change Record Ref Set: explicit empty / ref set / INVALID
- Change Applicability Ancestry/Overlay/Branch Evidence:
- Expected Authoritative Change Obligation Declaration Ref Set: explicit empty / ref set / INVALID
- Authoritative Change Obligation Declaration Ref Set:
- Declaration Set Completeness/Exact-Match Evidence:
- Component Lineage/Compatibility Evidence:

### Policy Element Registry

| Element ID | Policy Element Revision Ref | Predecessor Policy Element Revision Ref | Policy statement | Status | Decision Revision Ref | Observable Policy Acceptance | Owning Phase |
| --- | --- | --- | --- | --- | --- | --- | --- |

`Status`는 `open|decided|code_reviewed`만 허용한다. Freeze 포함 여부는 아래 Freeze Membership Registry로
판정한다. P2G-00은 이 registry를 별도 empty artifact로 초기화하지 않는다. 최초 P2G-10이 formal element를
승격할 때 처음 materialize하며, 그 전에는 absent/empty view가 정상이다. Aborted P2G-20 candidate는 이
registry에 넣지 않는다.

### Finding Triage Log

| Finding ID | Affected elements | Finding | Primary classification | Disposition | Evidence/decision ref |
| --- | --- | --- | --- | --- | --- |

### Open Policy Question Log

| Open Policy Question Ref | Affected elements | Question | Options/Observable impact | Status | Decision Revision Ref |
| --- | --- | --- | --- | --- | --- |

### User Decision Log

사용자 직접 결정과 `8.6`의 위임에 따른 AI 결정을 함께 조회하되 결정 주체와 근거를 구분한다. 기존 열은
유지한다. 위임 결정의 `Exact user approval ref`는 실제 위임 승인을 가리키며, 조건 충족·exact AI 결정의
근거는 Finding Triage Log/연결 설명에서 추적한다. 사용자 답변을 합성하지 않는다.

| Decision Revision Ref | Predecessor Decision Revision Ref | Affected Policy Element Revision Ref Set | Approved decision | Approved Observable Policy Acceptance by Element ID | Exact user approval ref | Resolves Open Policy Question Ref Set |
| --- | --- | --- | --- | --- | --- | --- |

일반 initial/successor와 재분할의 current-selection은 P2G-10의 주요 활동 및 “Decision 재분할” 절을 따른다.
일반 successor의 predecessor는 transition 직전 checkpoint의 current member다. 재분할 result는 predecessor
`N/A`인 새 lineage이며, source provenance는 Result Working Revision 내부의 아래 member로 표현한다.
Predecessor가 없다는 이유만으로 이미 재분할된 source를 다시 추가할 수 없다. 이 log는 history 조회이고
current selection의 owner는 Working Revision이다.

#### Decision Repartition Member Set — applicable Working Revision 내부

| Repartition ID | Source Decision Revision Ref Set | Result Initial Decision Revision Ref Set | Preserved Meaning/Acceptance Coverage | Existing Approval Reference and Preservation Evidence | Affected Stable Element ID → Result Decision Revision Ref |
| --- | --- | --- | --- | --- | --- |

재분할 transition에만 작성한다. source가 직전 current set이고 result가 새 독립 lineage이며, 의미 전체 보존과
각 affected element의 단일 Decision 재결속이 검증되어야 한다. member가 자신을 담는 Working Revision Ref를
payload에 넣지 않는다. 표만 갱신해서 current set을 바꾸거나 별도 Decision Repartition Ref를 만들지 않는다.

### Technical Decision Log

| Technical Decision ID | Affected elements | Decision | Code basis | Alternatives | Result |
| --- | --- | --- | --- | --- | --- |

### Evidence Gap Log

| Gap ID | Required fact | Affected claim | Current disposition | Owner/next action |
| --- | --- | --- | --- | --- |

### Reviewed Source Basis — P2G-20 Derived Artifact

- Reviewed Source Basis Ref:

| Source ID | Axis/type | Repository/environment | External revision/content identifier/version | Disposition | Observation/evidence | Supported claim | Limitations |
| --- | --- | --- | --- | --- | --- | --- | --- |

이 immutable selected set은 P2G-20 Review Batch, Code Review Binding과 Final Policy Freeze가 참조하는 reviewed
Source Basis다. Review Scope/Source Requirement Plan과 다른 artifact이며 Working Revision canonical payload에는 포함되지
않는다. 각 row는 Source Observation/Evidence Registry의 exact observation/evidence를 참조하거나 P2G-20에서
새로 확보한 immutable evidence를 가리킨다. predecessor observation의 raw evidence를 재사용하려면 target Review
Batch의 Input Working Revision과 Plan에 exact-bind한 새 observation envelope가 먼저 있어야 한다.

### Code Review Artifact — Proposed Implementation 작성 내용

Code Review Artifact의 existing `Proposed Implementation`에 `8.8`의 적용 가능한 책임·상태·입출력·순서·취소·
기존 코드 전환·대안·검증·미결 상세 선택을 작성한다. Code Review Artifact Ref와 관련 element/source/evidence를
기존 binding 경로에 연결한다. 독립 Architecture Artifact나 Design Ref를 새 prerequisite로 만들지 않는다.
세부 표현은 표·본문·의사코드 중 선택할 수 있지만 현재 사실과 제안을 분리한다.

### Policy Element-to-Code Review Matrix

별도 view는 optional이다. Code Review Artifact와 Binding/Review Batch의 실제 관계에서 파생하며, 이 표의
부재만으로 P2G-40 입력이 불완전하다고 보지 않는다. 표가 있으면 원래 owning relation과 일치해야 한다.

| Code Review Artifact Ref | Review Batch Ref | Review path | Element revision/payload | Source basis | Current actual | Approved target | Approved Observable Policy Acceptance | Delta | Proposed implementation | Boundary coverage | Finding/gap refs | Candidate/publish result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |

### Candidate Review Envelope

- Candidate Review Envelope Ref:
- Candidate Batch ID:
- Input Working Revision Ref:
- Reviewed Element/Revision:
- Review Path: `policy_revision_review` / `code_reviewed_artifact_revalidation`
- Candidate Code Review Artifact Ref Set:
- Candidate Review Evidence Ref Set:
- Proposed `code_reviewed` Revision Payload: N/A / canonical payload
- Candidate Binding Payload:
- Candidate Result: pending / accepted-for-complete-result / aborted
- Abort Reason: N/A / finding or gap
- Lifecycle Disposition: non-current

### Review Batch

- Review Batch Ref:
- Candidate Batch Outcome: ready-for-complete-result / aborted
- Input Working Revision Ref:
- Review Scope/Source Requirement Plan Ref:
- Source Observation Ref Set:
- Code Review Artifact Ref Set:
- Review Evidence Ref Set:
- Reviewed Source Basis Ref:
- Produced `code_reviewed` Policy Element Revision Ref Set:
- Result Working Revision Ref:
- Result Current `code_reviewed` Policy Element Revision Ref Set:
- Result-basis Code Review Binding Ref Set:
- Batch Completeness Evidence:
- Applicable P2G-20 Obligation Key Set:
- Complete Result Validation: pending / passed
- Abort Evidence: N/A / exact evidence

`Produced code_reviewed Policy Element Revision Ref Set`은 이 batch가 새로 생성한 formal revision만 기록한다.
`Result Current code_reviewed Policy Element Revision Ref Set`은 Result current element map의 전체
`code_reviewed` set이며 binding coverage의
기준이다. `aborted` batch는 formal revision, Result Working Revision이나 current Code Review Binding Ref Set을
publish하지 않으며 해당 Review Batch의 Result/Produced/Binding field는 `N/A`다. `ready-for-complete-result`는 current를
뜻하지 않는다. 필수 구성, reference, coverage와 exit condition이 모두 검증된 complete result만 current다.

### Code Review Binding

- Code Review Binding Ref:
- Code Review Artifact Ref:
- Review Path: policy_revision_review / code_reviewed_artifact_revalidation
- Policy Element Revision Ref:
- Predecessor Code Review Binding Ref: N/A / exact value
- Bound Working Revision Ref:
- Target Current-Element-Map Match Evidence:
- P2G-20 Review Scope/Source Requirement Plan Ref:
- Reviewed Source Basis Ref:
- Review Evidence Ref Set:
- Covered Required Claim/Boundary Set:
- Evaluated Active Change/Obligation/Closure Set: N/A / IDs
- Applicable Policy Impact Closure Canonical Union: N/A / stable-key ordered member set
- Lifecycle Disposition:
- Successor Reuse Status:
- Revalidation Evidence:

`Final Policy Freeze Ref`는 이 record의 canonical payload에 포함하지 않는다.

### Optional Binding Inclusion Index/View

- Code Review Binding Ref:
- Final Policy Freeze Ref:
- Inclusion Observation/Evidence:

이 view는 Final Policy Freeze의 Current Code Review Binding Ref Set에서 계산한다. 생성하지 않아도 Freeze
currentness, Binding Completeness 또는 P2G-30 종료 조건에 영향을 주지 않는다.

### Freeze Membership Registry

| Final Policy Freeze Ref | Policy Element Revision Ref | P2G-30 Complete Result Evidence |
| --- | --- | --- |

한 revision은 여러 row를 가질 수 있다. published row는 Freeze disposition이 바뀌어도 삭제하거나 수정하지
않는다.

### Post-Freeze Change Log

| Post-Freeze Change Record Ref | Change ID | Origin Composite Basis | Predecessor Final Policy Freeze Ref | Changed premise | Direct elements | Candidate declaration | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |

Post-Freeze Change에는 mutable `unresolved|resolved|superseded` status를 기록하지 않는다. 아래 관계를 target
basis에 대조해 effective status를 파생한다.

### Change Obligation Declaration

- Change Obligation Declaration Ref:
- Change ID:
- Post-Freeze Change Record Ref:
- Origin Composite Basis:
- Policy Impact Closure:
- Artifact Invalidation Closure:
- Complete Initial Obligation Member Set:
- Stage Coverage Evidence:
- Invalidated Artifact Disposition Set:
- Strict Stage-Order Evidence:
- Acyclicity Evidence:

필수 구성, reference, stage coverage와 graph 조건이 모두 검증된 뒤에만 이 declaration을 current로 선택한다.
각 initial obligation은 아래 member schema로 declaration 안에 포함되며 `(Change Obligation Declaration Ref,
Obligation ID)`로 식별한다.

#### Initial Resolution Obligation Member Set

| Obligation ID | Owner Stage | Origin Composite Basis | Internal Prerequisite Obligation ID Set | External Prerequisite Obligation Key Set | Target Basis Requirements/Projection | Artifact Invalidation Closure Selector | Required Resolution Artifact Type |
| --- | --- | --- | --- | --- | --- | --- | --- |

External Prerequisite Obligation Key는 같은 Change의 이미 발행된 predecessor container member에만 사용할 수
있다. 현재 container 내부 관계는 Obligation ID를 사용한다. 같은 stage prerequisite, 자기 Container Ref의
선참조와 cross-Change prerequisite는 허용하지 않는다.

### Invalidated Artifact Disposition Set

| Change Obligation Declaration Ref | Artifact Ref | Derived disposition | Invalidation reason | Change ID |
| --- | --- | --- | --- | --- |

첫 열의 `Change Obligation Declaration Ref`는 parent declaration context를 표시하는 view field이며 embedded
Invalidated Artifact Disposition Set의 member payload에는 포함하지 않는다.

이 set이 authoritative lifecycle owner다. Artifact Disposition Registry는 이 set을 표시하는 derived view다.

### Declaration Supersession Record

- Declaration Supersession Record Ref:
- Predecessor Change Obligation Declaration Ref:
- Successor Change Obligation Declaration Ref:
- Change ID:
- Origin Composite Basis:
- Closure/Stage Coverage Preservation Evidence:
- Chain Acyclicity Evidence:

predecessor와 successor declaration의 필수 구성과 coverage 보존을 함께 검증한 뒤에만 successor를 current로
선택한다. 이 문서는 경쟁 write, CAS 또는 재시도 protocol을 규정하지 않는다.

### Resolution Obligation View

| Obligation Key | Container Kind | Container Ref | Obligation ID | Change ID | Owner Stage | Origin Composite Basis | Prerequisite Obligation Key Set | Target Basis Requirements/Projection | Artifact Invalidation Closure Selector | Required Resolution Artifact Type |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |

이 표는 declaration과 supersession container의 embedded member를 조회하는 derived view이며 독립 obligation
artifact나 Obligation Ref를 만들지 않는다.

### Resolution Record

- Resolution Record Ref:
- Obligation Key:
- Owner Stage:
- Exact Resolved Prerequisite Basis:
- Exact Result Composite Basis:
- Resolution Artifact Ref:
- Resolution Evidence:

### Effective Resolution Evaluation — Derived View

| Target Composite Basis | Change ID | Declaration validity | Active leaf set | Effectively resolved / unresolved / INVALID | Evaluation evidence |
| --- | --- | --- | --- | --- | --- |

### Obligation Supersession Record

- Obligation Supersession Record Ref:
- Change ID:
- Authoritative Change Obligation Declaration Ref:
- Predecessor Obligation Key Set:
- Successor Obligation Member Set:
- Closure Coverage Evidence:
- Applicable Composite Basis:
- Strict Stage-Order Evidence:
- Acyclicity Evidence:

Successor Obligation Member Set의 각 member는 Initial Resolution Obligation Member와 같은 field를 사용한다.
record Ref가 발행된 뒤 `(Obligation Supersession Record Ref, Obligation ID)`로 식별한다.

### Complete Stage Result — Semantic Bundle

- Owner Stage:
- Required Result Artifact/Relation Set:
- Applicable Resolution Record Ref Set: explicit empty / record ref set
- Applicable Obligation Supersession Record Ref Set: explicit empty / record ref set
- Cross-Reference/Coverage Evidence:
- Exit Condition Validation: pending / passed
- Current Selection Result:

이 bundle은 독립 artifact나 Exact Ref가 아니다. 필수 구성과 exit condition이 모두 검증된 뒤 result artifact를
current로 선택하는 의미 경계를 표시한다. candidate를 개별 작성할 수 있지만 부분 결과를 후속 prerequisite로
사용하지 않는다. 저장소 transaction, CAS, crash recovery 또는 동시성 protocol을 뜻하지 않는다.

P2G-30에서는 다음 relation을 Complete Stage Result가 소유한다.

```text
P2G-30 Complete Stage Result Relation
  Final Policy Freeze Ref
  Freeze Membership Set
  P2G-30 Resolution Record Ref Set
```

Freeze Membership Set은 `(Final Policy Freeze Ref, Policy Element Revision Ref)` row의 집합이며 Final Policy
Freeze payload가 아니다. Final Freeze Ref가 발행된 뒤 Complete Stage Result가 membership relation을
결속하므로 Final Freeze identity에 자기참조가 생기지 않는다.

P2G-40에서는 다음 artifact를 Complete Stage Result가 함께 검증한다.

```text
P2G-40 Complete Stage Result
  Phase Plan Ref
  Phase Plan Validation Receipt Ref
  P2G-40 Resolution Record Ref Set
  Applicable Obligation Supersession Record Ref Set
```

Phase Plan Ref는 candidate payload에 대해 먼저 발행할 수 있지만, receipt가 exact Phase Plan/Freeze basis를
가리키며 `passed`이고 repair record와 exit condition이 함께 검증되기 전에는 current로 선택하지 않는다.

### Artifact Disposition Registry

| Artifact Ref | Exact Composite Basis/Projection | Derived lifecycle disposition | Authoritative declaration/resolution ref | Successor reuse status | Reason/change/obligation ID | Owner stage | Resolution/binding record |
| --- | --- | --- | --- | --- | --- | --- | --- |

### Final Policy Freeze

- Final Policy Freeze Ref:
- Current Planning Freeze Selection: selected / not-selected
- Working Revision Ref:
- Review Batch Ref:
- Predecessor Freeze:
- Ordered `code_reviewed` Policy Element Revision Ref Set:
- Decision Revision Ref Set:
- Observable Policy Acceptance Set:
- Current Code Review Binding Ref Set:
- Current Reviewed Source Basis Ref:
- Binding Completeness Evidence:
- Applicable Change/Obligation Basis:
- Resolved Prerequisite Evidence:
- Known Unknowns:
- Compatibility/Migration Disposition:
- Lifecycle Disposition:

Freeze member의 authoritative selection은 `Ordered code_reviewed Policy Element Revision Ref Set`이다. 이
template에는 `Freeze Membership Set`을 넣지 않는다. 외부 membership relation은 위 P2G-30 Complete Stage
Result와 Freeze Membership Registry에서 표시한다.

`Observable Policy Acceptance Set`은 위 Ordered `code_reviewed` Policy Element Revision Ref Set의 각 member가
소유한 Observable Policy Acceptance를 `(Policy Element Revision Ref -> Acceptance)`로 정렬해 계산한
projection이다. 별도 policy owner가 아니며 member revision과 하나라도 다르면 candidate Freeze는 invalid다.

Current Planning Freeze Selection은 Final Policy Freeze payload 밖의 derived view다. Final Freeze,
Membership과 P2G-30 Resolution Record Ref Set의 필수 구성과 상호 참조가 검증된 complete result만 current로
선택한다. optional Binding Inclusion Index/View는 이후 필요할 때 계산한다.

### Phase Plan

- Phase Plan Ref:
- Lifecycle Disposition:
- Exact Prerequisite Composite Basis:
- Bound Final Policy Freeze Ref:
- Repair Basis: Applicable P2G-40 Obligation Key Set와 derived resolution/supersession status — no applicable Change;
  all typed sets explicit empty / exact applicable refs와 status
- Applicable P2G-40 Obligation Key Set: `explicit empty` / 각 exact key를 하나의 inline-code span으로 열거
- P2G-40 Resolution Record Ref Set: `explicit empty` / 각 exact Ref를 하나의 inline-code span으로 열거
- Applicable Obligation Supersession Record Ref Set: `explicit empty` / 각 exact Ref를 하나의 inline-code span으로 열거
- Goal Structure Fact: `single_phase` / `multi_phase`
- Known Unknown/Limitations: applicable item / N/A

#### Work Package Set

| Work Package ID | Covered Stable Element ID Selector Set and coverage role | Execution node disposition | Implementation boundary | Expected change | Prerequisites | Validation/completion | Applicable unknown |
| --- | --- | --- | --- | --- | --- | --- | --- |

selector는 이 Phase Plan의 Bound Final Policy Freeze Ref에 상대적이다. exact Policy Element Revision, Decision과
Observable Policy Acceptance는 Freeze에서 파생하며 이 표에 다시 쓰지 않는다. coverage role은
`implementation | supporting | validation_only` 중 해당 관계를 구분한다.

#### Dependency Graph

`Dependency Type`은 폐쇄된 enum이 아닌 권장 vocabulary다.

| Prerequisite Work Package | Dependent Work Package | Dependency Type | Evidence/reason | Broken prerequisite, valid state or completion claim if ignored |
| --- | --- | --- | --- | --- |

#### Cyclic SCC Disposition Set

둘 이상의 Work Package를 포함하거나 실질적인 self-loop가 있는 SCC만 기록한다. 해당 SCC가 없으면 이 heading과
empty table을 생략하고 Phase Plan Validation Receipt에 `cyclic_scc_count: 0`을 기록한다.

| SCC member Work Packages | Disposition | Boundary/edge reconstruction | Evidence | Recomputed cycle result |
| --- | --- | --- | --- | --- |

#### Atomic Group Set

| Atomic Group ID | Member Work Packages | Indivisible boundary and evidence |
| --- | --- | --- |

#### Ordered Phase Set

| Phase ID | Primary Stable Element ID Selectors | Supporting policy relations | Assigned final execution nodes | Dependencies | Exit acceptance |
| --- | --- | --- | --- | --- | --- |

`Assigned final execution nodes`에는 standalone Work Package 또는 Atomic Group만 기록한다. Atomic Group member를
개별 node로 다시 기록하거나 여러 Phase로 분할하지 않는다.

#### Intermediate-State Audit

| Phase ID | State immediately after completion | Applicable validity checks | Completion-claim evidence | Result/limitation |
| --- | --- | --- | --- | --- |

#### Parallelization Evidence

병렬화를 승인한 Phase 조합이 있을 때만 이 표를 작성한다. 승인하지 않으면 표를 생략하고 Phase Plan top-level
또는 ordered Phase summary에 `Parallelization: none approved`를 한 번만 기록한다.

| Phase combination | No dependency path | Boundary/cutover independence | File/merge disposition | Independent validation | Combined-state validity |
| --- | --- | --- | --- | --- | --- |

#### Integration Validation Ownership Map

| Integration acceptance / source | Seam disposition | Read-only locator, invocation and evidence contract | Owning Final Execution Node / Phase | Availability prerequisite | Evidence / limitation |
| --- | --- | --- | --- | --- | --- |

`Seam disposition`은 `existing_read_only | audit_local_read_only | assigned_execution_node` 중 하나다. durable
test/fixture/probe 또는 source change가 필요하면 `assigned_execution_node`를 사용하며 owner를 비워둘 수 없다.

`single_phase`에서는 Ordered Phase Set의 한 row에 final-state Intermediate-State Audit 결과를 합칠 수 있다.
이 경우에도 Work Package, dependency, Atomic Group과 Integration Validation Ownership Map은 생략하지 않는다.

### Phase Plan Validation Receipt — P2G-40 Derived Artifact

이 artifact의 full payload는 `validate_p2g40_receipt.mjs --write-receipt`가 Phase Plan과 Final Freeze에서
산출하는 JSON이다. 아래 항목은 사람용 summary view이며 JSON을 수동 작성하거나 수정하기 위한 양식이 아니다.
기존 target과 새 계산 결과가 다르면 기존 Receipt를 덮어쓰지 않고 successor Receipt Ref/path를 사용한다.

- Phase Plan Validation Receipt Ref:
- Evaluated Phase Plan Ref:
- Bound Final Policy Freeze Ref:
- Validator/Method Version:
- Freeze/Prerequisite/Repair Basis Exact-Match: passed / failed
- Selector Resolution: resolved / missing / ambiguous / extra counts
- Primary Phase Coverage: covered / missing / duplicate / extra counts
- Work Package/Atomic Group Partition: passed / failed
- Dependency Graph: node/edge counts, cyclic SCC count, disposition coverage, condensation acyclicity
- Final Execution Node-to-Phase Cardinality: passed / failed
- Intermediate-State Audit Row Completeness: passed / failed
- Parallelization Evidence Check: not claimed / passed / failed
- Integration Validation Ownership Coverage: passed / failed
- Goal Structure Fact Check: passed / failed
- Immutable Machine Detail: embedded structured result / exact content reference
- Overall Result: passed / failed

receipt는 exact expansion, count, duplicate와 structural coverage를 소유한다. Phase Plan의 dependency 이유,
Atomic Group 경계, intermediate-state validity와 validation seam이 타당한지는 receipt가 대신 결정하지 않는다.
Overall Result가 `passed`이고 exact evaluated refs가 일치할 때만 candidate Phase Plan을 current로 선택한다.

### Goal Set

- Goal Set Ref:
- Predecessor Goal Set Ref: exact ref / N/A
- Lifecycle Disposition:
- Exact Prerequisite Composite Basis:
- Goal Structure Fact: `single_phase` / `multi_phase`
- Phase Plan Ref:
- Final Policy Freeze Ref:
- Complete Goal Member Set:
- Applicable P2G-50 Obligation Key Set:
- Goal Quality Contract Version: `2`
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

#### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |

각 current Phase를 정확히 한 Single/Phase Goal ID에 연결한다. Master Goal은 target이 아니며 Final Execution
Node는 Phase Plan의 배정을 그대로 기록하고 Atomic Group member를 개별 node로 전개하지 않는다.

#### Shared Validation Baseline Reuse Contract (applicable directive가 있을 때)

- User-Approved Directive Ref:
- Initial Full Baseline Owner / Timing / Command Set:
- Baseline Evidence Key: command/selectors, fixture/input, relevant source/data/test/config, toolchain/lock identity, result
- Reuse Rule: exact relevant input identity와 evidence contract가 unchanged일 때만
- Phase-Start Rule: Phase 시작만으로 full validation을 재실행하지 않음
- Production Input Invalidation / Focused Revalidation Map:
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule:
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule:
- Final Full Baseline Owner / Timing / Command Set:
- Failure/Zero-Execution/Post-Run-Change Retry Rule:
- Normal Full-Validation Count: initial 1 + final 1; invalidation/failure retry는 별도

#### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |

Phase Plan의 Integration Validation Ownership Map 전체를 exact ID로 옮긴다. `assigned_execution_node` row는
Phase-to-Goal mapping에서 파생한 exact owning Goal ID와 Final Execution Node를 기록한다. read-only seam은
locator/evidence contract를 기록하며 구현 node를 발명하지 않는다. `multi_phase` Master member는 이 common
mapping을 coordination view로 소비한다.

Goal ID는 Goal Set 안의 stable member key다. exact member는 `(Goal Set Ref, Goal ID)`로 식별하며 독립 Goal Ref를
만들지 않는다. `Complete Goal Member Set`은 아래 member의 전체 canonical payload를 포함한다. Goal ID 목록은
이 set에서 계산하는 index이며 별도 canonical ID Set, Exact Artifact Ref 또는 Content Digest를 만들지 않는다.

#### Single/Phase Goal Member

- Goal ID:
- Goal Kind: `single` / `phase`
- Phase ID:
- Assigned Final Execution Node Set:
- Covered Policy Element Revision Ref Set:
- Covered Decision Revision Ref Set:
- Derived Observable Policy Acceptance Coverage:
- Objective / Contract-Visible Result:
- User-Visible Problem:
- User-Visible Completion Condition Set:
- In-Scope / Out-of-Scope:
- Reviewed Current Behavior / Implementation Delta:
- Goal-Local Source of Truth / Owning Layer:
- Goal-Local Invariant Set / Source Refs:
- Code/Data/Schema/Store/Transport/Consumer Boundary:
- Consumer Verification Boundary: workspace-confirmed / canonical-external / known-inaccessible / unknown
- Goal-Local Dependencies / Execution Prerequisites:
- Validation Plan / Completion Matrix:
- Structural Completion / Legacy and Fallback Disposition:
- Canonical Documentation Obligation:
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan:
- Assigned Durable Integration Validation Seam Coverage: item / N/A with reason
- Applicable Migration/Compatibility/Rollout/Rollback: item / N/A with reason
- Goal-Local Known Unknown / Blocker / Stop Condition:
- Applicable Risk / Reviewer Requirement and Evidence: item / N/A with reason

##### 기존 필드에 포함할 상세 설계 인계

`8.8`을 적용해 위 boundary에는 권장 구조·조사 경계, prerequisite에는 구현 전 상세 설계·준비 점검 조건,
invariant에는 보존할 공유 계약, validation에는 설계 판단을 입증할 예제, unknown에는 미정·반환 조건을 넣는다.
이미 충분히 서술했다면 이 heading 자체를 별도 산출물이나 중복 표로 만들 필요는 없다. 다음 문장은 형식 예시다.

> Code/Data/… Boundary: 확인한 producer→owner→consumer 경계를 따라 실제 코드를 조사하고, 실행 handle과
> 결과·취소·상태 수명을 보존하는 권장안을 상세화한다. 재사용·교체 symbol과 남은 가정을 명시한다.
> Goal-Local Dependencies / Execution Prerequisites: 선행 구현 evidence를 확인한 뒤 정확한 인터페이스·전환·
> 실패·시험 입력을 설계하고, 공유 계약 정합성을 구현 전에 점검한다. 그 점검은 현행 GEX 절차 안에서 기록한다.

예시 문장을 그대로 붙여 실제 계약·evidence를 대신하지 않는다. Goal 크기와 위험에 맞게 구체화한다.

##### Validation Dimension Matrix

| Dimension | Disposition (`required` / `N/A`) | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| `success` |  |  |  |
| `invalid_input` |  |  |  |
| `failure_rollback` |  |  |  |
| `retry_idempotency` |  |  |  |
| `ordering_concurrency` |  |  |  |
| `boundary_empty_max` |  |  |  |
| `disconnect_restart_persistence` |  |  |  |
| `live_data` |  |  |  |
| `producer_consumer` |  |  |  |
| `legacy_fallback_absence` |  |  |  |

#### Master Goal Member (`multi_phase` only)

- Goal ID:
- Goal Kind: `master`
- Complete Phase Goal ID Set:
- Derived Sequential/Parallel Relations:
- Coordination Dependencies/Conditions:
- Derived Cross-Phase Invariants and Source Refs:
- Derived Integration Acceptance and Source Refs:
- Integration Validation Seam Disposition / Locator or Owning Phase Goal and Final Execution Node:
- Baseline Reuse Coordination / Initial and Final Full Baseline Owning Phase Goal:
- Overall Completion Conditions:
- Applicable Integration Risk / Reviewer Requirement and Evidence: item / N/A with reason
- Assigned Phase / Final Execution Node / Implementation Work Ownership: None

### Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |

이 section은 현행 `validate_p2g50_goal_set.mjs`가 읽는 Goal Set 내부의 파생 입력 표현이다. 별도 Matrix 파일을
prerequisite로 만들지 않는다. Final Freeze의 모든 ordered Policy Element member를 정확히 한 row로 파생한다.
Policy Element/Decision Ref는 Freeze의 exact member이고 Primary Phase는 Phase Plan의 validated selector relation,
Goal ID는 위 Phase-to-Goal mapping에서 가져온다. Observable Policy Acceptance는 해당 member의 값을 상속하며
재정의하지 않는다. 원래 관계와 불일치하면 표를 수동으로 맞춰 결함을 숨기지 않고 owning relation을 확인한다.
도구가 직접 owning relation을 읽도록 변경·검증되기 전에는 이 표를 출력 표현으로 계속 제공한다.

### Goal Set Structural Validation Summary

- Validator/Method Version: `p2g50-goal-set-validator/1.1.0`
- Invocation: 위 P2G-50 repository 표준 구조 검증 명령과 `--quality-contract 2`
- Overall Result: passed / failed
- Core Counts: Freeze members / Phases / Final Execution Nodes / Goal members / mapping / traceability / integration /
  quality-contract validation-dimension rows
- Failure Summary: explicit empty / exact failures

이 summary는 실행 결과의 사람용 view이며 별도 Receipt artifact가 아니다. `passed` detail을 중복 표로 다시
전개하지 않는다.

### Durable Handoff Artifact

- Handoff Artifact Ref:
- Predecessor Handoff Artifact Ref: exact ref / N/A
- Lifecycle Disposition:
- Exact Result Composite Basis:
- Final Policy Freeze Ref:
- Phase Plan Ref:
- Goal Set Ref:
- Remaining Evidence Gap / Blocker / Limitation / Out-of-Scope Summary:
- P2G Scope Completed: Policy freeze, Phase plan and Goal authoring delivered
- Authorization Boundary: Goal activation, implementation authorization, production mutation and completion not approved
- Applicable P2G-60 Obligation Key Set:
