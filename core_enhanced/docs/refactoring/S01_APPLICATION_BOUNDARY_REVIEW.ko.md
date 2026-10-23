# S01 애플리케이션 실행 경계 조사와 개선 계획

작성·갱신일: 2026-10-16

**현재 위치:** 대표 경계 조사와 아래 §7의 D1 교정·완료 검토를 마쳤다. 후속 [S04 조사·계획](S04_PERSISTENCE_BOUNDARY_REVIEW.ko.md)도 마쳤으며 최신 재개 위치는 [README](README.md)를 따른다.
§1–6은 조사 당시의 범위·판정·계획·실행 근거를 보존한다.

## 1 범위와 조사 방법

[출발점](README.md)의 다음 행동에 따라 일반 명령, host tick, 거절·취소, prepare/commit,
오류 격리와 결과 공개의 대표 경로를 조사했다. 종료점은 조사·후보 판정·개선 계획이며 제품 구현은 포함하지 않는다.
S07에서 확인한 전투 정산·Continue와 호스트 승인 연결은 재사용했다. 모든 action의 도메인 처리,
저장 codec·파일 복구 전체, Unity, 전투 커널 전체를 감사한 것은 아니다.

판단 순서는 계약과 코드의 생산자→소비자 추적, 의심 지점의 기존 테스트 확인, 실제 호출 경계 재대조다.
테스트는 현재 동작과 보호 범위의 증거이며 정상 의미를 결정하는 계약으로 사용하지 않았다.
직접 실행의 commit 후 결과 변환을 코드에서 확인한 뒤 취소 테스트와 `TrialState`의 외부 사본 경계를 대조했다.
문서·코드·테스트가 다르면 차이를 남기며 테스트의 기대값만으로 코드 오류 또는 정상 동작을 확정하지 않는다.

주요 입력은 [선택 파일 기준선](evidence/s01_001/baseline.json)에 기록했다. 전체 저장소를 다시 복제하거나
해시하지 않았다. 이전에 확인한 README·S07 결과를 기준으로 착수했다.

## 2 실제 실행과 상태 소유자

소스는 [공개 application](../../src/application.rs), [도메인 executor](../../src/game/world/application.rs),
[behavior dispatch](../../src/game/world.rs), [fault owner](../../src/application/fault.rs),
[action 자격 판정](../../src/game/world/action_capability.rs)을 기준으로 한다.

| 흐름 | 생산·검증·변경·공개 순서 | 실패와 다음 소유자 |
| --- | --- | --- |
| 명령 decode | `ApplicationCommand::from_document/try_new`가 unknown·malformed·kind 불일치를 검사한 뒤 private behavior를 보유 | app 접근 전 입력 거절. 공개 command가 임의 mutable domain state를 제공하지 않음 |
| 일반 명령 준비 | fault guard → 시도 revision 증가 → `prepare_with_transport_context` → current context·resource 검증 → complete draft → authority/cursor/action gate → 도메인 처리 → 후처리·context 검증 | domain 오류는 draft 폐기. 이미 게시된 전투의 fatal 오류는 별도 fault owner가 최초 원인을 보존 |
| 공개 결과 준비 | `ApplicationResult::from_behavior` → draft checkpoint export·미리 capture한 profile 인수 → `PreparedApplication` | 결과·checkpoint 준비 실패는 아직 commit되지 않은 draft를 폐기. prepared는 원래 app에 배타 참조를 가짐 |
| 준비 객체 폐기/commit | drop은 draft 폐기. `commit(self)`는 도메인을 재실행하지 않고 `commit_application`에서 root를 한 번 대체 | 결과·저장 자료는 readonly. 소비한 객체를 다시 commit할 수 없음. fault·시도 revision은 gameplay draft와 다른 소유자 |
| host tick | `prepare_active_battle_for_host_tick`가 같은 executor·publication 경계를 사용. 결과는 `Option<ApplicationResult>` | 진입 전·실행 오류는 원본 gameplay root 보존. 일반 tick의 None과 terminal 결과를 구분 |
| 직접 실행 | `execute_internal`/직접 advance는 GameCore prepare→commit 뒤 `ApplicationResult::from_behavior` 호출 | ordinary projection 오류는 recovery-pending으로 표현. private control 취소는 직접 app의 commit 뒤에도 전파될 수 있음. 아래 D1 참조 |
| snapshot/reconnect | `snapshot(&self)`의 readonly projection, reconnect의 별도 readonly 결과 경로 | 조회 오류는 fatal battle로 격리하지 않음. fault 이후 snapshot은 허용하되 Run의 action·deployment 광고를 닫음 |
| Query command | query로 분류된 behavior도 command executor의 draft 경계를 통과. action descriptor가 mutation/fault 판단을 구분 | `&self` snapshot과 동일한 구현 경로라고 간주하지 않음. 모든 query의 domain 내부 무변경성·비용은 이번에 전수 검증하지 않음 |
| profile/admin | guard와 시도 revision, 각 owner의 application route. profile 입력 decode와 domain 오류는 일반 오류로 변환 | 기존 battle fault 이후 mutation은 차단. 정상 profile/admin 입력 거절을 새 fatal battle로 분류하지 않음 |
| persistence rollback | app identity와 시도 revision을 가진 불투명 단회 token. active battle 밖에서만 capture/restore | 다른 app·지나간/중복 token 거절, profile 단조 정보 병합, revision 포화 시 rollback만 닫음 |
| 제작 세션 | `TrialState::prepare_step`가 application fork에서 직접 실행·필수 receipt를 완성하고, `advance`가 control scope 성공 뒤 aggregate 교체 | 내부 app이 commit됐어도 바깥 candidate가 실패하면 마지막 accepted session·receipt·usage 유지 |

