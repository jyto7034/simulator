# Goal Execution Pipeline

Status: `ready`

Scope: P2G-60 Handoff 이후, 사용자가 선택한 Single/Phase Goal의 구현·검증·완료 감사와 multi-phase Goal Set의
Master 최종 통합 감사까지

Upstream contract: [`policy_to_goal_pipeline.md`](./policy_to_goal_pipeline.md)

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
전체 실행 의도가 명시되면 `goal-run`이다. `goal-run`의 terminal result 범위는 요청 대상에 따라 다음과 같다.

| 요청 대상 | `goal-run` 범위와 terminal result |
| --- | --- |
| Single Goal | `GEX-00 -> GEX-10 -> GEX-20`; `Complete / High` 또는 중단 조건 |
| Phase Goal member | 해당 Phase의 `GEX-00 -> GEX-10 -> GEX-20`; `Complete / High` 또는 중단 조건 |
| multi-phase Goal Set 또는 Master Goal | 모든 Phase의 `GEX-00 -> GEX-10 -> GEX-20`과 마지막 `GEX-30`; Master `Complete / High` 또는 중단 조건 |

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
   순서대로 수행할 수 있다.
4. 선택 단계의 prerequisite가 없으면 뒤 단계로 건너뛰지 않는다. `stage-run`은 누락된 prerequisite와 필요한
   owner 단계를 보고하고 멈춘다. `goal-run`은 section 6.6이 명시적으로 허용한 in-scope correction 또는
   re-audit로 prerequisite를 충족할 수 있을 때만 그 분리된 pass를 먼저 수행하며, 그 밖에는 current run을
   중단한다.
5. 사용자가 단계나 Goal을 지정하지 않았고 직접 결과와 current artifact로도 실행 대상 또는 실행 모드를
   정할 수 없을 때만 사용자에게 묻는다.
6. `GEX-10`의 직접 `stage-run` 요청 또는 active `goal-run` authorization은 선택된 Goal 범위의 구현·테스트와
   필요한 코드/데이터/문서 수정을 승인한다. `GEX-00` 완료, P2G-60 Handoff 또는 Goal 작성만으로 구현이
   승인되지는 않는다.
7. `GEX-20`과 `GEX-30` pass는 감사와 감사 증거 작성을 승인하지만 구현 수정을 승인하지 않는다. active
   `goal-run`도 이 read-only 경계를 바꾸지 않는다.
8. `goal-run`의 자동 전환 권한은 exact target과 그 Goal의 기술적 correction에만 적용된다. 정책 의미 변경,
   Goal/Phase 구조 변경, 범위 확대, 배포 또는 외부 환경 변경은 별도 사용자 결정이나 권한 없이는 수행하지 않는다.

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
- 선택된 Goal 범위의 구현과 검증
- 구현 과정과 실패·재검증 증거 기록
- Single/Phase Goal 완료 감사
- multi-phase Goal Set의 최종 Master 통합 감사
- 기술 결함의 동일 Goal 재작업 또는 올바른 P2G owner 단계로의 반환

### 제외

- 새로운 material policy 결정
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
  -> GEX-00 실행 인수와 Single/Phase Goal 선택
  -> GEX-10 선택된 Goal 구현과 검증
  -> fresh GEX-20 Single/Phase 완료 감사
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

`stage-run`은 위 흐름에서 요청된 한 단계 뒤 멈춘다. `goal-run`만 section 6.6의 전환을 자동으로 이어간다.

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
| Remaining Gap/Blocker/Limitation/Out-of-Scope Summary | GEX-00과 각 exit 판단 | 숨기거나 pass로 바꾸면 안 되는 실행 전제 |
| P2G Scope/Authorization Boundary | GEX-00/10 | P2G는 구현을 승인하지 않았으며 direct GEX-10 또는 exact goal-run authorization이 별도로 필요함 |

GEX는 위 upstream payload를 복제하거나 수정하지 않는다. GEX 실행 문서는 accepted Ref, current actual과 실행
evidence를 연결한다.

## 4. 권한과 판단 경계

### 사용자 소유

- material gameplay, balance, UX와 public contract 의미
- persistence loss/lifetime, compatibility, migration과 rollout의 정책 의미
- `stage-run` 또는 `goal-run`의 시작, exact target과 authorization 범위
- goal-run 중단 지점에서 필요한 정책·범위·외부 권한 결정
- 별도 승인이 필요한 production, 배포 또는 외부 환경 변경

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
| `PLAN.md` | exact Goal, 실행 범위, current baseline, 구현 방향, validation·완료·중단 조건 | `GEX-00`, 필요 시 `GEX-10`의 current evidence 보완 |
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

