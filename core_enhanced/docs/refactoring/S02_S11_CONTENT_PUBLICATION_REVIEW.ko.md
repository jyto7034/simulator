# S02/S11 데이터 검증·게시·승인 경계 조사와 개선 계획

작성·갱신일: 2026-10-17

현재 단계: 대표 조사·선택 테스트 19개 검증과 D1 주석 교정·완료 검토 완료. 다음은 §6의 S08 대표 조사다.
최신 순서는 [출발점](README.md)을 따른다.

## 1 범위와 기준

외부 pack 파일 → owned snapshot → definition resolution → 최종 검증 → `ValidatedGameData` →
authoring/승인 host → 기존 원정과 다음 시작의 generation 소비를 연결했다.
최초 조사 종료점은 조사·후보 판정·개선 계획이었다. 제작 CLI 전체, 모든 schema/consumer, Lab 측정·승인 전체의 재인수는 아니다.
최초 조사에서는 제품 코드·계약·테스트를 변경하지 않았다. 후속 D1 주석 교정은 §4에 기록한다.

[데이터 loading 계약](../data_loading_contract.md)의 Source Of Truth, Mod grouping and approved host admission,
Common authoring validation, Prepared authoring trials, Filesystem 절을 기준으로 했다.
실패를 embedded/이전 성공 세대로 대체하지 않기, 최종 검증 전 token 미게시, source bytes와 semantic identity 구분,
정적 validation과 private approval 구분, 기존 실행 세대 보존을 우선 확인했다.
직전 진행 문서 4개와 S05 선택 입력 32개가 그대로임을 확인하고
[S01](S01_APPLICATION_BOUNDARY_REVIEW.ko.md)의 prepared·Trial commit과
[S04](S04_PERSISTENCE_BOUNDARY_REVIEW.ko.md)의 saved identity·승인 저장 admission 경계를 재사용했다.

## 2 실제 흐름과 권위

| 단계 | 실제 생산·소비 | 실패·게시 경계 |
| --- | --- | --- |
| 획득 | `content_pack::load_with_runtime_safety`가 trusted root/selection/limits/settings로 `snapshot::acquire` 호출 | 파일·입력 한도·경로 검사를 통과한 owned bytes만 다음 단계로 전달 |
| 재확인 | snapshot은 pinned root에서 각 경로를 다시 읽어 identity와 bytes를 비교하고 source receipt 생성 | 관찰한 중간 변경은 거절. filesystem 전체를 원자적으로 찍은 snapshot이라고 주장하지 않음 |
| 구조·충돌 해결 | catalog typed raw parse → dependency/writer graph → host의 명시 winner → final definitions | selection 순서를 implicit override로 쓰지 않음. 버려지는 writer도 구조/unsupported opcode 검사는 통과해야 함 |
| 최종 검증 | final definitions assemble/validate → database compile → 수식 runtime 지원 검사 → generated combat preview 계약 | 어느 단계든 실패하면 token 미반환. 마지막 publication poll 뒤에만 `ContentGeneration` 생성 |
| 보유 | private `ContentGeneration`은 database·semantic identity·source receipt·source metadata 소유. opaque token은 Arc | 기존 token의 원본 파일을 바꿔도 기존 generation은 변하지 않음. 새로운 load 실패가 기존 객체를 덮지 않음 |
| Authoring | `authoring::validate`는 동일 loader를 limits/cancellation scope에서 호출해 token+usage 소유 | receipt/describe/explain과 Trial은 보유한 세대 사용. 공개 authoring 성공은 정상 host approval이 아님 |
| 승인 후보 | `Candidate::acquire_registry`가 frozen bytes를 확보하고 trusted base receipt/settings와 일치 확인 후 publish | retained candidate의 individual/final projection은 같은 capture 사용. 변경/삭제된 원본 파일을 다시 읽어 후보를 바꾸지 않음 |
| 승인 발급 | `ValidationIssuer::run`이 exact cache 또는 supervised Lab 결과를 받고 revision·취소·PASS·durability 확인 | ContentFailed/Incomplete/Cancelled/Stale/RecoveryPending은 현행 완료 approval과 구별. 공개 DTO/token만으로 승인 생성 불가 |
| 적용 | `ApprovedHost::apply`가 issuer의 current approval 검사와 설치를 lock 범위 안에서 수행 | 외부 issuer·stale capability 거절. 이후 검증 중에도 이미 설치된 선택은 유지 |
| 소비 | host의 future generation은 새 application/StartNewGame에 제공. 기존 command는 active generation 사용 | 시작 실패·준비 폐기는 기존 application 보존. 승인 교체가 진행 중인 원정의 데이터를 즉시 교체하지 않음 |