`ApplicationRoute → ApplicationOwner`는 exhaustive routing이다. `ActionKind::descriptor`의 host/common/battle
분류, owner dispatch, capability 표시 순서는 서로 다른 책임이다. capability projection과 실행 gate는
`evaluate_common_action` 계열을 공유한다. 순서 배열의 존재만으로 정책 중복이라 판정하지 않는다.
현재 domain action 87개와 공개 command의 일대일 대응은 기존 실행 테스트로 확인한다.

`GameCore`의 state·seed·retained generation은 complete draft가 함께 소유한다. 반면
`CoreApplication`의 transaction revision은 시도마다 증가하고 최초 battle fault는 실패 후에도 남는다.
따라서 “실패하면 전체 CoreApplication 객체가 불변”이라고 기술하면 틀린다.
StartNewGame의 candidate generation은 기존 자격 gate를 통과한 draft에만 설치된다.
이번 실행에서는 별도의 A/B generation 교체 fixture를 재실행하지 않았다.

## 3 실제 외부 소비 경계

[서버 handler](../../../game_server/src/game/player_game_actor/handlers.rs)는 query·Continue·일반 변경을 분기한다.
active/profile persistence가 있는 일반 변경은 prepared command를 사용하고, active store가 있는 tick도 prepared
경로를 사용한다. query 또는 persistence 없는 직접 실행을 같은 저장 원자성으로 묶지 않는다.
Continue는 저장 snapshot을 읽어 Core 복원을 호출하는 별도 분기다.

`publish_prepared_application_bound`에서 approved 경로는
[ApprovedPersistence::publish](../../../game_server/src/mod_admission/persistence.rs)로 이어진다.
profile→active lock, expected revision 대조, successor profile·checkpoint 검증 후 accepted intent를 받는다.
accept 전 실패는 prepared를 버리고, accept 후에는 recovery-pending 여부와 함께 그 prepared를 한 번 commit한다.
checkpoint·profile이 같고 terminal/강제 revision이 아니면 durable IO를 생략해도 Core commit은 수행한다.
이 조사는 실제 호출과 순서에 대한 코드 확인이며 파일 실패 주입·프로세스 crash 재현 결과가 아니다.

[TrialState](../../src/authoring/trial.rs)의 `prepare_step`→`advance`를 따라가면 직접 app commit보다 바깥에
세션·receipt·usage의 acceptance 경계가 있다. 이는 [runtime 계약](../core_runtime_contract.md)의
“Finite authoring scenario input and preparation” 및 private projection interruption 설명과 일치한다.
public API의 저수준 취소 테스트와 실제 세션 보존 테스트는 서로 다른 owner를 관찰하므로 모순된 결과가 아니다.

## 4 구조 평가와 후보 판정