선택한 directory와 네 문서의 normalized path는 GEX-00 산출물과 `PLAN.md`에 기록한다. 이후 GEX pass와
correction/re-audit는 이 path를 재사용한다. execution-document path는 upstream Goal identity를 대체하는
authority가 아니다.

### 기존 Goal 실행 문서 재개 선독 Gate

exact `(Goal Set Ref, Goal ID)`에 대응하는 실행 문서가 이미 있으면 GEX-00은 코드 검색, repository status·diff
조사, 테스트, 검증 또는 production mutation보다 먼저 기존 기록의 current 실행 상태를 인수한다. 이 gate의
목적은 문서 전체 history를 다시 소비하는 것이 아니라 재개에 필요한 최소 current context와 직접 dependency를
확정하는 것이다.

먼저 execution-document directory 안의 다음 세 문서 존재 여부만 확인한다.

- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`

세 문서가 모두 없고 exact Goal을 가리키는 다른 실행 history나 `COMPLETION_AUDIT.md`도 없을 때만 신규 Goal
실행 문서 초기화 경로를 사용한다. 세 문서가 모두 있으면 기존 Goal 재개 경로를 사용한다. 하나 또는 두 개만
있거나, 세 문서는 없지만 같은 exact Goal의 audit/history가 있거나, 어느 하나의 필요한 current section을 읽을
수 없으면 남은 내용을 추측하거나 새 파일로 보충하지 않는다. 코드 조사·수정·테스트를 시작하지 않고
existing/missing/unreadable 문서, 확인 가능한 current pass/result, 안전하게 재개할 수 없는 이유와 필요한
사용자 입력을 보고한다.

`COMPLETION_AUDIT.md`는 구현 전부터 존재해야 하는 세 문서에는 포함하지 않는다. 다만 완료된 Goal, correction,
re-audit 또는 predecessor completion을 인수하는 재개라면 latest applicable verdict, Audit Target Record와
currentness 판정이 필수 dependency이므로 해당 부분을 함께 읽는다. 이때 필요한 audit가 명시되어 있지만 없거나
읽을 수 없으면 dependency를 충족한 것으로 추측하지 않는다.

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
- Phase Plan과 Goal Set이 선언한 직접 predecessor Phase의 latest applicable `Complete / High` 감사와 currentness
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

선독 뒤의 재개 disposition은 새 Status 체계를 만들지 않고 이 문서의 GEX pass result, `Complete / High`,
`Not Complete`, `Blocked`와 audit currentness 계약을 따른다. result나 dependency가 unknown 또는 모순이면 임의
Status를 부여하지 않고 중단한다. 이 gate는 실행 문서를 runtime·live data·wire 또는 정책 meaning의 새 source of
truth로 승격하지 않으며, exact upstream authority와 current actual 판정 순서는 section 6을 그대로 따른다.

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
파일 경로가 manifest 밖이라는 사실만으로 non-impacting을 주장하지 않는다. `stale` audit은 prerequisite나
GEX-30의 current Phase audit으로 사용할 수 없으며 fresh GEX-20 또는 GEX-30이 필요하다.

fresh re-audit은 implementation을 수정하지 않고 current actual에서 새 Audit Target Record를 고정할 수 있다.
이 경로는 delta가 이미 완료된 GEX-10 또는 Phase Plan이 승인한 successor Phase mutation으로 정확히 귀속되고,
upstream Goal 의미와 completion condition이 그대로이며, re-audit 전에 source/data correction이 필요하지 않을
때만 허용된다. 새 record는 predecessor Audit Target Record, changed identity와 delta owner를 가리키며 이전
record/verdict를 덮어쓰지 않는다. delta owner가 없거나 implementation defect·Goal boundary change가 필요하면
각각 GEX-10 또는 section 8의 P2G owner로 반환한다.

후속 Phase가 predecessor의 assigned execution node, contract-visible output, schema/store/transport/consumer,
legacy disposition, validation seam/fixture 또는 completion evidence를 변경하면 predecessor audit은 기본적으로
`stale`이다. Phase Plan이 예상한 delta라도 current completion condition에 대한 fresh read-only GEX-20으로
재감사해야 한다. 변경이 audit impact surface와 무관하다는 direct evidence가 있을 때만
`current / proven non-impacting delta`로 유지할 수 있다.

### 문서별 최소 내용

`PLAN.md`는 다음을 식별할 수 있어야 한다.

- exact Handoff/Goal Set/Goal member와 담당 Phase
- `stage-run | goal-run`, root execution target, current pass와 authorization 범위
- exact execution-document directory와 document path set
- objective, contract-visible result, in-scope와 out-of-scope
- predecessor/dependency와 current execution prerequisite
- current behavior, target behavior와 implementation boundary
- current dirty baseline과 보존할 unrelated work
- applicable Goal Quality Contract version, Validation Dimension Matrix, validation 계획, completion conditions와
  stop conditions

`EXPERIMENTS.md`는 완료 주장에 사용한 사실을 재검토할 수 있어야 한다. 중요한 명령은 대상, 실제 결과,
실행 수 또는 relevant output을 기록한다. 실패한 접근과 수정·재검증도 삭제하지 않고 남긴다.

`EXPERIMENT_NOTES.md`는 current summary이지 별도 정책 authority가 아니다. 새 정책 질문은 AI의 기술 결정처럼
기록하지 않고 사용자 결정 필요와 영향 범위를 표시한다.

`COMPLETION_AUDIT.md`는 최소한 다음을 포함한다.

- exact audit target과 upstream Goal/Phase
- Audit Target Record와 audit 시작 시 currentness comparison 결과
- 감사 방법과 구현 pass와의 분리 관계
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

최초 GEX-00은 current Handoff에서 시작한다. 실행이 시작된 뒤의 basis는 같은 Handoff와 section 5의
currentness 판정을 통과한 `Complete / High` predecessor Phase audit의 조합으로 읽는다. Goal이 승인한 implementation delta와
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

### 6.4 구현과 검증

- 작은 변경 단위마다 필요한 focused validation을 수행하고 실패 원인과 재검증을 기록한다.
- 완료 전에는 Goal 위험과 변경 표면에 비례한 broader validation을 수행한다.
- zero test, skipped test, 실행되지 않은 command와 unknown result는 pass가 아니다.
- 기존 baseline failure와 이번 변경이 만든 failure를 구분한다.
- validation을 생략하면 이유와 완료 주장에 미치는 영향을 기록한다. 필수 조건을 침범하는 생략은 완료를
  허용하지 않는다.
- 임시 fallback, dual path, shim 또는 compatibility layer는 Final Freeze/Goal이 허용하고 범위·검증·제거
  조건이 충분한 경우에만 구현한다.
- 구현으로 바뀐 durable contract, schema, wire 문서나 fixture가 있다면 같은 Goal 범위에서 정합성을 맞춘다.

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

### 6.5 새 질문과 결함

GEX-10 안의 일반적인 기술 실패는 선택된 Goal 의미와 범위를 유지하는 한 AI가 조사·수정·재검증할 수 있다.
하지만 다음 경계를 만나면 임의로 해결하지 않는다.

- 새로운 material policy 또는 acceptance 의미가 필요함
- current code evidence가 Final Freeze나 implementation boundary를 무효화함
- Work Package, Atomic Group, Phase dependency 또는 Goal mapping을 바꿔야 함
- required external consumer/evidence가 없어 완료 조건을 증명할 수 없음
- Goal 밖의 독립 작업을 포함해야만 구현 가능함

이 경우 발견 사실, 이미 수행한 안전한 작업, 영향 범위와 필요한 owner 단계를 기록하고 현재 GEX 단계를
완료하지 않는다.

### 6.6 Goal-run 자동 전환과 중단

active `goal-run`은 각 GEX pass가 종료 조건을 충족하고 결과와 evidence를 기록한 뒤에만 다음 pass로 이동한다.
`Not Complete` 또는 `Blocked` finding은 자동 전환 전에 영향 범위를 다음과 같이 판정한다.

- `Phase-local`: selected Phase와 그 dependent Phase의 prerequisite 또는 completion만 침범함. 독립 Phase의
  upstream basis, shared runtime/data/schema/store, authorization, dirty isolation과 validation에는 영향이 없음
- `run-global`: accepted upstream basis, 공용 execution environment/data/schema/store, authorization, dirty
  isolation 또는 여러 Phase의 completion을 침범함

영향 범위가 unknown이면 `run-global`로 취급한다. 별도 blocker registry나 resume state machine은 만들지 않고
finding과 근거를 `EXPERIMENT_NOTES.md` 및 applicable audit에 기록한다. 자동 전환은 다음 계약을 따른다.

| current pass 결과 | goal-run의 다음 동작 |
| --- | --- |
| `GEX-00` 성공 | 같은 selected Goal의 `GEX-10` 시작 |
| `GEX-10` 종료 조건 충족 | 구현 mutation을 멈추고 fresh `GEX-20` 시작 |
| `GEX-00/GEX-10 Phase-local Blocked`, Goal Set target에 독립 eligible Phase 존재 | 현재 Phase와 dependent Phase를 보류하고 독립 Phase의 `GEX-00` 시작 |
| `GEX-00/GEX-10 Phase-local Blocked`, Single/Phase target 또는 독립 eligible Phase 없음 | target Blocked terminal result 보고 |
| Goal Set target에서 expected delta로 predecessor audit만 `stale`이고 correction은 불필요함 | predecessor의 fresh `GEX-20` 후 중단됐던 pass 재시도 |
| `GEX-20 Complete / High`, Single/Phase target | target terminal completion 보고 |
| `GEX-20 Complete / High`, Goal Set target에 미완료 Phase 존재 | dependency상 eligible한 다음 Phase의 `GEX-00` 시작 |
| `GEX-20 Complete / High`, Goal Set의 모든 Phase 완료 | fresh `GEX-30` 시작 |
| `GEX-20 Not Complete`, selected Goal 내부 기술 결함 | audit pass를 닫고 correction `GEX-10`, 이후 fresh `GEX-20` |
| `GEX-20 Not Complete/Blocked`, Goal Set target의 더 진행할 수 없는 Phase-local finding과 독립 eligible Phase 존재 | 현재 Phase와 dependent Phase를 보류하고 독립 Phase의 `GEX-00` 시작 |
| `GEX-20 Not Complete/Blocked`, Single/Phase target 또는 독립 eligible Phase 없음 | target Not Complete/Blocked terminal result 보고 |
| `GEX-30 Not Complete`, 영향 Goal 내부 기술 결함 | 영향 Goal의 correction `GEX-10`과 fresh `GEX-20`, 이후 fresh `GEX-30` |
| `run-global Blocked`, P2G owner 필요 또는 GEX-30 Blocked | 자동 진행을 중단하고 finding, 영향과 필요한 owner 보고 |
| `GEX-30 Complete / High` | Goal Set terminal completion 보고 |

Goal Set target에서 dependency상 독립적인 eligible Phase가 둘 이상이면 Phase Plan의 명시적 순서를 우선하고,
순서가 없으면 Phase Plan의 선언 순서를 안정적인 scheduling tie-break로 사용한다. 독립 Phase의 직렬 실행 순서는
새 policy 또는 dependency 의미가 아니다. 순서가 observable result에 영향을 주거나 dirty collision을 만들 수
있으면 임의로 선택하지 않고 goal-run을 중단한다.

Phase-local finding 뒤 독립 Phase를 진행하려면 그 Phase가 원래 dependency상 eligible하고, blocked Phase의
mutation/evidence와 dirty overlap이 없으며, section 5의 predecessor audit currentness와 shared basis가
유효해야 한다. 독립 Phase 실행은 blocked Phase를 완료로 바꾸거나 Master prerequisite를 충족시키지 않는다.

selected Phase가 dependent Phase의 mutation 없이는 독립적으로 유효한 중간 상태와 `Complete / High` 완료
조건을 충족할 수 없으면, 현재 Phase 경계 또는 Atomic Group disposition이 유효하지 않은 것으로 판정한다.
dependent Phase의 GEX-00/GEX-10을 조기 실행하거나 사용자 승인만으로 dependency를 우회하지 않고 `P2G-40`으로
반환한다.

다음 조건에서는 active goal-run도 전체 자동 진행을 중단한다.

- material policy, Observable Policy Acceptance 또는 사용자-visible 의미 결정이 필요함
- Goal scope, Work Package, Atomic Group, Phase dependency, mapping 또는 execution boundary 변경이 필요함
- exact upstream basis가 stale하거나 서로 모순됨
- required external consumer/evidence, 배포·production·외부 환경 권한 문제가 shared prerequisite나 여러
  Phase에 영향을 줌
- existing dirty work와 Goal Set의 남은 mutation을 전반적으로 안전하게 분리할 수 없음
- root target이나 다음 execution target을 current artifact에서 모호하지 않게 정할 수 없음

run-global 중단은 authorization을 확장하거나 다른 단계를 대신 실행하지 않는다. 현재 evidence, 수행한 안전한
작업, 미완료 이유와 필요한 사용자 결정 또는 owner를 보고한다. Phase-local 보류도 해당 Phase의 dependent
Phase나 GEX-30을 건너뛰는 권한을 만들지 않는다.

### 6.7 감사 분리

GEX-20과 GEX-30은 구현이 끝나고 감사 대상이 안정된 뒤 시작하는 fresh pass다. `stage-run`에서는 새 사용자
요청이 필요하고, active `goal-run`에서는 같은 상위 요청 아래 자동 시작할 수 있다. fresh pass는 새 사용자
요청 자체가 아니라 구현 pass와 작업 문맥, mutation 권한, 산출물과 감사 기록이 분리된 별도 검토 pass를
뜻한다. 감사자는 Goal 문서의 완료 주장을 그대로 신뢰하지 않고 current code/data/document와 validation
evidence를 다시 확인한다.

감사 pass에서는 source, production code/data, audit target 또는 candidate를 수정하지 않는다. 테스트나
read-only probe가 생성한 임시 산출물은 증거로 분류하고 자신이 만든 임시 부산물만 정리할 수 있다. 결함을
발견하면 verdict와 반환 단계를 기록하고 수정은 후속 GEX-10 pass에서 수행한다.

`COMPLETION_AUDIT.md`와 감사 evidence 자체를 작성하는 것은 허용된다. 이 감사 문서 변경은 implementation
audit target과 분리해 귀속하며, 감사 중 발견한 source/data 결함을 함께 고치는 수단으로 사용하지 않는다.

다른 사람이나 별도 agent가 항상 필요한 것은 아니다. 그러나 Goal, 사용자 또는 risk 근거가 독립 reviewer를
요구하면 구현에 참여하지 않은 reviewer가 감사해야 한다. 같은 실행자가 감사할 수 있는 경우에도 구현 pass와
시간·작업 문맥·산출물이 분리된 fresh review여야 한다.

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

P2G-60 Handoff를 인수하고 이번 GEX-00 pass에서 다룰 Single/Phase Goal 하나를 선택한다. 현재 tree, dependency,
dirty work와 검증 가능성을 확인해 실행 문서를 준비하지만 production implementation은 수행하지 않는다.

#### 입력

- current P2G-60 Handoff와 exact Final Policy Freeze, Phase Plan, Goal Set Ref
- Handoff의 Remaining Gap/Blocker Summary와 P2G Scope/Authorization Boundary
- `stage-run | goal-run`, root execution target과 exact authorization 범위
- 선택을 요청한 Single/Phase Goal 또는 선택 기준
- 해당 Goal member의 Phase, execution node, scope, dependency, validation과 completion conditions
- applicable Goal Quality Contract version, Goal-local invariant/structural completion과 Validation Dimension Matrix
- applicable Phase Plan Integration Validation Ownership Map과 Master integration validation mapping
- current repository/workspace와 접근 가능한 source/data/test/consumer
- existing execution-document path 후보와 기존 실행 문서가 있으면 그 current 기록

#### 진입 조건

- 최초 실행이면 Handoff와 upstream prerequisite chain이 current이고 exact함. 후속 Phase이면 같은 Handoff와
  current predecessor Phase audit set이 section 6.1의 실행 basis를 이룸
- Goal member가 Goal Set에 존재하고 담당 Phase 및 execution node가 Phase Plan과 일치함
- predecessor/dependency Phase가 필요한 `Complete / High` 감사를 가짐
- predecessor audit이 section 5의 currentness 판정을 통과함
- applicable Master integration acceptance마다 existing/audit-local read-only seam 또는 이를 구현할 exact Phase
  Goal과 Final Execution Node가 식별됨
- 해당 Phase를 선택할 수 없는 blocker가 이미 알려져 있지 않음

#### 주요 활동

1. Handoff에서 exact Goal Set과 Goal member를 확인한다.
2. 한 Single/Phase Goal만 선택한다. 사용자가 exact Goal을 지정하지 않았고 eligible Goal이 정확히 하나이면
   그것을 선택할 수 있다. 서로 독립적인 eligible Goal이 둘 이상이면 `stage-run`에서는 사용자에게 선택을
   요청하고, Goal Set `goal-run`에서는 section 6.6의 scheduling tie-break를 따른다. Master Goal을
   implementation target으로 선택하지 않는다.
3. Phase dependency와 이미 완료된 감사의 Audit Target Record/currentness를 확인한다. 병렬 가능성은 Phase
   Plan에서 가져오며 새로 재판단하지 않는다.
4. section 5의 규칙으로 exact execution-document directory를 먼저 고정한다. 같은 exact Goal의 실행 문서나
   history가 전혀 없는 신규 set으로 판정되면 감사 전 필요한 `PLAN.md`, `EXPERIMENTS.md`,
   `EXPERIMENT_NOTES.md`를 초기화한다. 하나라도 있으면 `기존 Goal 실행 문서 재개 선독 Gate`를 먼저 적용하고,
   안전한 existing set일 때만 history를 보존하면서 current selection/baseline을 보완한다.
5. 선독 또는 신규 초기화가 끝난 뒤 Goal 범위에 필요한 current code/data/document 위치, repository status,
   relevant dirty overlap과 runner 접근 가능성을 비례적으로 확인한다.
6. applicable Master integration acceptance의 validation seam disposition, locator/evidence contract 또는 owning
   Phase Goal/Final Execution Node와 availability prerequisite를 확인한다. durable seam owner가 Phase Plan에
   없으면 P2G-40, Phase Plan mapping은 있지만 Goal validation/completion mapping만 누락됐으면 P2G-50으로
   반환한다.
7. `PLAN.md`에 run mode, root target, exact authorization, execution-document path set, upstream refs, 선택 Goal,
   scope, baseline, applicable Goal Quality Contract와 Validation Dimension disposition, structural completion,
   validation·completion·stop condition을 기록한다.
8. 이 pass에서는 구현을 수행하지 않는다. `stage-run`이면 GEX-00 결과와 GEX-10에 새 사용자 요청이 필요함을
   보고하고, active `goal-run`이면 GEX-00 결과를 기록한 뒤 section 6.6에 따라 GEX-10 pass를 시작한다.

#### 산출물

- exact selected `(Goal Set Ref, Goal ID)`와 Phase ID
- current run mode, root target과 authorization 범위
- exact execution-document directory와 document path set
- current execution baseline과 dirty/collision summary
- initialized 또는 current Goal execution document set
- applicable integration validation seam readiness와 ownership summary
- GEX-10 진입 prerequisite와 알려진 blocker

#### 종료 조건

- 실행 대상이 Single/Phase Goal 하나로 확정됨
- exact upstream chain과 dependency eligibility가 확인됨
- 현재 scope, baseline, validation, completion과 stop condition을 실행자가 식별할 수 있음
- applicable Goal Quality Contract, Goal-local invariant, structural completion과 required/N/A validation
  dimension을 식별할 수 있고 근거 없는 N/A 또는 happy-path 대체가 없음
- applicable integration acceptance의 validation seam과 구현 owner가 누락 없이 식별됨
- upstream 변경이 필요한 모순이나 unresolved blocker가 없음
- 이 GEX-00 pass에서 production implementation이 발생하지 않음

#### 중단과 반환

Handoff/Goal/Phase가 stale하거나 모순되면 section 8의 owner로 반환한다. dirty overlap, runner 또는 external
evidence 문제로 현재 실행 가능성을 판단할 수 없으면 `Blocked`로 보고하고 영향 범위를 판정한다. active
goal-run도 selected Phase의 prerequisite를 건너뛰지 않으며, section 6.6을 충족하는 독립 eligible Phase만
별도 GEX-00 pass로 선택할 수 있다.

<!-- GEX-00 END -->

<!-- GEX-10 START -->
### GEX-10 — 선택된 Goal 구현과 검증

#### 목적

GEX-00에서 선택한 Single/Phase Goal을 exact scope 안에서 end-to-end로 구현하고, 별도 완료 감사가 재검토할 수
있는 current implementation과 validation evidence를 만든다.

#### 입력

- successful GEX-00 결과와 current `PLAN.md`
- exact Handoff, Goal Set, selected Goal member와 Phase
- current repository/workspace와 dirty baseline
- Goal의 implementation boundary, validation 계획, completion과 stop conditions
- applicable Goal Quality Contract, Goal-local invariant/structural completion과 Validation Dimension Matrix
- 사용자의 direct GEX-10 `stage-run` 요청 또는 selected Goal을 포함하는 active `goal-run` authorization

#### 진입 조건

- selected Goal과 section 6.1의 execution basis가 여전히 유효함
- Goal dependency와 execution prerequisite가 충족됨
- Goal 범위와 현재 dirty work를 구분할 수 있음
- completion을 막는 미결 policy/evidence blocker가 없음

#### 주요 활동

1. mutation 전에 selected Goal, direct GEX-10 요청 또는 exact goal-run authorization과 승인 범위, current
   status/diff와 관련 prerequisite drift를 다시 확인하고 실행 기록에 남긴다.
2. current behavior, authority, producer-consumer, data/schema/store/transport와 validation seam을 필요한 만큼
   조사한다.
3. Goal의 assigned execution node를 유효한 중간 상태가 깨지지 않도록 구현한다. Goal 밖 cleanup이나 미래
   Phase 작업을 함께 수행하지 않으며 section 6.4의 Implementation Quality Guardrails를 source-level
   completion condition으로 적용한다.
4. 필요한 test, data, fixture, probe와 durable contract 문서를 구현과 함께 정렬한다. selected Goal에 durable
   integration validation seam이 배정됐으면 같은 assigned execution node 범위에서 구현·검증한다.
5. Validation Dimension Matrix의 `required` row와 current actual에서 발견한 applicable dimension을 evidence
   map에 연결하고 focused loop와 위험에 비례한 broader validation을 수행한다. 명령, 실제 결과,
   실패·수정·재검증과 생략을 `EXPERIMENTS.md`에 기록한다.
6. 기술 판단, evidence gap, 새 정책 질문과 out-of-scope 후속 후보를 `EXPERIMENT_NOTES.md`에 정리한다.
7. completion conditions를 current actual과 validation evidence에 대조한다.
8. 이번 implementation delta가 predecessor Phase audit의 impact surface를 바꿨는지 비교해 currentness를
   판정한다. expected delta로 audit만 `stale`해졌고 correction은 필요 없으면 그 사실과 re-audit prerequisite를
   기록한다.
9. section 5의 Audit Target Record로 감사할 exact current tree/scoped diff, external actual과 evidence set을
   고정하고 구현 수정을 멈춘다.
10. Phase 완료를 스스로 선언하지 않는다. `stage-run`이면 GEX-20에 새 사용자 요청이 필요함을 보고하고, active
   `goal-run`이면 구현 mutation을 멈춘 뒤 section 6.6에 따라 fresh GEX-20 pass를 시작한다.

#### 산출물

- selected Goal 범위의 implementation delta
- current test/data/document/probe와 필요한 legacy removal 결과
- 실패와 재검증을 포함한 execution/validation evidence
- exact Audit Target Record와 known omission/limitation
- predecessor audit currentness/continuity 결과와 필요한 re-audit prerequisite
- updated `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`

#### 종료 조건

- 모든 assigned execution node와 applicable implementation boundary가 구현됨
- completion condition마다 current implementation 또는 validation evidence를 제시할 수 있음
- Implementation Quality Guardrails 위반이 없고, applicable structural completion/legacy removal condition이
  source와 current consumer에서 충족됨
- 모든 required validation dimension이 direct evidence를 가지며, N/A disposition이 current actual에서
  타당하고 happy-path-only evidence가 applicable negative/edge dimension을 대신하지 않음
- required focused와 proportional broad validation이 pass하거나, baseline/omission disposition이 완료 주장을
  침범하지 않음
- unresolved material policy, required consumer/evidence gap 또는 unsafe dirty overlap이 없음
- Audit Target Record가 complete하고 감사 대상이 안정되어 있으며 이후 source/data/evidence 수정은 새
  GEX-10 pass 또는 audit currentness 재판정으로 구분할 수 있음
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
- `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
- GEX-10의 exact Audit Target Record와 scoped diff
- current code/data/document와 validation evidence
- applicable reviewer requirement
- 사용자의 direct GEX-20 `stage-run` 요청 또는 selected Goal을 포함하는 active `goal-run` authorization