근거: [loader](../../src/game/data/content_pack/mod.rs), [snapshot](../../src/game/data/content_pack/snapshot.rs),
[resolver](../../src/game/data/content_pack/resolver.rs), [token](../../src/game/data/mod.rs),
[authoring](../../src/authoring/mod.rs), [candidate](../../src/mod_validation/candidate.rs),
[issuer](../../src/mod_validation/issuer.rs), [승인 host](../../src/mod_validation/host.rs),
[application](../../src/application.rs).

### 서로 다른 identity와 수명

- semantic hash는 resolved definitions뿐 아니라 contract·pack identity·writer 결정을 포함한다.
  comment·파일 나열 순서 변경은 같은 의미일 수 있지만 exact source receipt는 달라질 수 있다.
- runtime safety profile은 content semantics와 별도로 `admission_identity`에 포함된다.
  candidate identity도 required configuration·source receipt·effective settings를 결합한다.
  같은 semantic hash만으로 승인이나 cache 재사용을 추정하지 않는다.
- 마지막 **해당 generation의 strong reference**가 없어져야 회수된다. Core/authoring/Trial 외에도
  Candidate·issuer job·installed approval이 참조할 수 있다. 마지막 application 종료만으로 승인 후보까지
  전부 해제된다고 주장하지 않는다. issuer에는 유한 job admission 한도가 있다.
- public loader는 새 토큰을 반환하는 함수이며 전역 “현재 세대” publisher가 아니다.
  source 재검증과 immutable generation, future selection, application prepared commit을 하나의 교체 지점으로 합치지 않는다.

### 정상 서버와 승인 조사 범위

[bootstrap](../../../game_server/src/bootstrap.rs)의 `load_game_data`는 static embedding/diagnostic API다.
정상 시작은 `load_approved_host` → [AdmissionSession](../../../game_server/src/mod_admission/session.rs)을 요구한다.
설정은 actual default-release host와 worker pair의 source/compiler/target/profile 문맥을 확인한다.

Startup은 각 individual candidate와 final combination을 별도로 획득·실행하고, 통과한 individual의 source receipts가
final capture에 그대로 포함되는지 확인한다. 모든 startup 파일을 단 한 번만 읽는다고 해석하지 않는다.
이미 보유한 candidate의 `validate_and_apply_report`는 individual/final projection을 frozen capture에서 만든다.
Individual PASS 합산으로 final PASS를 대체하지 않으며, 전부 held이면 replacement를 설치하지 않는다.

issuer는 새 job에 revision을 부여하고 이전 미완료 job을 stale/cancel 처리한다.
실행 후 revision·취소를 다시 확인하고 PASS 결과의 cache 게시가 durable일 때 approval을 만든다.
accepted publication이 recovery pending이면 exact current record 복구 후에 발급한다.
host는 issuer 검사와 future 선택 설치를 직렬화하고, 정상 소비의 acceptance와 Apply도 installation lock으로 연결한다.
실제 저장 게시와 기존 원정 identity는 S01/S04에서 확인한 계약을 따른다.

이 승인 부분은 이번에 **코드 호출·상태 전이를 조사한 근거**다. 실제 calibrated worker pair·private accepted cache를
사용한 full approval 실행을 새로 통과했다고 주장하지 않는다. 이번 server 테스트도 bootstrap의 static loading/profile 경계다.

## 3 구조 평가

| 기준 | 대표 흐름의 근거·판정 |
| --- | --- |
| 책임·도메인 경계 | acquisition/resolution/final validation, authoring, issuer, host install, application commit 구분. 유지 |
| 상태 소유권 | immutable token과 candidate capture, issuer capability, installed future selection의 owner 명확 |
| 전이·수명 | 기존 원정 A·후속 시작 B, Trial replacement 실패 후 A 유지, 마지막 참조 해제를 구분 |
| 실패·commit | parse부터 generated setup까지 실패 token 미게시. 승인 실패와 installed selection 유지 별도 |
| 규칙 중복 | production/authoring/Candidate가 공통 publication 경로 사용. static validation을 approval로 중복 해석하지 않음 |
| 의존·공개 범위 | opaque generation과 private approval constructor로 권한 구분. host가 실제 파일·worker·승인 설정 소유 |
| 변경 용이성 | raw adapter/최종 validator가 정의 변경을 처리하고 Core는 retained token 사용. 범용 JSON patch/새 registry 필요 미확인 |
| 데이터·외부 계약 | source receipt, semantic identity, required configuration, runtime profile이 서로 다른 의미로 유지됨 |
| 결정론·복원 | 명시 writer selection·정렬된 identity 입력. saved generation admission은 S04 재사용 |
| 오류·관측 | source/phase/code가 production·authoring에서 보존되고 invalid input을 fallback으로 숨기지 않음 |
| 검증 가능성 | 실제 외부 파일 loader·CoreApplication·Trial 경로 테스트 존재. resolver 내부 fixture와 정상 approval 재인수는 구분 |
| 성능·자원 | owned snapshot·generation pin·bounded validation·issuer job 한도 확인. 이번 최대 pack/긴 수명 비용 측정 없음 |