| 기준 | 확인 결과와 처리 |
| --- | --- |
| 책임과 도메인 경계 | 공개 decode/오류·projection, domain draft, host durable publication의 책임이 구분됨. 유지 |
| 상태 소유권 | root 단일 commit, prepared 배타 참조, root 밖 fault/revision의 수명이 확인됨. 유지; D1에 보존 범위 명시 |
| 상태 전이와 수명 | current/staged lifecycle 검증, prepared drop/consume, rollback owner/revision gate. 유지 |
| 실패와 commit | prepared와 직접 실행, 외부 Trial acceptance를 구분해야 함. 실행 구조 유지; 문서의 과도한 일반화 D1 |
| 규칙 중복 | action descriptor·owner·표시 순서는 목적이 다르고 capability/실행 자격은 공유. 일괄 registry 통합 기각 |
| 의존 방향과 공개 범위 | Core는 host 파일 I/O를 수행하지 않고 prepared 자료를 제공. 현재 조사 경로의 우회 root writer 없음 |
| 변경 용이성과 확장성 | action 추가는 닫힌 enum·mapper·owner의 exhaustive 갱신을 요구. 범용 trait 도입의 실제 이익 근거 없음 |
| 데이터와 외부 계약 | private domain→공개 document, checkpoint→host 저장 책임 구분. stale 숫자·버전 설명은 D1/S04로 분리 |
| 결정론과 계속 실행 | complete draft와 rollback 소유 관계 유지. Trial 실패 재시도 결과·receipt 동등성 확인. 저장 codec 전체 검증은 S04 |
| 오류와 관측 | 최초 fatal은 진단 변환 전 기록, 일반 거절/query와 구분. 관측 feature 실행은 이번 범위 밖; D1에 projection 취소 한계 명시 |
| 검증 가능성 | 공개 command·snapshot·prepare 경로 테스트가 존재. fatal/취소/포화는 제어된 fixture이며 자연 플레이 실패율의 증거가 아님 |
| 성능과 자원 수명 | complete clone 전 admission, retained extent 검사, publication profile의 budget 안 capture 확인. 비용 측정 없이 clone 제거·query 분리 채택하지 않음 |

### D1 — application 계약의 commit·취소 설명 정합화

- 유형/owner: 문서·주석 정합성 교정, S01. **현재 실행 코드 오류를 확정한 finding이 아니다.**
- 근거: runtime 계약 “Application Authority And Closed Lifecycle”은 모든 fallible projection이 commit 전에
  끝난다고 기술하지만 직접 `execute_internal`/advance의 공개 결과 변환은 GameCore commit 뒤다.
  “Transport-Neutral Core Public Projection”의 포괄적인 postcommit `Err` 금지도 private control interruption
  예외를 설명하지 않는다. 같은 문서의 authoring 절은 이미 interruption과 바깥 candidate 폐기를 설명한다.
  `world/application.rs::PublicationDisposition` 주석에도 적용 계층을 좁힐 필요가 있다.
- 영향: 후속 변경자가 prepared host 경로의 보장을 직접 app 전체에 적용하거나, 의도된 Trial 외부 rollback을
  중복 구현할 수 있다. 문서만으로 직접 API의 `Err`를 무조건 재시도하면 안전하다고 해석해서는 안 된다.
- 추가 정합성: 공개 projection 절의 command 58개는 현재 87개와 다르다. result/error 고정 개수와 checkpoint
  V12 문구도 현재 정의·저장 owner와 대조해야 한다. 숫자를 계속 복제하기보다 정의/소유 절을 참조한다.
  Durable 절의 journal V3/구현 V4 차이는 이미 S04 후속이며 이번에 지원 버전을 추정하지 않는다.
- 처리: 다음 작업으로 문서·관련 주석을 정합화한다. 실행 순서를 바꾸거나 새 API/저장 형식을 만들지 않는다.

기타 판단: `PreparedNonBattleApplication` alias는 이름이 좁지만 일반 prepared API도 사용한다.
호출자를 오동작시키는 증거 없이 공개 alias 삭제·rename은 채택하지 않는다. 직접/준비/관측 wrapper의
외형적 중복도 commit 시점·출력·오류 의미가 달라 이번에 통합하지 않는다. clone 비용 개선은 측정 전 보류다.

## 5 D1 구현·검증 계획과 후속