#### 진입 조건

- GEX-10 종료 조건이 충족되고 audit target이 식별됨
- initial audit이면 GEX-10 Audit Target Record가 `current / unchanged | current / proven non-impacting delta`임.
  stale predecessor의 fresh re-audit이면 section 5의 허용 조건에 따라 current actual에서 새 Audit Target Record를
  고정할 수 있음
- 감사자가 구현 pass와 분리된 fresh review를 수행할 수 있음

#### 주요 활동

1. initial audit이면 exact Audit Target Record를 current actual과 다시 비교하고 currentness 판정, changed
   member와 non-impact evidence를 기록한다. old record가 `stale`이면 그것을 current target인 것처럼 통과시키지
   않는다. 허용된 fresh re-audit이면 current actual에서 새 Audit Target Record를 먼저 고정하고 predecessor
   record, delta identity와 owner를 기록한다.
2. Goal의 objective, scope, assigned execution node, policy/decision/acceptance coverage, applicable Goal Quality
   Contract, Goal-local invariant/structural completion, validation과 completion conditions를 다시 읽는다.
3. 각 completion condition을 current actual, scoped diff와 validation evidence에 직접 대조한다.
4. applicable한 runtime/state, data/schema/store, transport/DTO, producer-consumer, migration/compatibility,
   legacy/fallback, durable document와 dirty attribution을 확인한다.