조사한 대표 흐름에서 동작 오류와 구조 변경이 필요한 설계 문제를 확인하지 못했다. 이는 위 기준에 따른 범위 내 유지 판단이며
전체 설계에 개선 여지가 없다는 뜻은 아니다. loader와 approval의 통합, 전역 generation registry,
implicit last-known-good fallback 도입은 선택하지 않는다. 실패 의미와 권위가 달라지기 때문이다.

## 4 D1 주석 교정 후보와 계획

**교정 전 확인한 설명 불일치:** [data/mod.rs](../../src/game/data/mod.rs)의 `value_expr_data` 필드 주석은
authored expression을 WP02 consumer 연결 전까지 publication-blocked라고 설명했다.
실제 [수식 지원 검사](../../src/game/value_expr/catalog.rs)의 `ensure_runtime_publishable`는
지원하지 않는 aggregate member/owner 조합을 거절하며 지원되는 authored expression은 게시한다.
관련 외부 loader 테스트도 허용/거절을 함께 검증한다. 이는 실행 오류가 아니라 오래된 주석이다.

아래 계획으로 **S02/S11 D1: 필드 주석 교정과 별도 시점 완료 검토**를 수행했다.

1. generation-local immutable amount plan이라는 소유권 설명을 유지한다.
2. 전면 차단·미래 WP02 연결을 전제로 한 문장을 제거하고 publication 시 runtime 지원 검사를 거친다는 현재 사실을 적는다.
3. 실행 코드·공개 API·schema·승인 정책·수식 지원 범위는 변경하지 않는다. 관련 없는 주석 전수 교정으로 넓히지 않는다.
4. diff가 주석에 한정되는지와 실제 검사 호출을 대조한다. 실행 입력이 그대로라면 이번 테스트 근거를 재사용한다.
5. 변경 후 별도 시점에 설명의 정확성과 영향 범위를 검토하고 README·지도·이 기록에 완료와 다음 위치를 갱신한다.

완료 조건은 거짓 전면 차단 설명이 없어지고 지원 범위의 권위를 현재 validator에 남기는 것이다.
새 수식 기능을 구현하거나 테스트 기대값을 바꾸는 작업이 아니다.

### D1 구현과 완료 검토

2026-10-17, 기존 조사 입력 27개·진행 문서 4개가 그대로임을 확인하고 위 계획을 실행했다.
`value_expr_data`의 문서 주석 두 줄을 “generation-local immutable amount plans”와
“authored expressions are checked for runtime support before content publication”으로 교정했다.
이는 설명 불일치 교정이며 실행 오류 수정이나 아키텍처 개선 실적으로 집계하지 않는다.

구현 대상을 고정한 뒤 별도 pass에서 현재 필드·loader 호출 순서·validator의 허용/거절을 대조했다.

| 완료 조건 | 판정·근거 |
| --- | --- |
| 소유권 설명 보존 | generation-local immutable amount plans 유지 |
| 전면 차단 설명 제거·지원 범위 권위 보존 | runtime 지원 검사 설명으로 교정. 구체 지원 목록은 기존 validator가 소유 |
| 실행·API·schema·승인 정책 불변 | 변경은 문서 주석 두 줄뿐. 주석을 제외한 줄은 이전 파일과 동일 |
| 기존 검증 재사용 | 선택 입력 중 나머지 26개 불변. 기존 raw log의 17+2 통과 확인. 테스트 추가·재실행 없음 |
| 분리 검토 | 구현 후 고정된 source hash·diff·현재 검사 경로 대조. 동일 작성자의 별도 시점 검토이며 독립 감사는 아님 |