1. `core_runtime_contract.md`의 application authority/public projection 절과 관련 application 주석만 수정한다.
   domain draft, prepared public result, 직접 public projection, Trial accepted aggregate의 보존 단위를 명시한다.
   fatal capsule·attempt revision은 root rollback 밖이라는 점을 함께 기술한다.
2. 직접 경로의 private control interruption과 ordinary recovery-pending을 구분한다. 기존 취소 테스트를
   없애거나 반환 계약을 바꿔 문장에 맞추지 않는다. command/result/error 개수는 정의 참조로 바꾸고 checkpoint
   버전 설명은 저장 계약 owner로 연결한다. S04 journal 호환성 정책은 이 작업에서 변경하지 않는다.
3. 수용 조건: 각 경로의 commit 전후와 실패 보존 범위를 코드에서 한 번에 추적할 수 있고, 앞뒤 계약 절이
   모순되지 않는다. 실행 코드·직렬화·공개 signature·밸런스 변화가 없어야 한다.
4. 검증: 이번 코드/실행 근거를 재사용해 변경 문구를 코드와 대조하고 링크·diff를 검사한다. 제품 코드가
   변하지 않는 문서·주석 수정에 같은 테스트 전체를 반복하거나 새 구현 복제 테스트를 추가하지 않는다.
5. 문구 교정 뒤 구현과 시간을 분리한 완료 검토를 남기고 S04의 저장·복원 조사로 이어간다.
   S04는 `world/active_run.rs`/`active_run/continuation.rs`와 host journal writer/reader에서 시작한다.

현재 소유자·공개 실행 계약은 유지한다. D1 없이 게임 실행이 불가능한 중간 상태는 아니지만,
선택한 문서 교정을 구현 완료로 표시하지 않는다. 다음 재개 위치는 이 절의 D1이다.

## 6 실행 근거와 조사 한계

선택 테스트의 원시 출력은 [regression001.log](evidence/s01_001/regression001.log),
최종 실행 집계와 파일 불변 확인은 [verification.json](evidence/s01_001/verification.json)이 소유한다.
이번 실행은 **21개 통과·실패 0·무시 0**이다. 빌드 37.43초, 테스트 8.48초이며 기존 경고 33개가 있었다.
제품 수정 없이 기존 테스트를 실행해 관찰을 확인하며 S07/B1/R1의 과거 통과 수를 이번 수에 합산하지 않는다.

검증 묶음은 공개 decode/일대일 command mapping, snapshot·projection, 일반 거절 재시도,
prepared 취소, rollback 단회·다른 owner·포화, closed-context 거절, fatal 격리, 직접 projection 취소,
prepared command/tick의 drop·commit, 실제 제작 step의 receipt 실패·재시도를 포함한다.
일반 거절 테스트의 snapshot equality를 모든 root 내부 필드 동등성으로 확대하지 않았다.
취소 injection·내부 fixture 테스트는 지원된 상태에서 경계 실패를 관찰하는 도구이며 자연 입력 재현과 구분한다.

전체 Core 테스트, release, runtime-observation feature, 서버 파일 실패 주입/프로세스 crash,
Unity는 이번에 실행하지 않았다. 관측 feature의 wrapper는 코드상 연결만 확인했다.
일반 명령 전체, 모든 query의 내부 무변경성, A/B generation 교체와 저장 호환성 전체의 완료를 주장하지 않는다.

조사 종료 판단은 선택한 대표 흐름과 12개 기준의 적용·유지·후보·제한을 설명했는지에 한정한다.
S01 전체 무결점 또는 전체 Core 리팩토링 완료 선언이 아니다.


## 7 D1 문서·주석 교정과 완료 검토

### 변경과 보존 범위

[계약 문서](../core_runtime_contract.md)의 Application Authority / Public Projection 두 절과
[private executor](../../src/game/world/application.rs)의 `PublicationDisposition`, `CommittedApplication::publish`
주석을 교정했다. [변경 전후 diff](evidence/s01_d1_001/source.diff)와 [검토 대상](evidence/s01_d1_001/review-target.json)을 연결한다.

- GameCore의 domain draft commit과 공개 결과 변환을 구분했다. Prepared 경로의 결과·checkpoint/profile 준비,
  직접 실행의 commit 후 projection, Trial의 외부 aggregate 채택을 각각 설명했다.