5. selected Goal에 durable integration validation seam 구현이 배정됐으면 current target에 존재하고 Master가
   사용할 locator, invocation, fixture/input과 evidence contract가 completion condition을 충족하는지 확인한다.
6. scoped diff와 current production path를 직접 읽어 test/fixture-specific behavior, silent normalization,
   broad catch-all recovery, consumer-side 재추론, 미래 전용 abstraction, arbitrary timing/sequence workaround,
   의미 없는 snapshot 갱신, duplicate authority와 승인되지 않은 legacy/fallback이 없는지 확인한다. Test
   결과만으로 이 source-level 판정을 대신하지 않는다.
7. 각 Validation Dimension의 required evidence와 N/A 근거를 current actual에 대조하고 중요한 focused/broad
   validation이 실제 대상을 non-zero로 실행했는지 확인한다. 필요하면 source를 수정하지 않는 범위에서
   validation을 다시 실행한다.
8. omitted/inaccessible/unknown evidence와 baseline failure가 완료 주장을 침범하는지 판정한다.
9. `single_phase`이면 별도 GEX-30이 없으므로 Goal Set 전체 acceptance와 broad regression도 이 감사에서
   확인한다.
10. `COMPLETION_AUDIT.md`에 Audit Target Record, condition별 evidence와 `Complete / High | Not Complete |
   Blocked` verdict를 기록한다.