판정은 **D1 완료**, 필수 잔여 없음이다. [변경 diff](evidence/s02_s11_d1_001/source.diff),
[구현 고정](evidence/s02_s11_d1_001/implementation.json), [완료 검토](evidence/s02_s11_d1_001/review.json),
[최종 확인](evidence/s02_s11_d1_001/verification.json)에 근거를 기록한다. §5의 실행 범위 한계는 그대로다.

## 5 검증과 한계

선택한 소스·계약·build·테스트 입력 27개는 [baseline.json](evidence/s02_s11_001/baseline.json),
실행 명령·환경은 [commands.json](evidence/s02_s11_001/commands.json)에 기록했다. 전체 의존성 freeze는 아니다.
최종 입력·문서 확인은 [verification.json](evidence/s02_s11_001/verification.json)에 기록한다.

| 실제 실행 | 결과 | 시간·근거 |
| --- | --- | --- |
| Core lib: publication/generation 8, resolver 3, snapshot 3, authoring validation 2, Trial 수명 1 | 17 통과, 실패·무시 0 | build 36.76초, test 20.04초. [core001.log](evidence/s02_s11_001/core001.log) |
| server lib: runtime profile·실제 pack bootstrap | 2 통과, 실패·무시 0 | build 38.22초, test 9.85초. [server001.log](evidence/s02_s11_001/server001.log) |

합계 **19개 통과, 실패·무시 0**이다. 기존 경고 Core lib test 33개, server 빌드의 Core lib 36개·server lib test 6개는 남아 있다.

- 외부 loader 테스트는 공식 데이터를 임시 pack으로 만들고 실제 파일을 수정·읽는다. 다른 가격/briefing,
  invalid syntax·opcode·generated spawn zone 같은 입력을 만들고 기존 A·B 세대 보존을 확인한다.
- resolver 테스트는 내부 `ParsedPack` fixture이므로 외부 파일 종단 경로와 구분한다.
  snapshot 재확인 테스트는 획득 중 hook으로 파일을 바꾸며 관찰한 변경의 거절을 확인한다.
- generation 시작 테스트는 `CoreApplication`의 공개 prepare/execute 경로와 외부 generation을 사용한다.
  정상 승인 토큰까지 발급한 server 시나리오가 아니다.
- Trial 수명 테스트는 별도 작성 route를 포함한 external pack으로 실제 prepare/advance/replace를 실행한다.
  취소·receipt cap·fuel 실패가 세션과 generation 수명을 보존하는지 확인하며 모든 Trial 입력의 완전성을 주장하지 않는다.
- 실제 cache-boundary 테스트는 현재 코드가 참조하는 `.rs`의 실행 요건만 확인했다. 별도 reviewed worker/cache 입력이
  필요한 테스트를 이번 일반 debug 검증에 섞거나 과거 `goals`의 완료 선언을 새로운 승인 근거로 사용하지 않았다.
- full Lab/calibration·worker parity·issuer cache crash/승인 교체 A/B 재인수, 전체 suite/release/Unity,
  최대 pack 규모 성능·모든 authoring CLI는 미실행이다. 이전 S01/S04 테스트 수는 이번 수에 더하지 않는다.

테스트 종료 후 코드·계약·기록의 근거 수준을 다시 대조하고 선택 입력 27개가 그대로임을 확인했다.
이는 같은 작성자의 조사 검토이며 외부 독립 감사나 정상 승인 시스템 전체의 완료 선언이 아니다.

## 6 다음 재개 위치

D1은 완료했다. 다음은 **S08 전투 내부 수명·이벤트 순서 대표 조사와 개선 계획**이다.
시작 위치는 `game/battle/core/sim.rs`의 `start_battle_execution`·`step_battle_execution`·
`finalize_battle_execution`이며, 실제 호출을 따라 `queue.rs`와 유닛 생성·사망·정리 소유자를 연결한다.
S07에서 확인한 전투 진입·terminal 경계를 재사용하고 한 command/tick의 처리·이벤트 생성·정리 흐름을 선정한다.
동작 정확성뿐 아니라 책임·상태 소유권·의존성·중복·복잡도·변경 및 검증 비용을 평가한다.
후보가 있으면 근거·보존 동작·작업 범위·검증 방법을 포함한 계획을 남기고, 없으면 조사 범위 내 유지 이유를 기록한다.
이번 단계에서 S08 상세 조사나 구현은 시작하지 않았다. 함수 위치만 확인했으며 기존 근거·변경분부터 재개한다.