- 직접 경로의 private control interruption은 commit 뒤에도 전파될 수 있으며, ordinary recovery-pending과
  다른 의미임을 명시했다. Generation 교체의 실패 보존 설명도 domain 준비 실패 범위로 한정했다.
- Gameplay rollback 밖의 시도 revision·최초 fault capsule, fatal 이후 재시도 차단을 설명했다.
- Host의 새 intent 수락 후 commit과 저장 내용이 같아 IO를 생략하는 commit을 구분했다.
- Command/result/error의 낡은 고정 개수는 현재 enum 정의·목록을 참조하도록 바꿨다. Checkpoint의 중복 V12 문구는
  저장 형식 소유 절로 연결했다. Journal V3/V4 차이나 decoder·migration 정책은 이번에 수정하지 않았다.

실행 코드·공개 signature·저장 bytes·밸런스는 바꾸지 않았다. Rust 변경 줄이 모두 주석임을 기계적으로 확인했고,
계약 변경은 위 두 절 안에만 있음을 확인했다. 다른 S01 기준선 입력은 그대로다.

### 검증과 분리된 완료 검토

제품 편집을 끝낸 뒤 확정 diff를 대상으로 기존 코드 추적 근거와 대조했다. 이 검토는 같은 작업자의
별도 시점 완료 검토이며 외부 독립 감사가 아니다. 검토 중 generation 실패의 범위와 host IO 생략 설명을
더 좁힌 뒤 최종 대상에서 다시 확인했다.

| 확인 항목 | 결과 |
| --- | --- |
| Prepared 결과·저장 자료 준비가 commit 전이라는 설명 | `prepare_command_with_generation`, `prepare_active_battle_for_host_tick`, `prepare_publication` 및 `PreparedApplication::commit`과 일치 |
| 직접 commit 이후 projection과 취소 한계 | `execute_internal`/직접 advance/Continue, `ApplicationResult::from_behavior`, `check_projection_control`의 기존 순서와 일치 |
| Trial에서 마지막 accepted aggregate 유지 | `prepare_step`의 fork와 `advance`의 control 성공 후 교체에 한정해 설명 |
| Root 밖 fault/revision과 단회 소유권 | 기존 guard·시도 revision·prepared consume 의미를 보존하고 전체 객체 불변으로 오해할 문구 제거 |
| Host publication | 기존 accepted intent와 동일 checkpoint/profile의 IO 생략 경로를 구분; 파일 crash 복구 통과로 확대하지 않음 |
| 변경 범위 | 선택 계약 두 절·Rust 주석만 변경. S04 저장 정책과 실행 코드 변화 없음 |

S01의 **21개 통과**는 이전 실행 근거로 재사용했다. 이번 D1에서 테스트를 다시 실행하거나 새 통과 수로
집계하지 않았다. 문서·주석만 바뀌었고 나머지 조사 입력이 같아 재실행이 필요한 의미 변화가 없었다.
링크·문서 형식·diff 검사 결과와 재사용 범위는 [검증 기록](evidence/s01_d1_001/verification.json)에 남겼다.
전체 Core·release·관측 feature·호스트 crash·Unity에 대한 추가 검증 주장도 없다.

### 종료와 다음 재개 위치

D1의 교정·검증·완료 검토를 마쳤다. 선택 작업의 필수 미해결은 없다.
다음은 **S04 저장·복원 대표 흐름 상세 조사와 개선 계획**이다. 다음 작업의 종료점은 조사·후보 판정·계획이며,
현재 기록만으로 journal 버전 교정이나 저장 정책 변경을 선행 구현하지 않는다.

- Core `world/active_run.rs`와 `active_run/continuation.rs`의 export/import → Continue와 명시적 SavePoint load를 추적한다.
- Host `player_profile_store/application_intent.rs`의 writer/reader와 approved persistence 소비자를 연결해
  계약의 journal V3와 구현 V4 차이, 실제 지원·거절·재적용 범위를 확인한다.
- S01의 준비 객체·accepted intent 연결 및 S07의 비용 전/후 snapshot·보상 continuation 근거는 재사용한다.
  실제 codec·reader 차이, 저장 후 재개와 필요한 실패 경계를 추가 확인하고 관련 검증을 선정한다.
- 프로세스 crash나 모든 과거 저장 호환성을 실행하지 않았다면 그 한계를 명시한다.