11. 결함이 있으면 수정하지 않고 exact finding과 반환 owner를 보고한다.

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
  없음
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
- `Not Complete` 또는 `Blocked`이면 dependent Phase와 GEX-30을 허용하지 않는다. active goal-run의 in-scope
  기술 결함은 audit pass 종료 뒤 correction GEX-10으로 반환할 수 있다. 더 진행할 수 없는 Phase-local
  finding이면 section 6.6의 조건을 충족하는 독립 eligible Phase만 진행할 수 있다.

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
- cross-Phase invariant, integration acceptance, Integration Validation Ownership Map과 applicable reviewer requirement
- 사용자의 direct GEX-30 `stage-run` 요청 또는 Goal Set을 대상으로 하는 active `goal-run` authorization

#### 진입 조건

- Goal Set의 모든 Phase가 section 5의 currentness 판정을 통과한 `Complete / High` 감사를 가짐
- final audit target과 section 6.1의 execution basis가 유효하고 서로 일치함
- Phase 완료 이후 통합 의미를 바꾸는 미감사 수정이 없음
- 모든 integration acceptance에 existing/audit-local read-only seam 또는 구현·감사 완료된 owning Phase
  Goal/Final Execution Node가 있음
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
6. 모든 Phase의 applicable Validation Dimension coverage와 N/A 근거를 final actual에서 연결하고,
   Phase-local happy path test만으로 드러나지 않는 cross-Phase와 broad integrated validation을 확인하거나
   read-only로 재실행한다.
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

