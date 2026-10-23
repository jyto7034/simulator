# S04 저장·복원 경계 조사와 개선 계획

작성·갱신일: 2026-10-16

**현재 위치:** 대표 조사·계획과 아래 §8의 S04 D1 교정·완료 검토를 마쳤다. 후속 [S03/S06 조사](S03_S06_NODE_ACTIVITY_REVIEW.ko.md)를 진행했으며 최신 다음 행동은 [출발점](README.md)을 따른다.
§1–7은 조사 당시의 판정·계획·실행 근거를 보존한다.

## 1 범위와 기준

[README](README.md)의 다음 작업에 따라 Continue, 명시적 SavePoint load, profile와 active-run의 paired
publication, accepted journal 재적용을 대표 흐름으로 조사했다. 이번 종료점은 조사·후보 판정·개선 계획이다.
제품 코드·저장 정책은 변경하지 않았다. S01의 prepared/commit·host 승인 경계와 S07의 결과·보상·후퇴
복원 근거를 재사용하고 실제 codec·writer/reader·복구 순서를 추가 확인했다.

게임 의미는 [룰북의 손실·복구](../game_rulebook.md#손실을-관리하고-다음-진행을-정한다)와
[저장과 복구 규칙](../game_rules/results-and-save.md#저장과-복구)을 기준으로 한다.
미완료 노드는 입장 전으로 되돌리고 완료 결과는 유지하며, Continue는 SavePoint 사용 횟수를 소비하지 않는다.
명시적 SavePoint load는 런당 최대 3회이고 사용량은 재시작·백업으로 감소하지 않아야 한다.
기술 계약은 [runtime 계약](../core_runtime_contract.md)의 Durable active-run / Permanent Player Profile 절과
현재 형식을 선언하는 U15/U17 설명을 함께 대조했다. 테스트의 기대값으로 이 의미를 대신하지 않았다.

## 2 형식과 소유권

| 경계 | 현재 구현과 생산·소비 | 판정 |
| --- | --- | --- |
| Core Continue | `active_run/continuation.rs::VERSION = 17`; `seal_document` → `restore_continue_document`/`inspect_requirements` | 현재 reader는 V17만 수용. checksum·content identity·current/SavePoint 관계를 검사 |
| 독립 checkpoint 표현 | `active_run.rs`의 private `encode_active_run_checkpoint` 상수는 17. root는 `checkpoint`, Continue는 `continuation` | 현재 `src`/`tests`에서 encoder 호출자는 찾지 못함. 명시적 SavePoint load는 보유한 `RunCheckpointPayload`를 사용하며 이 bytes의 Continue import가 아님 |
| Host active-run 봉투 | `ActiveRunStore` V7, opaque Core bytes·loads/peak·revision·required configuration | 낮은 계층의 `core_version`은 12/14/15/16/17을 운반할 수 있으나 게임 Continue 지원을 뜻하지 않음 |
| Approved host admission | `ApprovedStoreGate` → `check_record` → Core readonly requirements inspection | 현재 Core 형식·identity와 승인 configuration을 확인. low-level store에서 opaque bytes를 reopen한 것과 구분 |
| Permanent profile | `PlayerProfileEnvelope` V6 writer/reader, `validate_persisted`와 checksum | 다른 profile version을 빈 지식으로 자동 읽는 fallback 없음 |
| Paired application journal | `ApplicationIntent` V4, `IntentEnvelope::encode/decode`, exact old/new active·successor profile·configuration | reader는 다른 version을 `ProtectedOriginal`로 거절. V1 구조 선언이 있어도 decode fallback으로 사용하지 않음 |

근거: [Core export](../../src/game/world/active_run.rs), [Continue codec](../../src/game/world/active_run/continuation.rs),
[SavePoint 전이](../../src/game/world/node_flow.rs), [host active store](../../../game_server/src/persistence/active_run_store.rs),
[profile store](../../../game_server/src/persistence/player_profile_store.rs),
[journal](../../../game_server/src/persistence/player_profile_store/application_intent.rs),
[approved requirements](../../../game_server/src/persistence/mod_requirements.rs).

`ContinueState`는 immutable payload/encoded Arc와 unfinished node 표식을 소유한다. Export는 확정 cache를
가져오며 live battle을 새로 직렬화하지 않는다. Current와 SavePoint는 같은 payload 타입을 사용하되 소유 상태는
구분한다. 동일하면 wire의 `current` 참조로 한 번 인코딩하고, import는 독립 SavePoint를 materialize한다.
두 상태의 seed·instance namespace와 content identity를 확인한 뒤 설치한다.

Profile 전체를 Continue에 넣지는 않지만 원정의 `ExpeditionInstance`에는 player ID·sequence·UUID가 포함된다.
`restore_continue_document`는 저장된 원정 player ID와 호출자 ID를 비교한다. `validate_run_checkpoint_payload_with_seed`는
원정 profile의 active marker와 저장 instance의 일치·vault ownership도 검사한다. 따라서 전송 session/root player를
복원 때 주입한다는 사실을 “저장 graph에 player identity가 전혀 없다”로 확장하면 안 된다.

## 3 실제 복원 흐름

| 흐름 | 검증·변경·확정 순서 | 보존/실패 의미 |
| --- | --- | --- |
| 노드 진입 전 | `prepare_continue_before_operation` 또는 원정 입장 capture → immutable payload 설치 | 미완료 노드가 진행되는 동안 기존 확정 cache 유지. S07의 수동 후퇴 비용 후 snapshot과 별개 |
| 완료 이후 | `refresh_continue_after_operation`가 stable phase를 capture. terminal이며 pending reward가 없으면 cache 제거 | 승패 확정 결과·필수 선택 보존. 일반 후퇴는 입장 전 cache에 성공한 후퇴 횟수만 반영 |
| Continue | 빈 run·load 한도 검사 → allocation admission·V17/checksum·content/graph/phase/ownership 검증 → seed/player/UUID/current/SavePoint·result/VN 설치 → application commit | 실패는 S01 draft 경계에서 폐기. `loads_used`는 전달된 값을 유지하며 증가하지 않음. 기존 run 위에 중첩 복원하지 않음 |
| SavePoint load | 허용 phase·payload·현재 seed/instance·load 한도 검사 → payload 복원 → loads +1 → map 결과 → application commit | 세 번째 성공까지 허용. Continue와 달리 기존 run seed/instance도 일치해야 함. 후퇴 ledger 등 checkpoint 시점의 run 상태로 rewind |
| 자원과 profile | `RunCheckpointPayload::restore_into_state`가 run assets·RNG를 복원하고 warehouse debit 차이를 reconcile | profile을 통째로 과거 사본으로 대체하지 않음. 기존 지식과 peak 보존, warehouse 소비는 checkpoint와 함께 조정 |
| 저장 cache 재사용 | ActiveRunContinue 후 재capture 생략, 조회·live battle 동안 pre-entry cache 공유 | 게임 상태와 저장 bytes를 독립적으로 수정하는 두 writer를 만들지 않음. 성능 개선 필요성은 이번에 측정하지 않음 |

복원 함수는 [state payload](../../src/game/world/state.rs)의 `restore_into_state`로 연결된다.
RNG 복원은 보존 대상 VN session counter를 구분하고, pending node/battle/result를 지운 뒤 Continue가 확정된
result/reward/VN을 재설치한다. 세부 아이템·유물 graph validator 전체를 다시 감사한 것은 아니다.

## 4 파일 게시·재시작 경계

[ApprovedPersistence::open_roots](../../../game_server/src/mod_admission/persistence.rs)는 readonly gate를 먼저
구성하고 store를 열어 profile bootstrap과 journal recovery를 수행한다. `preflight_approved`는 unsupported
profile/journal, 승인 불일치 active record를 보호한다. Accepted A→B의 old/new record 예외도 실제 원본이
그 exact 두 값 중 하나인지 확인한다. 이 연결은 코드 조사이며 모든 승인 세대 조합을 이번에 실행한 것은 아니다.

`ActiveRunStore::snapshot`은 이름과 달리 유효 backup으로 current 파일을 복구할 수 있다. Readonly admission과
동일한 조회로 취급하지 않는다. 원본 버전 보호가 먼저 수행되며 새 revision은 tombstone에도 유지된다.
동일 revision의 다른 payload는 거절하고 expected revision이 맞지 않는 새 게시도 거절한다.

일반 publication은 profile→active lock과 expected revision 확인 후 다음 순서로 이어진다.

1. `accept_application_with_requirement`가 exact old record, successor active/profile/configuration을 조립한다.
2. `stage_application_intent_checked`가 temp write+sync 뒤 journal rename을 수행한다. Rename 전 실패는 미수락이다.
3. Rename 뒤 directory sync 또는 후속 파일 게시 실패는 accepted/recovery-pending으로 남긴다.
   새 요청은 pending gate로 막고 호스트는 수락된 prepared Core를 한 번 commit한다(S01 근거 재사용).
4. `finish_application_intent`가 원본 보호·설치 revision을 검증한 뒤 profile, active와 backup을 게시하고 journal을 제거한다.
5. 재시작의 `recover_pending_application`은 같은 intent를 끝낸다. 이미 설치된 동일 record도 멱등 허용한다.
   Journal 제거 뒤 마지막 sync만 실패한 상태는 재sync 후 pending을 해제한다.

명시적인 supported corrupt-active reset에는 `corrupt_active_originals`와 exact 원본 검증이 별도로 있다.
일반 reset·빈 저장 fallback으로 묶지 않는다. 이번 대표 시험은 일반 accepted journal 경로에 한정하며
corrupt reset의 모든 승인 조합·원본 교체 경쟁은 별도 범위다.

## 5 구조 평가와 개선 후보

| 기준 | 판정 |
| --- | --- |
| 책임·도메인 경계 | Core 상태 의미, host 봉투·파일 I/O, approved configuration 검증이 구분됨. 유지 |
| 상태 소유권 | current/SavePoint, monotonic knowledge/정확 lifecycle, accepted journal의 successor 구분. 유지 |
| 상태 전이·수명 | unfinished/stable/terminal과 pending choice 보존, journal accepted/pending/clear 수명 확인. 유지 |
| 실패·commit | rename acceptance와 Core prepared commit을 연결. pre/post acceptance 및 재적용을 시험으로 확인 |
| 규칙 중복 | 다섯 형식의 version은 같은 정책의 복제가 아님. 단일 상수로 통합하지 않음. 설명의 중복은 D1 |
| 의존 방향·공개 범위 | Core opaque checkpoint를 host가 운반하고 승인 gate가 Core inspector를 소비. 파일 I/O의 Core 유입 없음 |
| 변경 용이성 | 현재 형식과 과거 규칙의 복제 문구가 여러 절에 남음. canonical 지원 표와 참조로 정리할 D1 선정 |
| 데이터·외부 계약 | low-level 봉투 수용과 실제 Continue 지원을 구분. 버전·소유 ID 문구는 D1에서 교정 |
| 결정론·계속 실행 | 같은 다음 gate/RNG, SavePoint 독립 rewind, loads/peak 백업 보존 확인. 모든 도메인 replay 완료는 아님 |
| 오류·관측 | unsupported/protected/corrupt/pending을 구분하고 원본 보호. snapshot의 복구 쓰기를 readonly와 구분 |
| 검증 가능성 | Core fixture 기반 복원과 host 실제 파일·자식 프로세스 종료 시험을 분리. Opaque host fixture를 게임 저장 복원 증거로 사용하지 않음 |
| 성능·자원 수명 | immutable cache·공유 SavePoint·clone/decode admission을 확인. 전체 비용 측정 없이 codec 통합·clone 제거를 채택하지 않음 |

### S04 D1 — 저장 지원 범위와 원정 identity 설명 정합화

현재 조사한 대표 경로에서 실행 코드 오류는 확정하지 않았다. 확인한 후보는 **문서·주석 정합성 교정**이다.

- U15 첫머리에 profile6/journal4와 이전 절의 과거 의미를 설명한 override가 이미 있다. 따라서 V3 문구만으로
  현재 V3 호환 기능이 고장났다고 판정하지 않는다. 다만 같은 문서의 “현재 형식” 표는 V12/V13/profile4/journal3,
  U15 writer 문장은 Core16, Durable 절은 Core17/journal3, 시설 절은 profile5/V4 묵시 읽기로 흩어져 있다.
  실제 독자가 적용 순서를 역추적해야 하는 정합성 문제다.
- 실제 writer/reader는 Continue17·active7·profile6·journal4다. Profile과 journal은 각각 현 버전 외 입력을 거절한다.
  Low-level active store의 opaque 이전 payload 운반과 approved/Core의 재개 허용은 다른 계약이다.
- Durable 절과 `active_run.rs::continue_from_active_run_checkpoint` 주석의 player identity 생략 설명은
  저장된 원정 instance binding과 구분해야 한다. `ExpeditionInstance.player_id`는 저장·검증된다.
- 독립 checkpoint encoder가 V17이라는 사실과 정상 export가 Continue document를 사용한다는 사실을 나눠 쓴다.
  오래된 `V12` 타입명·테스트명만으로 현재 지원 버전을 판정하지 않는다.

주 owner는 S04다. 실행 코드·migration·reader 허용 목록을 문서에 맞춰 바꾸는 작업이 아니다.
호출자 없는 private 독립 encoder와 legacy 선언 제거는 별도 비필수 정리 가능성이지만 현재 경계를 고치는 데
필요하지 않아 선택하지 않았다. 새 저장 추상화·통합 codec도 채택하지 않았다.

## 6 D1 실행 계획과 다음 단계

1. `core_runtime_contract.md`의 Durable 절에 현재 writer/reader/approved 지원 범위를 명확히 모은다.
   Trusted Runtime Safety·U15·Permanent Profile·시설(U13/U17 추가 설명)의 중복 버전·암묵 변환 문구는
   그 소유 절을 참조하도록 정리한다. 과거 변경 이력과 현재 지원 선언을 구분한다.
2. Journal V4의 exact old/new·configuration 및 required corrupt-original option, profile V6의 엄격한 reader를
   설명한다. V1–V3 journal 또는 구 profile을 자동 변환하는 경로를 새로 만들지 않는다.
3. 전송 세션/root player 주입과 저장된 원정 player/instance binding을 계약·관련 주석에서 구분한다.
4. 수용 조건은 문서만 읽어도 format transport와 실제 Continue 허용을 구별할 수 있고 현재 코드와 일치하는 것,
   executable/API/schema/게임 의미 변화가 없는 것이다. 기존 실행 근거를 재사용하고 문서·주석만 수정한다면
   동일 테스트를 반복하지 않는다. 변경 후 별도 시점 완료 검토를 남긴다.
5. D1 완료 후 지도 순서의 S03/S06 원정·비전투 활동 전이 대표 흐름 조사로 이어간다.
   실제 종료·선택·노드 완료 책임을 먼저 정하고 모든 활동을 한 번에 전수 구현하지 않는다.

이번 단계는 조사·계획 완료까지이며 D1은 아직 구현하지 않았다. 다음 재개 위치는 위 D1이다.
전체 S04 무결점, 모든 과거 저장 호환성 또는 전체 Core 리팩토링 완료를 뜻하지 않는다.

## 7 실행 근거와 한계

[선택 입력 기준선](evidence/s04_001/baseline.json), [Core 원시 출력](evidence/s04_001/core001.log),
[서버 원시 출력](evidence/s04_001/server002.log), [최종 집계](evidence/s04_001/verification.json)를 연결한다.
**Core 8개 + 서버 10개 = 18개 통과, 실패·무시 0**이다. Core 빌드 37.19초/시험 5.79초,
서버 라이브러리 빌드 46.42초/시험 0.31초다. 16개 종료 지점은 서버 테스트 1개 내부의 입력이며 16개 별도 테스트로 더하지 않는다.
이전 S01/S07 테스트 수를 이번 결과에 합산하지 않는다.
첫 서버 명령은 `--bin game_server`를 선택해 0개를 실행했다([출력](evidence/s04_001/server001.log)).
저장 모듈의 실제 owner가 `src/lib.rs`임을 확인하고 `--lib`로 교정했다. 0개 실행은 검증 통과 수에 포함하지 않는다.

Core 시험은 새 application Continue, 같은 다음 gate/RNG, 3회 SavePoint load와 profile 보존, 독립 SavePoint
wire 복원, 미완료 SavePoint·정비·본사 연락·상점·보상 복원, 첫 SavePoint 이전 Continue, 창고 소비·투자 원장 복원을 확인한다.
유효 route/data를 사용하는 기존 fixture지만 일부는 테스트 준비용 내부 상태와 강제 노드 배치를 사용한다.
전체 자연 플레이·공개 API만으로 구성한 새 시나리오라고 주장하지 않는다.

서버 시험은 실제 temp 디렉터리의 publication·backup·loads/peak·terminal tombstone·원본 보호와 journal
acceptance 전후 실패·재적용을 확인한다. Crash 시험은 현재 test executable의 자식 프로세스를 파일 게시
16개 지점에서 지정 종료시키는 방식이다. 장치 전원 손실·OS page cache 손실 재현이 아니며 approved mod
세대 전체나 게임 Core payload 의미를 검증하지 않는다. 이 fixture의 V12 불투명 payload는 Core V12 재개 지원의 증거가 아니다.

전체 Core/server suite, release, Unity, approved A/B generation 교체 matrix, corrupt reset 전체,
모든 활동별 복원·과거 저장 migration·장치 전원 손실은 이번에 실행하지 않았다.


조사 종료 시 선택 소스·계약·빌드 입력이 기준선과 같음을 확인했다. 실제 실행 로그의 선택 테스트 수와
fixture·failure injection 범위를 다시 대조하고, 코드·실행 근거와 문서 후보를 분리해 기록했다.
기준의 12개 항목에서 유지·교정 후보·보류·한계를 설명했으므로 대표 경계 조사·계획을 완료한다.


## 8 S04 D1 교정·완료 검토

### 적용한 변경

[Runtime 계약](../core_runtime_contract.md)의 Durable 절에 현재 형식과 reader/approved 지원 범위를 모았다.
U15·Trusted Runtime Safety·Permanent Profile·시설 절은 현재 지원 표를 참조하고 도입 당시 설명과 구분한다.
[Continue 주석](../../src/game/world/active_run.rs)은 root player 초기화와 저장된 원정 소유자 binding을 나눠 설명한다.
변경 전후는 [diff](evidence/s04_d1_001/source.diff), 확정 대상은 [검토 대상](evidence/s04_d1_001/review-target.json)을 따른다.

- Core Continue V17, private 독립 checkpoint 표현 V17, host active envelope V7, profile V6, journal V4를 구분했다.
  Low-level 봉투가 이전 opaque payload를 운반하는 것과 approved/Core가 게임 재개를 허용하는 것은 별도다.
- Profile의 이전 묵시 읽기·journal V3의 현재 지원처럼 보이던 문구를 정리했다. 과거 U15 writer 설명은 이력으로 남겼다.
- Journal V4의 exact old/new/configuration과 required nullable `corrupt_active_originals`를 설명했다.
  이 필드는 파일 전체가 아닌 원본별 길이·SHA-256 식별값을 기록한다.
- `ExpeditionInstance`의 player ID·sequence·UUID 저장과 호출자/profile binding 검증을 명시했다.
- Preaccept 실패의 보장을 successor Core/profile/active 확정값의 미게시로 한정했다. Temp write나 기존 backup 복구까지
  모두 없다고 주장하지 않는다. S01에서 확인한 accepted 이후 recovery-pending·단회 commit 의미는 유지했다.

### 검증과 완료 검토

실행 코드·공개 API·저장 schema·migration 정책은 바꾸지 않았다. Rust 변경 줄은 모두 문서 주석이며,
계약 변경은 계획한 다섯 절에 한정된다. S04 조사 당시의 나머지 소스·빌드 입력은 그대로다.
기존 **18개 통과 및 16개 종료 지점**은 과거 실행 근거로 재사용했고 이번에는 테스트를 다시 실행하지 않았다.

편집 후 별도 시점에 확정 diff와 S04의 writer/reader·소비자 근거를 대조했다. 같은 작업자의 완료 검토이며
외부 독립 감사는 아니다. 원본 식별값과 preaccept 설명을 좁힌 뒤 최종 변경 범위를 다시 확인했다.

| 검토 항목 | 결과 |
| --- | --- |
| 현재 지원 형식 | Core17·active7·profile6·journal4의 기존 상수·reader gate와 일치. 이전 bytes의 자동 변환을 추가로 약속하지 않음 |
| 독립 checkpoint와 opaque 봉투 | 정상 Continue의 형식·승인 조건으로 오해할 설명 제거. 독립 encoder 제거·codec 통합은 수행하지 않음 |
| 원정 identity | `ExpeditionInstance`, Continue의 player 비교, profile active marker·ownership 검증과 일치 |
| Journal 의미 | V4 old/new binding·원본 식별값과 pre/post acceptance의 기존 의미를 보존 |
| 동작 보존 | Rust 주석만 변경. 저장 bytes·reader 허용 목록·게임 상태 전이 변경 없음 |
| 문서 연결 | 현재 표로 참조를 모으고 과거 설명과 분리. 링크·형식·diff 검사 통과 |

최종 검사와 재사용 범위는 [verification.json](evidence/s04_d1_001/verification.json)에 기록했다.
S04 D1의 필수 미해결은 없으며 교정·검증·완료 검토를 완료한다. 전체 저장 호환성·장치 전원 손실 검증을
새로 완료한 것으로 확대하지 않는다.

### 다음 재개 위치

다음은 **S03/S06 원정·비전투 활동 전이 대표 흐름 조사와 개선 계획**이다.
첫 흐름은 지도 노드 선택·입장 → 상점 이용/나가기 → 노드 완료·다음 phase로 잡고,
이벤트 선택·종료를 비교해 공통 완료 책임과 활동별 완료 조건을 확인한다.

- `world/node_flow.rs::handle_confirm_enter_node`, `handle_complete_node`에서 진입·완료 계획과 commit을 따라간다.
- `world/shop.rs`, `world/event_node.rs`의 실제 행동·거절·종료 생산자를 연결한다.
- S01의 outer draft/commit과 S04의 unfinished/stable Continue 의미는 재사용한다. 비전투 활동의 지급·소비·노드
  완료가 같은 확정 경계에 들어가는지, 취소·중복 입력·Continue 때 결과가 유지되는지 부족한 경로만 조사한다.
- 종료점은 대표 흐름 조사·후보 판정·필요한 개선/검증 계획이다. 모든 활동 구현이나 S03/S06 전체 감사를 뜻하지 않는다.