국소 기술 결함이면 영향받은 Single/Phase Goal의 correction GEX-10과 새 GEX-20 감사가 필요하다. 기존 감사
기록은 삭제하거나 성공으로 덮어쓰지 않고 history로 보존한다. policy/Phase/Goal 구조 문제이면 section 8의
P2G owner로 반환한다. 필요한 durable integration validation seam의 Final Execution Node/Phase owner가 없으면
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
| 정책·Phase·Goal 의미가 그대로인 selected Goal 내부 기술 결함 | `GEX-10` |
| implementation defect 없이 target delta나 evidence expiry로 이전 Phase audit만 `stale`해짐 | `GEX-20` fresh audit |
| 구현은 그대로이고 누락된 감사 evidence만 새로 이용 가능해짐 | `GEX-20`, Master이면 `GEX-30` fresh audit |
| final cross-Phase 기술 결함 | 영향 Goal의 `GEX-10`과 `GEX-20`, 이후 `GEX-30` |

P2G owner로의 반환을 보고하는 것은 그 P2G 단계를 실행하거나 승인하는 것이 아니다. 발견 evidence, 완료되지
않은 이유, 영향 범위와 권장 owner를 보고하고 현재 run을 중단한다. GEX 단계로의 기술적 반환은 현재 pass 안에서
수정한다는 뜻이 아니다. `stage-run`은 새 사용자 요청을 기다리고, active `goal-run`만 section 6.6에 따라 새
correction 또는 audit pass를 시작할 수 있다.

## 9. 기존 자료에서 유지한 원칙과 제외한 구조

이 문서는 기존 Phase/Master Goal의 실제 운용에서 다음 원칙을 유지한다.

- `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, `COMPLETION_AUDIT.md`의 역할 분리
- 구현 전에 current tree와 dirty baseline을 다시 확인
- Phase 구현 뒤 별도의 독립 완료 감사 수행
- Phase 감사 통과 전 dependent Phase를 실행하지 않으며 독립 Phase는 scoped blocker/currentness 계약을 따름
- active goal-run에서도 GEX 단계별 pass와 mutation/read-only 권한을 분리
- 모든 Phase 완료 뒤 Master final actual을 다시 감사
- Master가 Phase 구현 세부사항을 복제하거나 직접 구현하지 않음
- exact validation result, baseline failure, omission, consumer와 legacy evidence를 완료 주장에 연결
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
