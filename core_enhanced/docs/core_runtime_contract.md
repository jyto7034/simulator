# Core Runtime Contract

이 문서는 core runtime 구현자가 지켜야 하는 source-of-truth, 결정론, 관측/전송 경계 계약을 정리한다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | Runtime canonical |
| 소유 범위 | 결정론과 RNG, runtime state와 원자적 commit, event/checkpoint/result lifecycle, projection ownership, recovery, VN runtime, actor identity |
| 소유하지 않는 범위 | 플레이어-facing 규칙의 의도, 타겟팅·범위 알고리즘, live RON authoring/loading, Unity wire의 정확한 JSON shape와 소비 순서 |
| 함께 확인할 문서 | [game_rulebook.md](game_rulebook.md), [skill_target_contract.md](skill_target_contract.md), [data_loading_contract.md](data_loading_contract.md), 외부 `unity_core_contract.md`, 외부 `core_unity_battle_transport_contract.md` |

[game_rulebook.md](game_rulebook.md)는 플레이 규칙을 소유하고, 이 문서는 그 규칙을 runtime code,
event log, snapshot, Unity-facing DTO로 구현할 때의 금지/허용 경계를 소유한다. 이 durable
계약들이 runtime 목표의 source of truth며, 코드·live RON/data가 다르면 구현 drift로 취급해
문서에 맞게 수정한다. 새 material 의미나 durable 문서 충돌이 발견될 때만 사용자에게
정책 결정을 요청한다.

## Tutorial and Expedition lifecycle

현재 공개 mode는 `Tutorial`과 `Expedition`이다. `Standard`/`Endless` 입력과 제거된 튜토리얼 완료 flag는
호환 alias로 받지 않는다. 빈 profile에서도 Expedition을 시작할 수 있다. `StartNewGame`은 title 또는
확정 terminal에서만 시작하며 `replace_active_run: false`를 사용한다. 진행 중 원정은 명시 `AbandonRun`으로
종료한다. Tutorial은 authored preset의 독립3층이며 briefing의 `SkipTutorial`, 완료, 실패와 복원 모두
영구 profile을 변경하지 않는다. 이 장은 현재 구현 계약이며 U12 기능 검증/완료 판정을 뜻하지 않는다.

Expedition 선발은 role별2/총6 후보에서 중복 없는3명과 필수 `loan_copy_ids` 배열(0–2 사본)을 한 prepared
departure로 처리한다. authored 시작 예산500과 기본 loadout은 성공 출발에 한 번만 지급한다. scenario seed는
지도/채용 난수를 소유하고 player+checked sequence의 expedition instance는 직원/장비/파편 UUID를 소유한다.
대여는 permanent vault→active 사본 이동이며 기록 기반 재발급이 아니다. dormant 사본도 보존하고 현재 정의가
없으면 대여만 거절한다. vault/active 및 장비/파편 합집합의 중복 UUID와 key/embedded ID 불일치를 거절한다.

HQ의 공개 result/query, 채용 명령과 회복 후보는 현재 가격/잔액, 깊이, session 잔여 횟수, roster 정원과
실제 기본 무기의 deployability를 함께 소비한다. 현재 authored 가격100, session 최대2회, 지원 깊이1–4,
roster10/deployment8이다. 무효 정의 또는 무기 없는 offer를 합법 채용 후보로 노출하지 않는다.

일반 노드 첫 진입은 instance-local receipt를 저장하고 authored 억제력 비용을 한 번 소비한다. 현재 개발
초깃값12/기본비용1이며 activity 또는 node definition의 `first_entry_cost`가 우선한다(0 허용). Start/Gate/방 이동은 소모하지 않는다. 규정비용 전체를 Breakdown 유예4에서 차감하고 실제 억제력 잔량은 saturating 차감한다. 입장 사망과 자산회수는 치료·활동보다 먼저 확정한다. 양수→0에서는 활동 실행 전에 continuation 선택을
저장한다. 깊이1–2는 Continue/Abandon, 깊이3 이상은 Continue/Return을 소비한다. 깊이3 이상의 실제 Gate는
Return/Continue를 제공한다. pending 선택은 NodeConfirm, selectable node, 정확 session/kind/category/payload,
깊이와 cause에 결속되며 Gate payload는 `None`이다. 선택 복원이나 재입장으로 비용/활동을 중복 실행하지 않는다.

성공 귀환은 실제 휴대 사본을 vault로 이동하고 일반 무기 강화0/장착 해제를 적용하며 직원·run 자원을 종료한다.
실패/포기는 active 자산만 폐기한다. 모든 terminal은 active marker와 Continue를 같은 draft에서 종료하고,
첫 성공 귀환의 `Never→Pending→Consumed` story는 `AcknowledgeFirstReturnStory`로 한 번 소비한다.
same-instance SavePoint 복원은 최신 영구 knowledge/story를 되감지 않는다. durable 출발/대여/귀환/실패/story ack는
exact profile+active paired intent를 사용하며 profile-only host에서는 lifecycle 변경을 Core commit 전에 거절한다.

## U16-1 지원 노드 활동

상점의 작성 상품은 기존 장비/소비품에 유물·부활 촉매·침식 촉매를 추가한다.
유물 구매는 정상 사본 하나를 기존 공통 지급 경계에서 생성하며 `PurchaseItem`은
`relic_diffs`와 변경된 `crafting_resources`도 반환한다. 부활 재료와 침식 자원은 별도
소유자에 지급한다. 상품 표시에는 유물 정의/정상 variant 또는 촉매 용도를 제공한다.
일반 유물 매각과 가루·파츠의 직접 구매/판매 권한은 추가하지 않는다.

`base_price`는 정가 정의가 없는 새 상품에만 필수이며 기존 장비/소비품 정가를 덮어쓰지
않는다. `guaranteed` 상품을 먼저 배치하고 남은 자리를 기존 가중 추첨으로 채운다.
구매된 비반복 상품은 갱신 후보에서 제외된다. 일반 상인의 기본 부활 촉매는 이 경로로
제공하며 가격은 작성 데이터의 초기 조절값이다. 구매·갱신은 비용/소진/자산/UUID를
함께 확정하고 실패 시 전부 유지한다. 보유 표시 기준 50/60은 Core 구매 한도가 아니다.

HQ 조회의 `recruitment_terms`는 기존 채용 가격100, 기본 무기/파편 포함 여부와 남은
채용/자격을 공개한다. 기존 채용 queue와 대여 원장은 계속 각 소유자가 관리한다.
일반 소비품의 즉시 부상 치료도 이전 활성 효과를 제거하고 수량 하나를 사용한다.
전투 결과의 기간 차감, 후퇴 복원과 무료 침식 Event의 권한은 기존 계약을 유지한다.

새 상품은 현재 저장 codec으로 왕복하며 미완료 노드는 Continue에서 입장 전 상태로,
완료 노드는 확정 결과로 복원한다. 실제 paired-store 게시 실패는 메모리와 두 파일의
부분 확정을 남기지 않는다. 유물 소유자가 있는 Continue의 JSON Value 예약은 객체마다
루트 노드 하나와 전체 필드 다섯 개마다 추가 B-tree 노드를 예약한다. 현재 기본
`serde_json::Map`/Rust B-tree의 점유 불변식에 의존하며 toolchain/backend 변경 시 재검증한다.
typed decode의 보수적 예약과 독립 producer들의 누적 계산·16MiB 기본 한도는 유지한다.
근거와 실제 할당 관측은 U16-1 실행의 `RESTORE_MEMORY_REVIEW.md`에 기록한다.

## U15 사본 투자와 재료 창고

U15 도입 당시 저장 writer는 Core Continue/독립 checkpoint16, 서버 profile6, paired application journal4였다.
현재 형식·reader 지원 범위는 [Durable active-run host boundary](#durable-active-run-host-boundary)를 따른다.
아래 U04–U14 절의 이전 버전 번호·묵시 읽기 설명은 당시 계약이다. U15는 투자 원장 없는 개발 저장을
자동 변환하지 않으며 지원하지 않는 버전은 원본을 보존하고 거절한다. 기존 명시 V7/V8 변환 API도
U15 투자 누락을 임의 추정하는 변환 경로로 사용하지 않는다.

무기만 정비 강화할 수 있으며 재료별 실제 결제는 `OwnedEquipment.investment.materials`에 기록한다.
파편의 `OwnedFragment.investment`와 귀환 `VaultFragment.investment`도 같은 재료별
`{carried, warehouse}` 누적 원장을 갖는다. 무료 지급은 빈 원장이다. 분쇄 추가 환급은 각 재료의
두 출처 합계에 대해 `floor(total/2)`이며 현재 recipe 가격이나 강화 단계로 투자액을 추정하지 않는다.
기본 분쇄 보상과 환급을 재료별 합산하고 모두 소지분으로 지급한다. 투자 엔케팔린은 환급하지 않는다.
장착 파편·장비의 분쇄는 명시 확인이 필요하며 실패 시 사본/장착/통화/재료/UUID 상태를 보존한다.

예약 id `equipment_dust`의 유일한 잔액 owner는 `RunCraftingResources.equipment_dust`다.
전투·유물 분쇄·typed 가루 보상과 같은 id의 일반 재료 보상, 무기 강화·분쇄 환급은 이 잔액을 쓴다.
정비·Inventory의 재료 목록은 이를 투영한다. inventory stack에 복제 잔액을 저장하지 않는다.
가루는 귀환 회수/창고 대상이 아니며 원정 종료 시 사라진다.

`PlayerBehavior::AwakenSkillFragmentWithMaterials`와 wire `awaken_skill_fragment_with_materials`는
`copy_uuid`, `form_id`, `materials: [{material_id, carried, warehouse}]`를 받는다. 각 재료는
recipe 합계와 정확히 일치해야 하며 중복/부족/미지 항목을 거절하고 다른 출처로 자동 대체하지 않는다.
기존 `AwakenSkillFragment`는 소지분 전용이다. 읽기 전용 `PreviewSkillFragmentBloom`
(`preview_skill_fragment_bloom`)은 같은 입력을 받고 `maintenance_options.selected_bloom_payment`에
요구량·선택량·실제 출처 예상 원장·실행가능/거절이유를 반환한다. 조회는 자원/사본/Continue를 바꾸지 않는다.
`DismantleEquipmentConfirmed` (`dismantle_equipment_confirmed`)의 필드는 `item_uuid`, `confirmed`다.
기존 `DismantleEquipment`에는 장착품 확인 권한이 없다.

`ReturnFromExpedition` (`return_from_expedition`)은 `materials: [{material_id, amount}]`를 받는다.
허용된 귀환 지점에서 소지 환상체 파츠 종류 합계2와 부활 촉매1까지 명시 회수한다. 빈 선택은 가능하며
선택하지 않은 소지 재료는 종료된다. 자동 종류선택·가루보관·엔케팔린보관은 없다. 귀환 무기는 강화와
투자 원장을 초기화하고, 파편은 개화와 실제 투자 원장을 유지하여 다음 대여/분쇄에 사용한다.

창고 현재 잔액과 해당 원정 누적 출처 소비는 `profile.lifecycle.vault.materials`와
`profile.lifecycle.warehouse_debits`가 소유한다. checkpoint의 목표 소비는 `RunProgression.warehouse_debits`다.
SavePoint와 미완료 노드 Continue는 현재 소비와 목표 소비의 차이만 창고에 반영하며 소지 잔액·사본 투자·
통화도 같은 snapshot으로 복원한다. 창고 전체 snapshot으로 다른 변동을 덮지 않는다. 완료 후 Continue와
실패/포기는 확정 소비를 환불하지 않는다. 저장에 수락되지 않은 한 명령의 rollback token은 별도 lifecycle
자산 복제본을 복원하고 현재 단조 지식은 보존한다. 이 token은 gameplay SavePoint와 수명이 다르다.

부활 촉매는 `HatchEncounter`에서 동일한 명시 출처 결제 helper를 사용한다. 성공한 부화는 출처별 `MaterialInvestment` 영수증과 함께 원정에 저장된다.

## Runtime Foundations

이 장은 다른 runtime 계약이 전제로 삼는 결정론과 검증된 data의 진입 경계를 소유한다.

### Trusted Runtime Safety Profile

운영 기본값의 단일 원본은 [`config/runtime_safety.ron`](../../config/runtime_safety.ron)이다.
`RuntimeSafetySettings::from_ron(source, bytes)`는 파일을 읽지 않고, 64KiB·중첩16 이내의 완전한 version1
설정을 검증한다. 필수 필드 누락·미지 필드·중복·음수·overflow·0·기존 engine ceiling 초과는 source/field/message
오류다. History encoded-entry-array 한도는 빈 JSON 배열의 2bytes 이상이어야 한다. 기존 typed `ExprLimits`의
합법 범위에 새 upper ceiling을 추가하지 않는다. 설정은 콘텐츠 팩·save·플레이어 명령으로 덮어쓰지 않는다.

검증된 generation은 불변 `Arc<RuntimeSafetySettings>`를 소유하고 `CoreApplication`과 첫 `BattleCore`
생성 전에 전달한다. Native 작업량/extent, authoring 실행, expression parse/compile/evaluate, trial 누적 작업/receipt,
battle history window를 각각 기존 소유 ledger에서 적용한다. Nested operation·실패·rollback은 이미 사용한
allowance와 첫 fault를 초기화하지 않는다. Retained bytes는 admission/extent 회계이며 process RSS가 아니다.

작업량·메모리 예산의 기본값은 실측으로 재검토할 수 있다. 내부 work는 CPU 명령 수나 실행 시간이 아니므로
정상 콘텐츠·예상 확장·비정상 입력의 실제 시간과 메모리 peak/회수를 함께 검토한다. 오프라인 검증·실시간 전투·
trusted 개발 측정의 목적을 구분하고, 유한 상한과 host 권한·기존 실패 기록을 보존한다.
측정·조정 절차는 [Core-first 로드맵 §6.4](roadmaps/core_first/ROADMAP.ko.md#64-예산안전성과-증거-재사용)를 따른다.
값의 변경은 아래 profile 식별·캐시 분리·기존 session 불변성 계약을 유지한다.

오프라인 authoring 개발은 trusted host가
[`config/runtime_safety_authoring_dev.ron`](../../config/runtime_safety_authoring_dev.ron)을 명시적으로 선택할 수 있다.
예: `author_content validate <content-root> <selection.json> --runtime-safety config/runtime_safety_authoring_dev.ron`.
이 preset은 U09 당시 기본 설정에서 authoring 실행의 `max_work`만 유한 상한 1,073,741,824로 늘린 저장 profile이다.
그 native retained 한도는 8MiB이고 `development_measurement`가 없으므로 8MiB가 hard 한도다. 현재 embedded 기본값과
`ENGINE_MAX`의 16MiB와 구별하며, host가 명시적으로 고른 이 낮은 profile을 기본값으로 자동 보정하지 않는다.
자동 재시도·fallback이나 콘텐츠가 선택하는 한도 완화는 없다. 정상 유물 1개와 대표 확장 유물 7개의
실측을 근거로 한 개발 편의 설정이며, 임의 콘텐츠 규모의 완주·처리시간 보장을 뜻하지 않는다.

CMVA 개발 검증은 별도의 [`config/runtime_safety_cmva_authoring_dev_v2.ron`](../../config/runtime_safety_cmva_authoring_dev_v2.ron)을
명시적으로 선택할 수 있다. 실제 profile bytes는 `fbd5a60a12dc3bc36080880f96d31e3817abe7a0247f45e45eb4fab197ef9de3`이며,
authoring work100,000,000과 clone slots1,000,000만 해당 CMVA 개발 설정으로 인수한다. Embedded/default·native
work10,000,000/retained16MiB·expression 및 나머지 설정은 바꾸지 않는다. 실제16/500/1000 roster,
1/8/8 deployed,60×1초 입력과 release heap/RSS/work를 검토한 development scope이며 universal scale 보장이 아니다.
Versioned 측정 criterion은 이 runtime profile과 별개이고 native operation cap을 올리지 않는다.

`raw_sha256`는 입력 bytes, `effective_sha256`는 검증된 숫자 설정의 canonical JSON hash다. Comment·공백은
effective identity를 바꾸지 않는다. 콘텐츠 semantic identity 및 expression semantic fingerprint와 별개이며,
replay 소비자는 기록된 `runtime_safety_sha256`를 `verify_effective_sha256`로 비교할 수 있다.
서로 다른 profile의 generation/plan을 하나로 캐시하거나 파일 변경으로 기존 session을 바꾸지 않는다.

Checkpoint는 현재 host profile로 resource/state를 검증한 뒤 commit한다. 작은 한도로 거절되어도 기존
application과 원 save bytes를 보존하며 save의 숫자로 host 한도를 올리지 않는다.
현재 저장 형식과 각 reader의 지원 범위는
[Durable active-run host boundary](#durable-active-run-host-boundary)가 소유한다. Low-level host 봉투의 opaque
payload 운반은 Core의 게임 재개 허용과 다르다. 지원하지 않는 형식/구 mode/제거된 심리 입력 및 V7/V8
converter는 silent alias/drop/fallback 없이 원본을 보호하고 명시 거절한다.
`capture_persistence_rollback` / `restore_persistence_rollback`은 이전 non-battle 호환 capability다.
동일 application의 다음 한 mutation에만 유효하며 premature/foreign/stale/중복 복원은 거절한다.
Active battle에서는 발급/복원하지 않고 monotonic player profile을 유지한다.
현재 host의 durable mutation은 `prepare_command`, `prepare_tick`, `prepare_admin_command`로
결과·continuation·profile을 함께 준비한다. `PreparedApplication`은 application을 독점적으로 빌리며
Clone/Serde/raw root 접근을 제공하지 않는다. Durable intent 수락 전 폐기는 gameplay authority를
보존하고, 수락 뒤 `commit`은 준비한 root와 관측을 한 번 반영하며 gameplay나 allowance를 재실행하지 않는다.
이 경계는 active battle의 자동 tick 및 terminal result에도 적용된다. Legacy
`prepare_non_battle_command`는 non-battle 전용 제한을 유지한다.

History 한도는 전투 길이 제한이 아니다. 생성·import·streaming deserialize는 선택된 window를 적용하고
eviction 뒤 floor/latest sequence 및 gap/resync 의미를 유지한다. Log에 serialize된 한도로 host policy를
대체할 수 없다. `BattleEventLog::deserialize_with_runtime_safety`는 host가 선택한 profile을 명시적으로 받는다.

### Optional Runtime Observation

Core의 `runtime-observation` feature는 기본 비활성이다. 기존 native ledger와 reconciliation이 계산한
값을 읽으며 새 owner 순회, safety 한도, RNG draw, gameplay 오류나 save field를 추가하지 않는다.
`CoreApplication`에 붙인 observer는 clone되는 GameCore draft 밖에 있고, fork는 observer를 상속하지 않는다.
최상위 명령의 사용 작업량·허용된 누적 admission·duration·짧은 peak는 실패/rollback 뒤에도 하나의 시도로
관측한다. 최종 public error를 reason으로 한 번 기록하고, committed usage/commit count는 실제 live root
publication에서만 갱신한다. 준비 객체는 반환 전에 thread-local 관측을 분리하며 폐기는 시도만 남긴다.

Core는 고정 크기 summary를 유한 queue에 `try_send`한다. 수집기 정지·포화·미연결은 관측 손실이고 gameplay
결과나 event를 버리는 조건이 아니다. Host는 별도 선택 설정·유한 metric label/bucket·batch upkeep·지연 해제
gate·weak session lifecycle을 소유한다. 종료/lease drop은 queue delivery 없이도 집계와 선택 슬롯을 정리한다.
Wall duration/수집 시간과 battle simulation time을 구분하며 retained extent/admission은 RSS가 아니다.
실행 방법과 metric/storage 의미는 [로컬 관측 안내](../../game_server/config/observability/README.md) 및
[회계·수집 계약](goals/core_runtime_safety_observability/execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md)을 따른다.

### Determinism And RNG

전투 판정 RNG는 event log 관측 순서와 분리한다.

- 치명타, proc, 명중, 회피, 특수 발동처럼 gameplay 결과를 바꾸는 모든 랜덤 판정은 `event_log_seq`나 로그 기록 순서를 seed 재료로 사용하지 않는다.
- 새 랜덤 판정을 추가할 때는 공격 원천, caster/target, projectile/cast/step/hit identity, battle time 같은 안정적인 gameplay identity에서 seed를 파생한다.
- `BattleEventLog`는 결과를 기록하고 Unity presentation에 전달하는 관측 채널이며, 판정 source of truth가 아니다.
- 로그 이벤트를 추가, 삭제, 분할, 병합하는 작업이 이미 존재하는 전투 판정 결과를 바꾸면 안 된다.
- proc 성공 횟수, cooldown 상태, max trigger count 같은 runtime state는 발동 제한과 디버그에 사용할 수 있지만, proc roll identity의 주재료로 사용하지 않는다.

맵 생성, 보상 후보 생성처럼 생성 시점에 결과가 run state로 고정되는 절차적 생성은 seeded `StdRng` 순차 draw를 사용할 수 있다. 단, 생성 결과를 저장하지 않고 seed만으로 재생성하는 구조를 도입할 때는 별도 계약과 재현성 테스트가 필요하다.

### Data Authoring And Loading

Live RON authoring strictness, external pack loader ownership, fixture 경계, validation gate의 canonical 계약은 [data_loading_contract.md](data_loading_contract.md)를 따른다. 이 문서는 검증된 data가 runtime에 들어온 이후의 동작 계약만 소유한다.

## Approved mod generations and normal host ownership (U11)

콘텐츠 publication token과 normal runtime approval은 별개다. 일반 launcher/server bootstrap은
`AdmissionSession`의 private issuer-approved `ApprovedHost`를 사용하며 public DTO·static validation·raw
`ValidatedGameData`로 normal actor를 만들거나 승인을 우회하지 않는다.
[Data Loading Contract](data_loading_contract.md#mod-grouping-and-approved-host-admission-u11)는 grouping,
individual/final validation, source/identity와 Lab/cache 경계를 소유하고 [사용 가이드](tools/cmva_workflow.ko.md)는
실제 CLI/server 절차를 설명한다. 2026-10-12 [실제 literal entry 검증](goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/evidence/reviewed_adoption_functional002/24__public_entry__literal_smoke.json)은
launcher starter run/Continue와 server WebSocket 재시작 Continue·exact checkpoint 보존·invalid content의 persistence 이전 거절을 확인했다.

- Host의 initial installation과 Apply는 issuer의 현재 approval 확인 및 설치를 같은 authority boundary로
  수행한다. 승인 capability는 exact candidate/build/profile/suite/environment/criterion/selection 문맥을 pin한다.
  Stale uninstalled capability는 implicit Apply가 될 수 없다. DTO 재직렬화·cache 파일 편집은 authority가 아니다.
- 설치한 future B는 이후 승인 없는 C 검증 중에도 사용할 수 있다. 이미 시작한 A run/actor, prepared draft와
  immutable runtime source는 A를 소유하고 B Apply 때문에 교체되지 않는다. 새 run/actor만 설치된 B를 사용한다.
  `StartNewGame` 준비에서만 future generation과 future required configuration을 전달하며 다른 명령은
  기존 application generation/required configuration을 유지한다.
- Durable acceptance는 installed/active configuration을 다시 확인하고 successful accepted A→B 이후 persistence
  pin을 B로 전진시킨다. 미래 C 설치가 active B의 autosave/Continue 권한을 폐기하지 않는다.
  무제한 과거 generation 목록을 새 admission 권한으로 사용하지 않는다.
- Generation은 필요한 host/run/actor/trial/approval/source의 strong reference 동안 유지하고 마지막 pin 해제 시
  회수된다. Retained immutable entry snapshot은 Arc handle을 공유하며 mutable roster/checkpoint와 alias하지 않는다.
  Unique restore는 owned state를 move하고 shared restore는 full resource/relic/UUID preflight와 bounded deep copy를
  live write 전에 마친다. 거절 시 기존 live owner와 accepted state를 보존한다.
- `WorldLabPreparation`의 duplicated bulk roster는 ephemeral Lab simulation 경계다. Actual normal preparation,
  command/clock/effect와 in-memory Retreat/reentry를 사용하지만 checkpoint hash/export가 ordinary Continue 호환을
  보장하지 않는다. 정상 Continue의 exact recruitment/roster ownership 검증은 그대로 유지한다.

## Active Battle Lifecycle

이 장은 active battle의 관측 로그, setup/checkpoint presentation과 CombatResult 종료 경계를
소유한다. 플레이어-facing 성공·실패 의미는 [game_rulebook.md](game_rulebook.md), 정확한 wire
payload와 message order는 외부 Unity canonical을 따른다.

### Faulted Published Battle

`CoreApplication::battle_fault()`는 이미 게시된 live battle에서 발생한 최초 runtime 오류를
application 수명 동안 보존한다. Capsule은 마지막 accepted battle UUID/revision, 작업 종류와
닫힌 native/expression/internal code를 소유하며 동적 cause 문자열이나 trace 이력을 보관하지 않는다.
GameCore candidate rollback 밖에서 raw domain 오류를 진단으로 바꾸기 전에 기록하므로 진단 생성이
실패해도 최초 원인과 gate는 남는다. Gameplay root/RNG/event/resources는 마지막 commit을 유지한다.

Native 한도·내부 불변식, trusted battle 데이터 불일치와 실제 runtime 수식 오류가 대상이다.
일반 authority/action/target/input 거절, resource 부족·예약 충돌, read-only/profile/admin 입력 오류,
미게시 전투 진입 candidate 실패와 domain commit 이후의 public projection 실패는 이 상태를 만들지 않는다.
사용자 입력의 존재하지 않는 skill ID는 기존 invalid_static_data wire 거절을 유지하며 terminal이 아니다.

Fault 이후 모든 gameplay/advance/host tick/next event/disconnect pause, admin/profile/reset,
generation 교체, prepare/rollback capture/restore, Continue와 checkpoint export는 `battle_faulted`로
거절한다. `reset_player_profile`은 checked Result를 반환한다. 고정 크기 fault query와 read-only
snapshot은 허용한다. Run snapshot은 `battle_fault`를 포함하고 action capabilities를 비우며 retreat와
deployment를 광고하지 않는다. 정상 application은 이 필드를 추가하지 않는다. Fault fork도 같은 원인을
유지하며 private speculative candidate 실패는 accepted parent로 전파되지 않는다.

Host는 session authority 확인 뒤 Core gate를 사용한다. 첫 manual/tick 오류에서 timer를 취소하고,
Ensure 및 이미 예약된 callback도 gate를 확인한다. 오류 actor는 detach/reconnect/세션 교체 뒤에도
같은 상태를 유지하며 active battle의 기존 무TTL 수명을 따른다. 최소 host state_snapshot은
`game_state: battle_faulted`, `battle_fault`, 빈 `action_capabilities`,
`recovery: quit_then_continue_valid_checkpoint`다. 이 조회에는 profile sync, live setup projection,
backup 복구를 쓸 수 있는 store.load가 필요하지 않다.

해당 actor의 관리 명령·checkpoint/profile writer·stopped writer는 모두 닫는다. 다른 actor와 공유 store를
전역 차단하지 않는다. 명시적 websocket Quit로 owner를 종료한 뒤 새 owner에서 기존 Continue만 사용할
수 있다. Valid checkpoint의 기존 content/profile 검증과 loads_used 의미를 유지하고, 부재·손상·부적합
저장은 기존 오류로 거절한다. 자동 재개, 같은 전투 재시작, 오류를 정상 패배/보상으로 정산하는 경로는 없다.
내부 직원 정합성·closed lifecycle 검증·이미 성공한 native 명령의 잘못된 outcome은
생산 지점에서 typed invariant로 구분한다. 소유한 event dispatch와 boss runtime 경계에서
처리되지 않고 전파된 내부 InvalidAction/UnitNotFound도 typed invariant로 바뀐다.
사용자가 지정한 없는 직원·잘못된 sustained activation은 일반 거부를 유지하며,
문자열이나 공유 오류형 전체를 근거로 terminal을 추정하지 않는다. 기존 expression/native
첫 원인은 더 뒤의 내부 오류로 덮어쓰지 않는다.

Fault는 save schema에 기록하거나 기존 파일을 삭제·변환하는 이유가 아니다.

Observer는 최초 실패 summary/work/peak와 lease를 유지하면서 current battle/usage를 별도로 비운다.
commit counter를 올리지 않는다. 수집기의 active aggregate와 selected slot은 기존 종료 절차로 정리되며
반복 fault gate는 gameplay나 실패 summary를 다시 실행하지 않는다. 관측 연결·부재·포화는 격리 결과를
바꾸지 않는다. 구현·검증 범위는 [U03 실행 계획](goals/core_runtime_fault_isolation/execution/single_goal/PLAN.md)에 연결한다.

### Battle Event Log Boundary

`BattleEventLog`는 live `DefenseRoute` 전투의 sequence-monotonic bounded 관측 window다. 보존된 entry는
immutable이지만 오래된 entry는 아래 상한에 따라 제거된다. 전투 전체를 미리 산출하는 replay source가
아니며, gameplay 판정을 되돌려 계산하는 입력도 아니다.

- `battle_update.events_delta`는 아직 클라이언트가 presentation event log에 적용하지 않은 event log entry 구간이다.
- 전투 중 정확한 연출 타이밍 source는 `battle_update.events_delta`다.
- 전투 중 현재 상태/복구 source는 `battle_update.checkpoint`다.
- Unity는 event로 현재 상태를 계산하지 않고, checkpoint로 연출 타이밍을 만들지 않는다.
- `checkpoint.at_seq`는 그 seq까지의 event를 presentation timeline에서 처리한 뒤 reconcile할 권위 상태를 뜻한다.
- seq가 부여된 live presentation event는 이후 수정하지 않는다.
- Active diagnostic window는 event `8,192`개, compact JSON entry-array `4 MiB`(bracket/comma 포함),
  retained battle-time span `180,000 ms`를 각각 inclusive 상한으로 삼는다. Append 뒤 어느 한 상한이라도
  넘으면 모든 상한을 만족할 때까지 oldest entry를 제거한다. Exact cap은 유효하고 cap+1은 결정적으로
  eviction/drop을 일으키며, 단일 entry가 byte cap을 넘거나 encode/accounting할 수 없으면 그 diagnostic
  entry를 보존하지 않는다.
- `180,000 ms`는 history window이지 battle timeout이 아니다. History eviction, 단일 entry drop 또는
  diagnostic sequence exhaustion은 simulation, winner/outcome, participant state, result aggregate와 다음 Core
  decision을 중단·실패·rollback하지 않는다.
- Retained entry와 별도로 `floor_seq`와 `latest_seq`를 monotonic하게 추적한다. Cursor가
  `floor_seq <= after_seq <= latest_seq`이면 `partition_point`로 exact bounded delta를 반환한다.
  `after_seq < floor_seq`는 floor/latest를 포함한 typed `BattleSnapshotResyncRequired`이며,
  `after_seq > latest_seq`는 기존 typed `InvalidBattleResyncSeq`다. Setup-loss recovery와 이미 사전 검증을
  통과한 command가 실행 중 floor를 전진시킨 경우에는 `snapshot_resync_required` update와 현재 checkpoint를
  반환한다. 동일 cursor의 반복 projection은 gap/duplicate 없이 동일해야 한다.

### Playback Clock And Mutation Ordering

`BattlePlaybackState`는 active battle의 paused flag와 stored speed를 소유한다. Production simulation
advance는 pause-aware server tick 경계를 통과하며 paused이면 `BattleCore` event queue에 delta를 전달하지
않는다. Raw deterministic advance 함수는 test/domain driver primitive이지 playback authority가 아니며,
production caller가 Pause를 우회하는 용도로 사용하면 안 된다.

- Active battle mutation은 exact `battle_uuid`와 `battle_state_revision`을 domain mutation 전에 검증한다.
- Server actor는 command를 한 번에 하나씩 처리한다. 각 성공 mutation은 즉시 state/event를 commit하고
  revision을 정확히 1 증가시키며, 실패는 state/resource/cooldown/event/revision을 변경하지 않는다.
- 여러 command를 하나의 gameplay batch로 묶거나 뒤 실패 때문에 앞 성공을 rollback하지 않는다. 두
  request가 같은 revision을 제출하면 먼저 commit된 요청 뒤의 요청은 stale authority로 거부한다.
- Pause/Resume, 배치, 철수, manual cast start와 stored speed 변경 같은 command-time mutation은 frozen
  battle time에도 commit할 수 있다. Scheduled attack/cast resolve, projectile/area, buff tick/expiry,
  cooldown과 회복은 Pause 중 진행하지 않고 Resume 뒤 catch-up하지 않는다.
- `SetBattleSpeed`는 Pause 중에도 stored speed와 revision을 바꾸지만 clock은 frozen 상태로 유지한다.
- Read-only battle state/setup recovery/preview는 현재 checkpoint를 반환하고 revision을 소비하지 않는다.
- 같은 battle time의 semantic event 순서는 append-only event seq가 소유한다. `source_command_id`는 request
  correlation이며 event ordering이나 gameplay RNG authority가 아니다.
- Host disconnect는 process-local active battle을 idempotent하게 자동 Pause하며 TTL이나 background
  simulation을 시작하지 않는다. 같은 server process에 다시 attach하면 paused flag, stored speed, 같은
  battle time/revision, event cursor와 spawn/compendium ledger를 그대로 투영하고, clock 재개는 명시적
  `ResumeBattle`만 수행한다. 제출되지 않은 local plan/selection은 저장하거나 복구하지 않는다.

### Terminal And Same-Time Event Authority

`BattleExecutionState`가 stable `BattleOutcomeId`, `BattleVerdict`, terminal time과 finalized 여부의 단일
authority다. Direct, until-time, elapsed/by와 world/server advance는 같은 named-outcome evaluator를 사용한다.

- Encounter는 reachable Victory를 최소 하나 포함한 closed terminal outcome 목록을 authoring한다. Core에는
  system max-time, implicit Draw 또는 caller가 주입하는 winner fallback이 없다. 시간 종료가 필요하면
  `TimeReached` outcome을 데이터에 명시한다.
- 같은-time event bucket은 spawn Kernel, Wave와 summon producer settlement를 모두 drain한 뒤 terminal을
  평가한다. 아직 spawn되지 않은 required Wave도 Victory를 보류한다.
- 동시에 여러 condition이 참이면 Defeat가 Victory보다 먼저이고, 그 뒤 authored priority 내림차순과 stable
  outcome ID 오름차순으로 정확히 하나를 고른다. Committed outcome은 한 번 저장되며 result/node/reward/
  progression consumer가 같은 ID의 exactly-one policy를 atomic/idempotent하게 소비한다.
- 모든 `BattleEvent`는 time, 기존 semantic priority band, 명시적 variant discriminator, 안정적인
  instance/cast/step/generation key와 complete payload fallback의 total order를 가진다. Unequal event는
  `Equal`로 비교되지 않으며 insertion/hash iteration/pointer/RNG는 ordering input이 아니다.
- 기존 priority band는 유지한다. 같은 priority-8 band에서는 `AutoCastEnd`가 `ManualCastEnd`보다 먼저이고,
  같은 variant의 나머지 tie는 complete payload로 결정한다.

### Atomic Battle Advance Transaction

Elapsed, pause-aware server tick과 raw next-event-bucket advance는 하나의 staged transaction을 사용한다.
Transaction은 active battle(Core/queue/log/execution/employee/deployment/tactical/playback 포함), VN runtime과
consumed state, UUID continuation 및 finished transition projection을 현재 state에서 stage한다.

- Cursor와 revision capacity는 staged Core mutation 전에 검증한다. Server tick의 fractional playback
  remainder도 같은 stage 안에서 계산한다.
- Core step/finalize, BattleStarted/BattleTimeReached trigger, employee/tactical reconciliation, exact DTO와
  optional CombatResult transition이 모두 성공한 뒤 stage를 한 번 publish한다.
- 성공 advance는 `battle_state_revision`을 정확히 1 증가시키고 terminal이면 그 update를 만든 뒤
  `CombatResult`로 전환한다. Paused/fractional tick이 simulation delta를 만들지 않으면 기존 no-update 의미를
  보존한다.
- Future cursor, revision overflow, trigger lookup/session, employee reconcile 또는 projection 실패는 clock,
  terminal/finalized state, unit/cast/buff/projectile/movement state, queue/log, employee/deployment/tactical,
  playback remainder, VN/consumed, UUID, revision, GameState와 active content를 전혀 변경하지 않는다.
- Debug combat trace export는 여전히 non-authoritative이며 export 실패가 committed gameplay transition을
  막지 않는다.

### Manual Skill Targeting Focus Authority

`SkillCastSelection`은 effect `SkillTarget`과 독립된 required cast-selection authority다. Live RON은
`self`, `automatic_unit`, `player_unit`, `player_tile` 중 하나를 명시하며 runtime은 first step, target kind,
weapon profile이나 payload 존재 여부에서 selection을 추론하지 않는다. `SelfUnit/SelfUnit`,
`AutomaticUnit|PlayerUnit/EnemySingle|AllySingle`, `PlayerTile/CastTarget` 조합만 publication된다.

- Self/AutomaticUnit activation은 focus 없이 처리한다. PlayerUnit/PlayerTile은 먼저
  `validate_manual_skill_targeting_focus`의 caster/manual/Attack/activity/control/resonance gate를 통과해
  `LiveBattleTacticalFocusDto { purpose, caster_unit_instance_id, skill_id, restore_playback }`를 active battle에
  투영한다. 내부 skill focus는 caster/skill만 소유하고 playback 복원은 공통 interaction이 소유한다.
  다른 skill focus 및 focus 없는 player-selected activation은 거절하며 일반/배치 선택과는 중첩할 수 있다.
- Focus 시작은 한 projection commit이다. 첫 일반/배치/스킬 선택이 공통 restore playback을 저장하고,
  선택이 하나라도 남으면 표시·실효 speed는 `X0_5`다. Pause는 유지한다. Pause/Resume은 최종 정지 의도를
  갱신하고 선택 중 SetSpeed는 `SelectionActive`로 거절한다. 마지막 선택 종료만 저장된 speed를 복원한다.
- `PreviewSkillTarget`은 matching focus와 current revision에서 final validator를 실행해 typed eligibility만
  반환한다. State/event/revision을 바꾸지 않으며 final activation은 candidate를 다시 검증한다. Missing/type
  mismatch/invalid target과 모든 caster failure는 focus/playback/resonance/cooldown/event/revision 불변이다.
- Successful player-selected activation은 `ManualCastStart`, focus 제거와 exact playback 복원을 하나의
  `battle_state_revision + 1` commit으로 만든다. Cancel은 cast/event/cost 없이 focus를 제거하고 restore
  playback을 한 commit으로 복원한다. Caster가 active deployment에서 사라지면 reconcile이 같은 restore
  규칙으로 stale focus를 제거한다.
- `PendingSkillCast`는 cast target identity와 acceptance 시점 target tile을 함께 보존한다. 직원의 필수 Direct
  ReuseCastTarget은 실행 시 unit lifecycle/hostility/air eligibility가 무효면 남은 사용 동작을 취소하고 retarget하지 않는다.
  TileArea는 stored accepted tile을 사용하므로 target 이동/사망/철수 뒤에도 geometry가 바뀌지 않는다.
  Cast completion은 target 무효화 때문에 resonance나 이미 committed state를 refund/rollback하지 않는다.
- PlayerUnit/PlayerTile skill의 어떤 step에도 `RetargetOnStep`을 허용하지 않는다. 직원 manual fragment는
  Self/AutomaticUnit selection에서도 접수 후 retarget를 허용하지 않으며, autonomous skill의 기존 retarget semantics는 유지한다.
- `LiveBattleStateCheckpointDto.tactical_focus`와 root InBattle checkpoint는 focus purpose/caster/skill/restore
  playback을 그대로 투영한다. Reconnect는 같은 active session을 읽고 hover/local candidate는 저장하지
  않는다. Focus는 durable run checkpoint/save payload가 아니므로 save migration은 없다.
- Focus begin/cancel/success battle update는 다른 mutation update와 같은 refreshed action capability slice를
  가진다. Preview는 `SkillTargetPreview` read-only command result이고 side battle update나 trailing root
  snapshot을 만들지 않는다. Skill catalog shape는 selection 추가로 version `3`이다.

### Tactical Resource Authority

`ActiveBattleSession::tactical_resource`의 `TacticalResourceState`가 player-side battle-global current,
maximum, natural-recovery cursor와 deploy/redeploy price authority를 소유한다. `LiveBattleDeploymentState`는
cooldown, instance salt, deployed mapping과 redeploy lock만 소유하며 resource arithmetic을 복제하지 않는다.

- Battle start는 required `RunPolicyData.tactical_resource`에서 initial/maximum/recovery와 price tuning을
  복사한다. Current mutation은 `recover_until`, full `pay`와 signed `apply_delta` 경계만 사용하고 모두
  saturating/clamped arithmetic이다.
- Natural recovery는 `BattleExecutionState::last_event_time_ms()`의 whole-second cadence를 소비한다. Capped
  interval도 recovery cursor를 전진시켜 excess를 bank하지 않으며 large elapsed/rate product는 wrap하지
  않는다. Pause-aware server tick이 battle time을 동결하므로 wall-time catch-up 경로가 없다.
- Deploy preflight는 candidate effective cost 전액을 확인한다. Core deploy가 성공한 같은 serialized
  transaction에서 정확히 한 번 지불하며, validation/Core failure는 resource, salt, mapping, lock, event와
  revision을 보존한다. Withdrawal은 pool을 변경하지 않고 redeploy lock의 base price만 고정한다.
- `SkillEffectDef::ModifyTacticalResource`는 Core에서 `BattleLiveSignal::TacticalResourceDelta`를 만들고 world
  boundary가 current에만 적용한다. Battle core는 별도 resource copy나 maximum mutation을 소유하지 않는다.
- `LiveBattleStateCheckpointDto.tactical_resource`와 root `GameStateContextDto::InBattle.tactical_resource`는
  required `LiveBattleTacticalResourceDto { current, maximum, recovery_per_second }`다. Deployment DTO에는
  resource current/maximum/recovery legacy field가 없고 game_server는 checkpoint shape를 그대로 운반한다.
- Reconnect/resync는 같은 process의 in-memory active session만 읽는다. Active battle은 persisted run
  checkpoint/save payload가 아니며 process restart/`ContinueRun`은 미완료 node의 진입 전 확정 상태를 복원하며 전투,
  Wave/summon state와 attempt-local compendium candidate를 복구하지 않는다. Battle result/Retreat가
  `active_battle`을 제거하면 tactical resource도 폐기된다. 따라서 active-battle save version, legacy alias,
  dual projection과 migration reader가 없다.

### Atomic Tactical World Mutation

`DeployUnit`, `WithdrawUnit`, `SetEngagementStance`, `TacticalRelocateUnit`과 `TacticalSwapUnits`는 각각 하나의
serialized world transaction이다. World는 command별 선행 payload reason을 기존 순서로 평가한 뒤 complete
`ActiveBattleSession`을 stage하고, live session을 직접 변경하지 않은 채 남은 Core/world mutation과 응답
projection을 끝낸다.

- Exact battle UUID/revision authority는 domain 진입 전에 검증한다. Shared staging boundary는 revision capacity를
  첫 staged mutation 전에 확인한다.
- Core command, event/log/queue, deployment salt/mapping/lock/facing, tactical resource, employee reconciliation,
  Phase 08 tactical-effect reconciliation, revision increment와 cursor-based battle update를 모두 stage에서 수행한다.
- Final visual-novel snapshot과 refreshed action-capability slice까지 성공한 뒤 staged session을 정확히 한 번
  publish한다. Cursor, Core command, employee reconciliation, revision 또는 final projection 실패는 Core와 모든
  adjacent battle-world state를 transaction 전 값 그대로 보존한다.
- 성공 request만 비용·revision·command event를 각각 정확히 한 번 commit한다. Pause는 battle time만 동결하며
  이 command-time transaction을 막지 않는다. 다섯 request는 서로 독립된 원자 단위이고 batch/연쇄 rollback
  의미를 만들지 않는다.
- 이 boundary는 기존 command/result/event/checkpoint nesting과 RON/wire/public Rust/save shape를 변경하지 않는다.

### Withdrawal, Detached Delivery, And Redeployment

`WithdrawUnit`은 active player `RuntimeUnit`에 대한 command-time mutation이다. 별도 action-state capability나
Hard CC/Silence/FieldWork guard를 두지 않으며 Core의 active lifecycle 검사를 통과한 unit만 한 번
`Withdrawn`으로 전환한다. Mutation은 자원을 소비하거나 환불하지 않고 unit target/action/movement/block,
pending cast와 source/target buff를 같은 commit에서 정리한다.

- `AttackResolve`는 release 경계다. Attacker가 `Withdrawn`이면 queued instant/projectile resolve를 semantic
  event 없이 폐기한다. `Dead` source의 이미 committed instant snapshot을 보존하는 기존 same-time 규칙과는
  구분한다.
- Basic projectile record와 skill `ActiveProjectileRuntime`은 launch가 끝난 detached delivery다. Source가
  철수해도 launch-time owner, target capability와 `DamageSourceSnapshot`으로 진행한다. Target이 철수하면
  active target 검사를 통과하지 못해 miss/targetless impact가 된다.
- Persistent tile area는 생성 시 source damage context와 facing을 snapshot한다. `GroundFixed`는 caster cast
  runtime이 제거된 뒤에도 저장된 geometry/source context로 tick/expire한다. `FollowCaster`와 `FollowTarget`
  area는 source-bound이므로 철수 commit에서 제거하며 queued tick/expire event는 runtime lookup 실패로
  no-op한다.
- Withdrawal은 caster의 `ActiveSkillCast`를 제거한다. 이후 도착한 queued `SkillStep`은 active cast key가
  없으면 no-op하므로 delayed/conditional/unreleased step이 다시 materialize되지 않는다. 이미 만들어진
  detached projectile/GroundFixed event는 active cast 없이도 자신의 snapshot으로 resolve한다.
- 자발적 철수 lock은 철수 순간의
  `FixedCurrentHp(min(max_hp, current_hp + floor(max_hp * 30 / 100)))`, `ready_at_ms = battle_time + 30_000`과
  redeploy base cost를 저장한다. 전투불능 reconciliation은 current HP override 없이 최신 attempt employee
  condition에서 HP를 다시 계산하고 `ready_at_ms = battle_time + 90_000`을 저장한다.
- 두 lock 생성 경로는 하나의 overflow-safe
  `ceil(base_deploy_cost * redeploy_cost_multiplier_pct / 100)` 계산을 공유한다. Candidate projection과 deploy
  commit은 저장된 base cost에 employee `DeployCostReduction`을 적용한다.
- Incapacitation ledger key는 `UnitInstanceId`다. 같은 instance penalty는 idempotent하고 재배치한 새 instance의
  후속 전투불능은 새 penalty다. Redeploy draft는 attempt-local roster/loadout/condition에서 다시 만들어지며
  old runtime transient state를 복사하지 않는다.
- Official live checkpoint는 active unit만 투영하고 deployment projection이 withdrawn/dead employee의
  ready time, base/effective cost와 eligibility를 소유한다. Pause는 battle time을 멈추므로 30초/90초 lock도
  멈추고 reconnect는 같은 revision/time/lock 값을 반환한다. 이 lifecycle correction은 새 DTO, save schema,
  compatibility/fallback path를 만들지 않는다.

### Tactical Stance And Derived Tile Effects

`BattleCore::tactical_states`는 live-deployed player unit별 `UnitTacticalState`를 소유한다. 저장 authority는
`EngagementStance::Attack | Defense`, deployment tile kind, equipped `WeaponCombatProfile`, base block,
cooldown deadline와 Rest scheduler bookkeeping뿐이다. `DefenseFocus`/`Rest`는 저장 variant가 아니라
최종 activity+stance+tile+weapon profile에서 파생한다.

- Deploy command는 world의 validated deployment zone을 `TacticalTileKind`로 Core에 전달한다. Core는 weapon
  profile을 draft에서 다시 확정하고 Attack, `time_ms + stance_change_cooldown_ms`로 초기화한다. Withdraw는
  tactical state를 제거하고 redeploy는 새 instance/state/cooldown을 만든다.
- `stance_change_disabled_reason`이 active player combatant, tactical/weapon state, same target, derived Hard CC,
  cooldown을 mutation 전에 검사한다. `SetEngagementStance` 성공만 modifier/block/rest epoch를 갱신하고
  `StanceChanged`를 기록한다.
- Stance, relocation/swap과 field-work lifecycle은 논리 activity/stance/tile을 먼저 확정한 뒤 하나의 typed
  final-state reconciliation으로 이전 효과와 최종 효과를 비교한다. 이 authority만 Ground Focus aggregate,
  exact block capacity와 Platform Rest epoch/deadline을 변경하고 tactical snapshot도 같은 파생을 사용한다.
- Defense admission은 regular `AttackStart(schedule_next=true)`, pending auto cast와 manual cast validation에만
  적용한다. Pending intent는 보존하며 Attack 복귀 시 basic attack/auto cast를 재평가한다. 이미 생성된
  `AttackResolve`, pending cast end, active cast/delivery와 triggered `schedule_next=false` attack은 gate를 다시
  적용하지 않는다.
- Ground 전환은 runtime block capacity를 base 또는 base+bonus로 설정하고 general incoming reduction에 focus
  bonus를 가감한다. Damage pipeline은 general+type reduction을 기존 final clamp에 넣는다. Block refresh는
  기존 engagement도 `compare_enemy_block_priority`로 정렬하여 capacity shrink survivor를 결정한 뒤 남은
  capacity에만 새 적을 넣는다. Derived Hard CC는 projection의 effective block을 0으로 만들지만 focus damage
  modifier를 제거하지 않는다.
- Platform Defense는 `RestTick { time_ms, unit_instance_id, rest_epoch }`를 full interval 뒤 예약한다. Event
  priority는 모든 damage-producing projectile/area/buff/skill/attack resolve 뒤, 새 auto cast 전이다. Epoch와
  exact next tick 검사가 stale event를 no-op한다. Active Defense+Platform이면 Hard CC에서 tick 전체를 skip하고,
  아니면 positive HP delta를 실제 변화가 있을 때만 기록한 뒤 다음 absolute interval을 예약한다.
  Inactive unit은 reschedule/restore하지 않는다. Rest는 공명을 지급하지 않는다.
- 직원 공명은 Rest와 독립적인 정수 battle-second pulse에서 +1이다. Hard CC/use/cap에서 버린 pulse는
  catch-up하지 않으며 Attack/Defense/Work/Silence는 이 cadence를 바꾸지 않는다. Pause는 event queue를
  advance하지 않으므로 별도 wall-time cursor가 없다.
- Official deployed/checkpoint projection은 stance, tile/effect, cooldown deadline, Attack/Defense별 typed
  eligibility, base/effective block, active focus reduction과 next Rest tick을 Core snapshot에서 생산한다.
  Opponent/non-live units의 checkpoint tactical state는 `null`이다. Reconnect는 같은 active state를 다시
  projection하며 active battle은 durable run save payload가 아니므로 save migration은 없다.

### Tactical Relocation And Logical Occupancy

`BattleCore::validate_player_tactical_destination`은 valid/static battlefield tile과 active player
`Combatant | DefenseObject` logical occupancy를 공통 검사한다. `deploy_player_unit`이 scenario group을
추가하기 전에 이 validator를 다시 실행하므로 preview 이후 상태가 바뀌어도 비용/salt/event mutation 전에
`PositionOccupied`로 실패한다. Enemy body는 player tactical slot occupant가 아니다.

- World는 destination이 player deployment zone인지와 zone kind를 확정하고 Core에
  `TacticalTileKind`를 전달한다. Core는 source 자신을 occupancy에서 제외한 뒤 weapon affinity와 usable
  Ground/Platform profile을 다시 검사한다. Path/segment/reservation state는 생성하지 않는다.
- `tactical_relocation_disabled_reason`은 active player combatant/tactical state, derived Hard CC,
  unreleased basic windup 또는 pending skill cast, employee-local cooldown을 mutation 전에 검사한다. Silence,
  block, stance cooldown은 이 gate에 포함하지 않는다. World가 같은 preflight 뒤 global tactical resource
  cost `5`와 revision capacity를 검사한다.
- Commit은 serialized command 안에서 같은 `RuntimeUnit`의 `body.position`과 `tactical_anchor`를
  destination center로 설정하고 facing을 갱신한다. `action_state`를 Idle로 설정하고 `move_epoch`를
  한 번 증가시키며 current target과 active movement segment를 제거한다.
  HP, buff map, resonance와 runtime unit identity를 보존한다.
  작업 칸이면 즉시 FieldWorking, 그 외에는 Defense와 `ForcedDefenseEnd(time_ms + 1500, epoch)`를 설정한다.
  정확한 epoch/deadline에서 Attack으로 자동 복귀한다. Hard CC가 남아도 태세는 복귀하되 행동 gate는 유지한다.
  강제 구간에는 수동 태세를 거절하며 자동 복귀가 추가 cooldown을 만들지 않는다. Relocation cooldown은 10초다.
- relocated blocker의 기존 block 관계와 relocated unit을 참조한 target을 해제한 후 `refresh_block_state`가 새 좌표에서 capacity와
  deterministic enemy priority를 다시 계산한다. Checkpoint range와 persistent area geometry는 authoritative
  body position에서 다시 파생한다. Relocation은 active projectile/area runtime을 제거하지 않으므로 Homing은
  live target body, Fixed는 launch aim, GroundFixed는 stored geometry, FollowCaster/FollowTarget은 live body를
  계속 사용한다.
- Ground↔Platform은 Any profile만 통과한다. Body/tile과 최종 field-work assignment가 확정된 뒤 이전/최종
  derived aggregate를 한 번 reconcile해 Ground focus/block 또는 Platform Rest를 적용한다. Rest active 여부가
  바뀌거나 다른 타일로 이동하면 Rest의 새 full interval을 예약한다. Same-tile facing-only commit은 기존
  Rest cadence를 보존한다. 작업 진입/퇴장은 강제 타이머를 무효화하며 작업 퇴장은 Attack이다.
- 성공은 `UnitTacticallyRelocated`에 old/new logical/world position, facing과 cooldown deadline을 기록한다.
  Official tactical DTO는 relocation cooldown과
  Core reason에 resource availability를 합친 typed eligibility를 deployed/checkpoint/root에 동일하게 투영한다.
  Active battle은 durable run save payload가 아니므로 별도 save reader나 compatibility schema는 없다.

### Atomic Tactical Swap

`TacticalSwapUnits`는 relocation의 occupied fallback이 아닌 별도 player/Core command다. World는 서로 다른 두
roster employee가 현재 배치됐는지 확인하고 각 current position으로 destination zone kind를 해석한다. Core는
두 active Player combatant의 Hard CC/unreleased action/relocation cooldown/exclusive movement와 양쪽
destination valid/static/affinity/profile을 모두 mutation 전에 검사한다.

- Common occupancy는 제외 ID slice를 받아 participant pair만 제외한다. 따라서 pair의 서로 점유된 destination은
  허용하지만 같은 logical tile의 제3 Player `Combatant | DefenseObject`는 `PositionOccupied`다.
- `tactically_swap_units`는 양쪽 preflight가 성공한 뒤 두 blocker/target 관계를 함께 해제하고 shared relocation
  helper로 두 body, explicit facing, tile kind와 같은 cooldown deadline을 적용한다. 최종 field-work occupancy를
  reconcile한 뒤 두 unit의 이전/최종 tactical effect를 각각 한 번 적용하고, `refresh_block_state`, pending
  basic/autocast scheduling을 각각 한 번 수행한다. Recoverable validation error는 첫 unit만 바뀐 partial state를
  만들 수 없다.
- World는 existing `tactical_relocation.cost`를 한 번 지불하고 두 deployed facing을 갱신한 뒤 battle projection을
  한 번 commit한다. Per-unit relocation cooldown/eligibility DTO와 frozen Pause battle time을 그대로 재사용하며
  swap 전용 policy/save field는 없다.
- `UnitsTacticallySwapped` event는 양쪽 unit ID, old/new logical/world position, before/after facing과 shared cooldown
  deadline을 한 entry에 기록한다. Presentation
  event/result/action capability와 server behavior mapping도 같은 current-only shape이며 caller destination/path
  extension은 거부한다.

### Field Work Runtime Authority

`BattleScenario.tactical_plan.field_work_objects`는 immutable setup definition이고 `BattleCore`의 object별 runtime은
`Active | Completed`, accumulated work, last settled battle time, contributor count, schedule generation과 stable
work-cell assignment를 소유한다. `FieldWorkObjectId`와 `FieldWorkCellId`는 string identity이며 runtime snapshot은
object/cell ID와 worker `UnitInstanceId` 순서가 결정론적이다.

- Deploy/redeploy/withdraw/death/relocation/swap은 position mutation의 모든 participant를 commit한 뒤
  `reconcile_field_work`를 호출한다. Active Player Combatant가 matching tile kind의 incomplete cell을 최종 점유한
  경우만 assignment되며 별도 command, reservation 또는 pending movement state는 없다. Enemy body는 이 mapping에
  영향을 주지 않는다.
- Assignment 진입은 action engagement를 해제하고 unit activity를 `FieldWorking`, stance를 `Defense`로 만든다.
  Ground focus reduction/block 또는 Platform Rest를 제거하며 block capacity는 0이다. 이탈은 같은 runtime unit을
  `Ready + Defense`로 돌리고 현재 tile의 focus/block 또는 새 full-interval Rest를 다시 파생한다. Withdraw와
  relocation/swap command 자체는 activity 때문에 거부하지 않는다. 같은 worker가 교차 tile-kind work cell 사이를
  이동해 assignment worker 집합에는 남더라도 moved-unit final-state reconciliation이 `FieldWorking`의 무효과를
  유지한다.
- Progress settlement는 이전 settlement부터 현재 authoritative battle time까지의 delta에 cached eligible contributor
  count를 곱해 object에 더하고 required work에서 clamp한다. Assignment가 없어져도 progress는 Preserve한다.
  Hard CC apply/expiry event 전후에 먼저 settle하고 derived control capability로 contributor count만 다시 계산하므로
  suspension 중 catch-up이 없다. Pause-aware production tick은 Core clock을 전혀 진행시키지 않는다.
- Contributor count가 바뀔 때 generation을 올리고 remaining/count의 ceiling deadline에
  `FieldWorkCompletion`을 예약한다. Old generation completion은 no-op한다. Current generation이 required에 도달한
  최초 event만 status를 Completed로 바꾸고 typed repair, `FieldWorkCompleted` log와 assignment/activity 종료를 같은
  mutation에서 실행한다. Target이 full HP여서 actual repair가 0이어도 object는 소비되고 stale event는 effect/event를
  반복하지 않는다.
- Semantic log는 `FieldWorkAssignmentStarted`, `FieldWorkAssignmentStopped`, `FieldWorkCompleted`뿐이다. Progress tick은
  event가 아니며 checkpoint가 accumulated/required/status/cell-worker mapping을 소유한다. 이 strict event 추가로
  `BATTLE_EVENT_LOG_VERSION`은 `31`이다.
- `battle_setup_snapshot` version `2`는 static objects와 별도인 immutable `field_work_objects` definition을 포함한다.
  `battle_update.checkpoint.field_work_objects`는 current runtime state를 포함하고 unit tactical `activity`와 central
  `skill_start_disabled_reason: field_working`을 함께 투영한다. `RecoverBattleSetupLoss`는 같은 setup과 현재 mapping을
  revision 소비 없이 재투영한다. `game_server::PlayerGameServerMessage`는 이 Core DTO를 top-level setup/update/resync에
  그대로 전달하고 별도 assign/cancel client command를 정의하지 않는다.

Active battle은 durable run save payload가 아니므로 field-work save reader, version migration, alias 또는 compatibility
state는 없다.

### Active-effect Control Capability And Cancellation

`BuffKind::control_capability()`가 active buff metadata를 `HardCrowdControl` 또는 `Silence`로 분류하고,
`BattleCore::unit_control_state(unit_id, battle_time_ms)`가 현재 `buffs` map에서 `UnitControlState`를 매번
파생한다. Command와 movement/block consumer는 `Stun`/`Freeze`/`Silence` variant를 직접 검사하지 않는다.
Capability를 unit/save state에 별도 저장하거나 expiry를 복제하지 않는다.

- Active buff identity는 기존 `(caster_instance_id, target_instance_id, buff_id)`다. 서로 다른 identity의
  Hard CC/Silence는 동시에 존재하고 각자 자연 만료한다. Hard CC state의
  `hard_cc_release_time_ms`는 남은 Hard CC `expires_at_ms`의 최댓값이며 Silence는 active instance가 하나라도
  있으면 true다. Event scheduler가 expiry event와 같은 timestamp에 새 event를 역삽입하지 않도록 자동
  attack/cast/movement retry만 이 값의 다음 millisecond에 예약한다.
- Manual/automatic skill start는 하나의 `skill_start_disabled_reason`을 사용한다. Hard CC가 Silence보다
  먼저 `HardCrowdControl`을 반환하며 Silence 단독이면 `Silenced`다. Manual validation은 mutation 전에 이
  reason을 반환하고 automatic cast는 pending intent를 보존해 마지막 제한 expiry 뒤 다시 평가한다.
- Automatic basic attack start는 같은 Hard CC query를 사용한다. 시작된 automatic windup은
  `basic_attack_epoch`와 owned windup deadline으로 식별하며 Hard CC 적용은 epoch를 증가시키고 stale
  `AttackResolve`를 semantic event 없이 no-op한다. Windup이 소유한 basic/movement lock만 deadline exact
  match일 때 해제해 다른 긴 lock을 줄이지 않는다.
- Pending skill cast/focus는 Hard CC 적용 transaction에서 제거하고 기존 `SkillCastInterrupted` event로
  원래 start seq와 interrupter를 기록한다. Cast가 소유한 `next_action_time`, resonance/basic/movement
  focus lock만 해제한다. Auto mode는 pending intent를 다시 세우고 manual mode는 새 command를 기다린다.
  Stale `AutoCastEnd`/`ManualCastEnd`는 pending start-seq mismatch로 no-op한다.
- Release된 basic projectile, `ActiveProjectileRuntime`, `AreaRuntime`, queued buff/effect는
  control cleanup 대상이 아니다. 아직 실행하지 않은 skill continuation/delayed step과 직원 use가
  소유한 extra attack은 Hard CC가 취소한다. 이미 release된 carrier는 계속 처리한다. 독립 reactive
  attack은 해당 직원 use control의 소유가 아니므로 그 취소 대상에서 제외한다. Hard CC는 active proc state의 absolute
  `next_ready_ms`, negative damage/effect 처리와 battle clock을 수정하지 않고 positive `add_resonance`만
  제한한다.
- Block refresh는 blocker와 blockable enemy 양쪽의 derived Hard CC를 검사한다. 적용 즉시 기존 engagement를
  제거하고 중첩 Hard CC가 모두 만료된 뒤 current positions/capacity에서 재계산한다. Silence는 block state를
  바꾸지 않는다. Continuous movement input도 같은 derived Hard CC release query를 사용한다.
- Official active-battle checkpoint의 각 active unit은 필수 `control` object를 가진다:
  `hard_cc_active`, `hard_cc_release_time_ms`, `silenced`, `skill_start_disabled_reason`. 이 값은 checkpoint
  battle time과 active buff authority에서 파생하며 overlap reason은 Hard CC 우선이다. 구 `control` 누락
  shape를 default/alias로 받지 않는 strict cutover이고 game_server는 Core DTO를 그대로 transport한다.
  Run checkpoint/save는 active battle 자체를 저장하지 않으므로 별도 persisted capability나 migration은 없다.
- Live buff RON은 이미 `Stun`, `Freeze`, `Silence` kind와 `max_stacks: 1`을 표현하므로 data schema migration이
  없다. Existing `SkillCastInterrupted`와 buff applied/expired log shape를 재사용해 battle event-log version도
  바꾸지 않는다.

### Battle Setup And Movement Presentation

전투 시작 scene construction source는 `battle_setup_snapshot`이다. 전투 중 이동 연출 source는 checkpoint position이 아니라 movement event다.

- 유닛 이동 연출은 `MovementSegmentStarted`/`MovementStopped` event를 기준으로 한다.
- 실제 위치 변화가 있는 movement tick은 Unity-facing event로 노출된다.
- `checkpoint.units[*].world_position`은 이동 연출 source가 아니라 event 처리 후 보정할 reconcile target이다.
- 유닛 철수/사망 연출 위치는 `UnitWithdrawn`/`UnitDied` event의 `world_position`과 projected tile `position`을 기준으로 한다.
- 철수/사망한 유닛은 official checkpoint `units`에서 빠질 수 있으므로, Unity는 checkpoint에서 inactive unit을 찾아 연출 위치를 추론하지 않는다.
- `battle_update.checkpoint.units`는 official gameplay presentation/reconcile source이며 Active unit만 포함한다.
- inactive runtime unit 조회가 필요하면 debug/admin/replay 전용 query를 사용하고, 해당 query의 결과를 gameplay 표시 source로 사용하지 않는다.

#### Route And Battlefield Projection

- `BattlefieldRoute.cells`는 적이 실제로 따라가는 route path다. Unity는 `start`/`end`만 보고 경로를
  재계산하지 않고 setup snapshot의 `routes[*].cells`를 scene route 표시와 이동 연출 기준으로 사용한다.
- route cells는 유효 타일 위에 있어야 하며 static obstacle과 void tile을 지나면 안 된다. 연속된
  cell은 상하좌우 cardinal-adjacent여야 한다.
- authored route가 없는 Defense 템플릿은 core가 deterministic generated route를 만들 수 있다.
  Generated route도 같은 cell 조건을 만족해야 하며 `[start, end]`만 가진 대각선/직선 fallback은
  공식 계약이 아니다.
- `battle_setup_snapshot.routes`, runtime enemy movement plan, movement event/checkpoint는 같은 route
  cells에서 파생되어야 한다.
- 전투 이동과 충돌은 2D 연속좌표 기반이며 static obstacle과 비사각형 전장의 void tile은 이동 충돌
  입력으로 투영된다. 배치 입력과 range 후보 선별은 별도의 타일 기반 계약을 따른다.

### Active Battle Result And Diagnostic Sources

게임은 player-facing 또는 authoritative gameplay 전투 history를 생성하지 않는다.

- Bounded event window는 active battle의 test/debug diagnostic capability다. Participant result와 incremental
  result aggregate는 authoritative active-battle state다. 해당 content가 종료되면 run/save/checkpoint/profile
  상태에 diagnostic history를 보존하지 않는다.
- 선택적 개발자 trace는 실제 `battle_uuid`를 key로 `target/combat_traces/run_<seed>/<battle_uuid>.json`에 쓴다. 이 파일은 save/checkpoint/gameplay state가 아니며 serialization/filesystem 실패는 warning만 남기고 전투 결과 전환을 실패시키지 않는다.
- story document collection은 전투 history와 독립된 시스템이다.
- 전투 결과 화면의 1차 통계 source는 Core가 event 발생 시 diagnostic append보다 먼저 갱신하는 bounded
  `BattleResultStatsAccumulator`와 terminal `ParticipantBattleResult`에서 산출한
  `selected_event.result_stats`다. Retained event window는 통계 authority가 아니다.
- 전술적 전투불능은 player `RuntimeUnit` instance별 idempotence ledger로 직원의 attempt-local condition에 정확히 한 번 반영한다. 재배치는 그 working 직원 상태에서 새 draft를 만들며 별도 death HP 비율 override를 사용하지 않는다.
- 전술적 `RuntimeUnit` 사망과 persistent 직원 사망은 같은 상태가 아니다. `apply_incapacitation`은 instance마다 physical damage를 1, 정책 trauma를 한 번 더하고, physical damage 6만 persistent 사망이다. physical damage 5 또는 trauma 100인 직원은 살아 있지만 combat eligibility가 없으므로 새 runtime instance를 만들 수 없다.
- 완료된 결과는 attempt-local condition을 다시 패널티 계산하지 않고 roster에 한 번 커밋한다. 직원 condition, event의 `OnAnyResolution` 효과, 전투 단위 소비 아이템 duration은 하나의 staged result state에서 적용한 뒤 한 번 커밋한다.
- `RetreatBattle`은 전투 진입 전 world snapshot으로 roster/inventory/resource, 소비 아이템, node/session/content, RNG/UUID, VN/trigger와 SavePoint를 복원한 뒤 성공한 후퇴 사용량을 한 번 증가시켜 `NodeConfirm`으로 돌아간다. 후퇴는 두 번까지 허용되고 세 번째 도전에서는 불가하다. 이 사용량은 재시작/Continue에도 유지되며 CombatResult/node completion을 만들지 않는다. 명시적 SavePoint load만 그 시점의 run-local 후퇴 상태로 되감는다.
- 커밋 결과가 `NoLivingEmployees`면 남은 attempt보다 failure 후보 판정이 우선한다. `NoDeployableEmployees`는 실패 node 완료 후 현재 보유 Physical Treatment, 현재 progression에서 즉시 선택 가능한 SavePoint/Gate, runtime candidate catalog가 실제 채용 후보를 제공하는 `HeadquartersContact`를 하나의 evaluator로 계산한다. `combat_recovery_restriction` projection과 `SelectMapNode`/loadout/consumable validation은 같은 제한을 사용하고 허용 행동 완료 후 전체 deployability와 recovery path를 다시 계산한다.
- 강제 BossOmen capstone 승리 completion은 terminal Floor advance를 계획하지 않는다. Authored capstone의 forced boss를 완료/clear하고 생성 때 precommit된 concealed Exit Gate와 connection을 공개해 현재 Floor의 유일한 selectable successor로 staged commit하며, 기존 Gate confirm만 `GateTransition` recovery와 Floor advance를 수행한다.
- 모든 gameplay failure 후보는 결과 transaction을 먼저 커밋한다. 유효하고 `loads_used < 3`인 checkpoint가 있으면 `RunFailed` 전에 `AwaitingRunFailureDecision`으로 전환해 `LoadRunCheckpoint`, `ConfirmRunFailure`만 투영한다. checkpoint가 없거나 loadable하지 않으면 즉시 failure를 확정한다. BossOmen 전용 post-`RunFailed` load 예외는 없다.
- 직원 카드의 화면상 나열 순서는 gameplay 상태가 아니다. Core는 직원 보유·생사·출전 가능 여부와 전투 배치를 계속 소유하지만 표시 순서, 표시 슬롯, 표시 순서 변경 command/capability/result를 저장하거나 투영하지 않는다. 따라서 checkpoint load와 battle retreat는 Unity의 로컬 표시 순서를 되돌리지 않는다.
- `LoadRunCheckpoint`와 `RecoverBattleSetupLoss`는 서로 다른 recovery command다. 전자는 Core가 root capability를 투영한 경우에만 checkpoint를 소비하고, 후자는 active battle의 immutable setup과 현재 revisioned projection을 읽기 전용으로 다시 보내며 revision을 소비하거나 mutation authority를 요구하지 않는다.
- 성공한 `LoadRunCheckpoint`는 checkpoint-owned state 복원과 `loads_used + 1`을 하나의 Core
  transaction으로 적용한다. `loads_used`는 checkpoint payload로 복원하지 않는 run-scoped monotonic
  metadata이며 실패한 load, SavePoint overwrite, restart와 backup recovery에서 감소하지 않는다.
- `power_model_version`과 `current_floor_power_anchor`는 checkpoint payload가 map/node difficulty와 같은
  generation으로 소유하고 load/continue에서 exact 복원한다. `run_peak_certified_power`만 checkpoint 밖
  power high-water다. Core transaction rollback은 peak만 max-merge하고 anchor/model generation은 rollback
  snapshot으로 복원한다.

### Durable active-run host boundary

현재 형식과 지원 범위의 단일 설명은 다음 표다. 다른 절의 도입 당시 버전이나 과거 묵시 읽기 규칙을
현재 migration 지원으로 해석하지 않는다. 버전 숫자가 같아도 서로 다른 root·owner의 문서는 교환할 수 없다.

| 경계 | 현재 형식·생산자 | 읽기·복원 범위 |
| --- | --- | --- |
| Core Continue | `active_run/continuation.rs::VERSION` V17; `seal_document`, 확정 cache export | `restore_continue_document`와 readonly `inspect_requirements`는 V17 Continue 문서만 수용. Checksum·content 및 각 경계의 검증을 수행 |
| 독립 checkpoint 표현 | `active_run.rs::ACTIVE_RUN_CORE_PAYLOAD_VERSION` V17; private `encode_active_run_checkpoint` | 정상 Continue export/import 형식이 아님. 명시적 SavePoint load는 보유한 `RunCheckpointPayload`를 복원하며 독립 checkpoint bytes를 Continue로 읽지 않음 |
| Host active-run 봉투 | `ActiveRunStore` V7; opaque Core bytes·loads/peak·revision·required configuration | Low-level store는 Core payload header 12/14/15/16/17을 운반할 수 있음. Approved host는 추가로 Core의 현재 requirements inspection과 승인 configuration을 검사하므로 이전 payload의 게임 재개 허용을 뜻하지 않음 |
| Permanent profile | `PlayerProfileEnvelope` V6 | 현재 reader는 V6만 수용. 다른 version을 빈 신규 지식으로 읽거나 union migration하지 않음 |
| Paired application journal | `ApplicationIntent` / `IntentEnvelope` V4 | 현재 reader는 V4만 수용. 다른 version은 `ProtectedOriginal`로 거절하며 자동 replay·변환하지 않음 |

- Core Continue export는 최신 확정 상태를 strict V17 document로 직렬화한다. Root는 `version`,
  `continuation`, `continuation_sha256`만 허용한다. Continuation은 current run-local graph/content identity/
  power/resource/record/RNG 상태, 독립적인 optional SavePoint, phase, recovery restriction, compact result,
  pending reward와 원 grant context/return phase, VN runtime/trigger 상태를 소유한다. Nullable 필드도
  반드시 존재해야 한다. Permanent profile 전체와 transport session/root player 객체, live battle/event log는 넣지 않는다.
  다만 run progression의 `ExpeditionInstance`는 소유자 player ID·sequence·UUID를 저장한다. Continue는 이 player ID를
  호출자와 대조하고 원정 instance가 authoritative profile의 active marker·vault ownership과 일치하는지 검증한다.
  Root player는 복원 호출의 identity로 초기화되며, 저장 graph의 원정 소유자 binding을 대체하거나 생략하지 않는다.
  SavePoint는 current와 정확히 같으면 `{current:{}}`, 다르면 `{snapshot:{checkpoint:...}}`로 인코딩한다.
  Import는 reference를 독립 SavePoint로 materialize하고 이후 current mutation과 명시적 rewind를 분리한다.
  독립 checkpoint Core payload encoder는 V17이며 root가 `version`, `checkpoint`,
  `checkpoint_sha256`이다. 이는 외부 Continue document V17과 별도 형식이다.
  V7/V8 legacy converter는 현재 미지원으로 명시 거절하며 새 저장 bytes를 만들지 않는다.
- Checksum은 모든 object의 lexical key order와 `serde_json` scalar spelling을 사용한다. Array 순서는
  유지하되 checkpoint의 set인 VN consumed keys와 purchased merchandise는 canonical 정렬한다.
  Import는 checksum/content identity, required/unknown fields와 version, graph/seed/economy/resource,
  result/ack/phase/choice context를 검증한다. 불일치 시 `InvalidStaticData`로 거절하며 구형 decoder로
  재시도하거나 embedded 값을 추정하지 않는다. 이전 형식의 의미 회귀 검사는 V10 fixture로 계승한다.
- `ContinueRun`은 최신 확정 상태를 복원하며 명시적 SavePoint load 횟수를 소비하지 않는다. 첫 SavePoint
  이전에도 사용할 수 있다. 미완료 battle/shop/event/reward 등은 해당 node 진입 전으로 돌아가며,
  성공한 후퇴 사용량은 보존한다. 완료 결과 화면과 필수 선택은 저장된 compact receipt를 복원한다.
  `LoadRunCheckpoint`는 별도의 SavePoint payload로 되감고 외부 monotonic `loads_used`를 증가시킨다.
- 전투 종료 transaction에서 직원 condition/death/consumable, 무조건 보상, node/encounter 진행과 profile을
  한 번 확정한다. `CompleteCombatResult`는 저장된 결과를 확인하고 다음 phase를 투영하는 단계다.
  비어 있지 않은 ChooseOne은 확정한 후보·context를 보존하고 선택/수령/나가기를 끝내기 전 우회할 수 없다.
  후보 0개는 선택 없이 종료한다. Terminal 결과도 pending choice가 남으면 Continue를 유지한다.
- `game_server`의 approved local-install `ActiveRunStore`는 strict V7 envelope, exact required mod/content
  configuration, checksum, opaque generation UUID와 loads/peak monotonic metadata를 소유한다. Tombstone도 generation을 유지한다. Actor가 획득한 expected
  generation과의 CAS가 stale claim과 terminal/new-run ABA를 거절한다. Profile→active 순서의 lock,
  process-lifetime exclusive lock, temp/rename/directory fsync를 사용한다.
- 새 intent가 필요한 approved durable application 변경은 `application_commit.pending.json` journal V4에 exact old/new
  active record와 required configuration, successor profile 및 active document/tombstone을 함께 기록한다.
  `old_active.revision`은 expected revision과 같아야 하며 새 required configuration이 있으면 old configuration도
  결속한다. `corrupt_active_originals`는 nullable이지만 생략할 수 없는 필드다. 일반 변경은 null이고, 명시적인
  supported corrupt-active reset은 원본 파일별 길이·SHA-256 식별값을 이 필드에 기록해 수락 전·설치 시 검증한다.
  Intent rename 전 실패는 준비 root를 폐기한다. Rename 이후 fsync/publication/cleanup 실패는 이미 수락된
  결과를 `recovery-pending`으로 보존한다. Retry/restart는 같은 accepted intent를 멱등 재적용하며 다른
  generation을 덮어쓰지 않는다. Pending 동안 일반 mutation을 차단한다.
- `ApprovedPersistence::open_roots`는 명시적 active/profile root를 각각 인수하고 같은 root도 허용한다.
  Readonly preflight는 두 root의 current/backup/legacy/journal 원본을 directory·lock 생성, profile load와
  backup repair보다 먼저 검사한다. Valid-incompatible 또는 unsupported 원본은 보호하고 missing required mod를
  빈 저장으로 바꾸지 않는다. Corrupt current와 exact-compatible backup의 정상 복구는 유지한다.
- Accepted V4 A→B의 B-only restart는 현재 approved B, bound exact old/new record 및 모든 원본 copy의 일치를
  함께 확인한 경우만 허용한다. 같은 revision의 다른 payload와 제3 generation은 거절한다. Valid authoritative
  intent의 complete profile은 위 readonly active/config 검사 뒤에만 corrupt/missing profile 복구를 허용한다.
  Intent가 없거나 unsupported profile 원본이면 복구 authority를 추정하지 않는다.
- 이전 active envelope V5/V6와 application journal V1–V3는 현재 미지원이다. Current/backup/pending의
  unsupported 원본을 readonly preflight에서 보호하며 자동 replay/변환이나 guessed required mods를 만들지 않는다.
- 완료된 publication의 current와 backup은 같은 최신 확정 generation을 담는다. Valid backup의 복구는
  비차단 상태로 알리며 loads/peak는 감소시키지 않는다. 모든 current/backup/legacy/journal의 unsupported
  version은 자동 복구·reset·새 run publication에 앞서 보호하며 빈 상태로 대체하거나 삭제하지 않는다.
  일반 손상과 버전 비호환을 구분하며, 손상 reset도 보호 대상 원본을 덮어쓰지 않는다.
- Pending choice 없는 `RunComplete`, 확정된 `RunFailed`, `AbandonRun`은 terminal disposition으로
  tombstone을 같은 application intent에 포함한다. 새 run은 확정 terminal 이후
  `replace_active_run: false`로 시작하며 진행 중 교체와 `true`는 거절한다. 평범한 live tick은 불변 continuation cache가 같으면 durable IO를 수행하지 않는다.
- result stats aggregate는 diagnostic entry retention과 독립해 runtime instance event 의미를 직원 UUID별
  한 행으로 집계한다. 마지막 배치 instance가 terminal HP/생존을 대표하고, 전투불능 여부와 누적 metric은
  같은 직원의 모든 instance에서 합성한다.
- MVP는 집계된 직원 행마다 authored metric 순서로 signed `i64` 가중곱과 총합을 명시적으로
  saturate한 하나의 score evaluation을 만든다. 후보 비교와 공개 DTO의 score/title/highlight는 그 동일한
  evaluation을 사용하며, 정책의 signed weight와 tie 순서는 바꾸지 않는다.
- Unity는 MVP, 누적 피해량, 처치 수, 받은 피해량, 기본 공격/스킬 사용 횟수, 배치/철수 횟수, 배치 시간, 최종 HP/생존/전투불능 여부를 raw event log에서 재계산하지 않는다.
- Default-release `BattleResult`, `CombatBattleState`, result projection과 checkpoint/continuation에는 event log
  field나 attachment API가 없다. Event log 상세와 combat trace는 test/debug/test-fixtures capability에서만
  사용할 수 있고 player gameplay history로 누적하지 않는다. Checkpoint codec/version 변경은 별도 계약이
  소유하며 이 lifetime cutover가 암묵적 history field를 추가하지 않는다.
- 폐기된 suppression research, bonus objective, research progress와 pending delivery는 combat result 및
  snapshot 계약에 존재하지 않는다. Encounter source의 first/repeat 보상은 결과 완료 transaction에서
  inventory와 source-completion ledger에 함께 적용되며 별도 수령 transition을 기다리지 않는다.

### Weapon-owned battle eligibility

- `WeaponCombatProfile`이 employee battle source의 deployment affinity, base block, Ground
  DefenseFocus와 Platform Rest authoring을 유일하게 소유한다. Employee base profile의 affinity나
  archetype/default 값은 battle fallback authority가 아니다.
- Safe-zone roster/snapshot/fragment compatibility는 비무장 직원을 보존할 수 있다. 이 경로의 profile은
  battle draft source가 아니며 effective weapon profile/affinity는 `null`이다.
- Battle draft 생성은 실제 장착 weapon을 요구한다. condition disable reason을 먼저 계산하고 그 다음
  `WeaponRequired`를 계산하므로 root action capability와 command error의 안정적인 wire code는
  `weapon_required`다.
- Deploy/redeploy, pending deployment range preview와 certified combat membership은 같은 armed
  eligibility를 사용한다. 거부는 tactical resource, deployment mapping, battle revision과 employee state를
  변경하지 않는다.
- Fragment minimum과 certified power의 block 입력은 authored base `weapon_profile.block_capacity`다.
  Ground/Platform effective effect를 미리 합성하지 않는다.
- Active-run V12 continuation의 current/SavePoint payload는 required embedded equipment metadata와
  encounter progression graph를 포함한다. Checksum과 retained `content_identity`의 contract version/
  semantic fingerprint를 검증하며 구형/unknown version, missing/unknown field와 불일치를 거절한다.
  R02의 V6 및 후속 V7/V8/V9 cutover 기록은 당시 증거이며 현재 reader/writer는 위 V12 경계를 따른다.

## Run State And Atomic Commit

이 장은 node resolution과 run state 사이에서 encounter progression, 영구 도감, inventory, shop,
reward와 직원 성장을 검증하고 한 번만 commit하는 runtime 경계를 소유한다. 각 기능의 플레이어-visible 의미는
[game_rulebook.md](game_rulebook.md)가 소유한다.

### Application Authority And Closed Lifecycle

- Core의 모든 변경 진입점은 하나의 application executor를 통과한다. `ActionKind` 전체와 behavior 밖의
  battle advance/server tick/disconnect, admin, active-run continue/rollback, profile 변경은 exhaustive
  `ApplicationRoute -> ApplicationOwner` registry에 각각 한 번 등록된다. Behavior dispatch도 이 owner를
  소비하며 별도의 root variant router가 같은 authority를 재분류하지 않는다.
- GameCore executor 순서는 `validate current lifecycle -> clone complete authority draft -> run domain validation
  and mutation -> prepare domain result -> validate staged lifecycle -> typed commit token -> one infallible
  root replacement -> return prepared domain result`다. `GameCoreState`, active battle, event/revision,
  checkpoint/durable intent, `run_seed`, retained `ValidatedGameData` generation은 draft에 함께 속한다.
- `CoreApplication::execute_with_generation`은 typed `StartNewGame`에만 candidate를 받는다. 기존 lifecycle/
  blocking VN/NotStarted gate를 먼저 통과한 draft에만 candidate를 설치하고, 그 세대의 RunStart briefing과
  starter 준비를 수행한다. 도메인 처리와 domain result 준비가 성공한 뒤 state/seed/generation/output을 함께 commit한다.
  도메인 준비 실패·허용되지 않은 Start·다른 command에는 기존 세대/상태/profile/난수와 출력 authority를 보존한다.
  직접 경로의 commit 후 공개 결과 변환은 아래 별도 경계를 따른다.
- Host가 B를 게시해도 A의 active run/briefing/battle은 A의 strong generation을 유지한다. A/B content를
  런 중에 섞거나 failed B를 A 성공으로 대체하지 않는다. 성공한 eligible 새 런은 briefing부터 B에 결속된다.
- Checkpoint capture는 retained identity를 포함한다. Continue, explicit load와 cached export는 identity를
  검증한 뒤 기존 seed/graph/power 검증을 유지한다. Continue는 saved seed/UUID를 복원하고 load count를
  소비하지 않으며 explicit load는 현재 run seed와도 일치해야 하고 최대 세 번이다. Profile/peak와 보존 대상
  VN session counter의 단조성은 유지한다. Test transaction rollback도 state/seed와 strong generation을 함께
  복원하고 새로운 profile/peak는 병합한다.
- GameCore draft 안에서 validator, gameplay handler, cursor/domain result 준비가 `Err`를 반환하면 draft는
  폐기되고 committed gameplay authority는 유지된다. 같은 입력의 retry는 같은 gameplay authority basis에서
  시작하지만, 이미 게시된 전투의 fatal 오류는 아래 application fault guard로 후속 실행을 차단한다.
- `CoreApplication::prepare_command`/`prepare_command_with_generation`과 `prepare_active_battle_for_host_tick`은
  GameCore token을 commit하기 전에 공개 `ApplicationResult`와 checkpoint/profile 자료를 준비한다.
  준비 오류 또는 `PreparedApplication` 폐기는 gameplay draft를 게시하지 않는다. Host는 새 durable intent가 필요한
  경우 그 수락 뒤 준비 객체를 소비한다. 저장 내용이 같아 IO를 생략하는 경로도 기존 저장·revision 검증 후
  준비 객체를 소비한다. `commit`은 배타적으로 빌린 준비 객체를 한 번 소비하며 gameplay·공개 결과 변환을 재실행하지 않는다.
- 직접 `CoreApplication::execute`/`execute_with_generation`, battle advance/tick/event-bucket 및 Continue 경로는
  GameCore commit **뒤에** 공개 결과를 변환한다. Ordinary projection 실패는 `AcceptedRecoveryPending` 결과로
  표현한다. Private execution control의 취소·한도 interruption은 이 변환에서도 오류로 전파될 수 있으므로,
  직접 호출의 `Err`가 항상 해당 application의 commit 부재를 뜻하지는 않는다. 이 interruption을 battle fatal로
  기록하지 않는다. 상세한 projection/control 의미는 아래 authoring 절을 따른다.
- 실제 제작 소비자인 `TrialState::prepare_step`은 별도 application fork에서 직접 실행·필수 receipt를 준비한다.
  `advance`는 control scope가 성공한 뒤에만 candidate aggregate를 채택한다. 내부 application이 이미 commit했어도
  뒤의 projection·receipt·control 실패는 바깥 candidate를 폐기하여 마지막 accepted session을 유지한다.
- `CoreApplication`의 시도용 transaction revision과 최초 battle fault capsule은 gameplay draft 밖에 있다.
  실패·준비 폐기로 gameplay가 유지돼도 시도 revision은 증가할 수 있고, fatal capsule은 남는다.
  따라서 위 보존 조건은 전체 application 객체의 불변을 뜻하지 않는다. Fault 수명·허용 조회는
  [Faulted Published Battle](#faulted-published-battle), rollback token은 단회 owner/revision 검증을 따른다.
- Private `LifecycleContext`가 closed `GameState` phase를 소유하고 `GameState`는 snapshot으로만 노출된다.
  Run/node/content/battle optional context는 draft 조립 재료일 뿐 독립 phase writer가 아니다. Commit gate는
  NotStarted/pre-run/map/node/shop/reward/battle/result phase가 요구하는 context와 node/battle/session identity를
  검사하고 incoherent 조합을 authoritative state로 publish하지 않는다.
- Domain-local staging helper는 complete application draft 안에서 algorithm preparation에만 사용할 수 있다.
  Public/root entry가 이를 이용해 live root를 별도 commit하거나 application commit port를 우회할 수 없다.
  Battle kernel privacy, history lifetime, checkpoint version과 transport-neutral public projection은 각각 후속
  소유 boundary이며 이 application contract가 그 의미를 선행 변경하지 않는다.

### Transport-Neutral Core Public Projection

- Default release의 public root는 `application::CoreApplication`이다. 이 service는 `GameCore`를 private으로
  소유하며 inner borrow, dereference, raw-state conversion을 제공하지 않는다. Host 저장용 capability는
  위 borrowed preparation/단일 commit 계약을 따른다.
  `game::world`, `PlayerBehavior`, `BehaviorResult`와 legacy command-result contract family는 crate-private이다.
- `ApplicationCommandKind::ALL`은 `ActionKind::ALL`의 Core player action과 exact bijection이다. `ApplicationCommand`는 immutable
  `ApplicationDocument`를 private domain command로 decode하고 kind/payload 일치를 authority access 전에 검증한다.
  Unknown/malformed/mismatched input은 typed `ApplicationError`며 live authority를 읽거나 변경하지 않는다.
- `ApplicationResultKind::PLAYER_RESULTS`와 별도 `AdminOperation`은 player result meaning과 admin operation의 identity를 표현한다.
  현재 domain result의 exhaustive match가 kind/meaning/payload를 생성하며 battle command도 공유 update
  content와 별개로 command 의미를 유지한다. `Accepted`, `ReadOnly`,
  `AcceptedRecoveryPending` disposition이 commit/result delivery 상태를 구분한다. Ordinary commit 후 projection
  문제는 recovery-pending으로 표현하며, private control interruption과 Trial의 외부 acceptance는 위 절을 따른다.
- `ApplicationErrorKind`는 input/projection error와 domain error meaning을 stable code/detail에
  exhaustive mapping한다. Domain error enum serialization은 public contract가 아니며 wildcard/unknown semantic
  fallback은 없다.
- `ApplicationSnapshotKind`는 run, inventory, employee roster, selected event, action capabilities, player
  profile, encounter compendium의 immutable replace-whole read model을 소유한다. 없는 selected event는
  `null`이고, collection order, battle revision/cursor, P05 bounded-history/resync 의미는 private producer가 만든
  projection과 동일하다.
- Battle advance/tick/event-bucket/disconnect, typed Core admin, profile merge/reset, checkpoint export/continue도
  `CoreApplication`의 scoped port다. `ApplicationCheckpoint`는 checksum-protected continuation bytes/load count/peak
  power를 그대로 운반하며 별도 host envelope나 wire version을 정의하지 않는다. 현재 payload 버전과 지원 범위는
  [Durable active-run host boundary](#durable-active-run-host-boundary)가 소유한다.
- 이 hard cutover의 compatibility boundary는 Core 내부에만 있다. External transport schema, server/Unity
  mapper, fixture, consumer migration, rollout과 compatibility PASS는 검사하거나 주장하지 않는다.

### Final Core Performance Guard

- Final performance authority는 P01의 immutable release baseline
  `evidence/P01_PERFORMANCE_BASELINE.json`과 P08의 Core-owned standalone verifier다. Event-heavy workload는
  동일 seed/input에서 5,000 ms를 진행하고, concurrent workload는 64 session을 8 lane에서 각각
  `100 × 10 ms` 진행한다. Event 수, compact JSON byte 수와 SHA-256가 baseline과 하나라도 다르면 latency와
  무관하게 실패한다.
- 일반 release build의 21개 post-warm-up sample만 latency/non-regression 판정에 사용하고,
  `allocation-observer` build의 별도 5개 sample만 allocation/peak/retained 판정에 사용한다. Linux reference
  run은 coordinator CPU 30과 lane CPU `0,2,4,6,8,10,12,14` affinity를 receipt에 기록한다. Instrumented
  allocator 시간은 latency PASS 근거가 아니다.
- 이전 P08의 frozen 측정은 event-heavy p95 `10.818511 ms < 25 ms`, 64-session p95
  `8.343780 ms < 10 ms`다. 두 exact one-sided comparison의 supported-regression 판정은 false이며 baseline
  output identity `186 / 65,720 / 2ada19d4…333a`와
  `11,840 / 3,881,088 / 68da2cf9…ca9`를 모든 sample이 유지한다.
- Direct heap guard에서 event-heavy sample은 allocation/reallocation `14,295 / 25,896`, peak/retained delta
  `260,550 / 260,550 bytes`이고, concurrent sample은 `170,740 / 140,036`, 최대 peak delta
  `8,001,938 bytes`, retained delta `4,196,928 bytes`다. 각 값은 동일 P01 workload의 operation/peak/retained
  상한 이하이며 end-retained 감소가 peak 증가를 숨길 수 없다.
- P08 당시 `CoreApplication` live-run guard는 Run snapshot+serialization p95 `226.508 µs`, `48,493 bytes`,
  V5 checkpoint export p95 `8.344 µs`, `17,956 bytes`였다. 이는 이전 대상의 역사적 측정이며 R02 V6의
  현재 측정으로 재사용하지 않는다. `<500 µs / <64 KiB`, `<100 µs / <32 KiB` Core engineering cap은 유지한다.
  현재 continuation cache는 확정 상태 변경 시 직렬화한 exact V12 bytes를 불변 Arc로 보존한다.
  Export는 이 Arc를 공유하며 미완료 live tick마다 다시 직렬화하지 않는다. 현재 성능은 U04 검증
  receipt로 별도 판단하며 위 역사적 V5/V6 수치를 V10 측정으로 재사용하지 않는다.
- U04 compact V10 수정본의 [plain guard 원자료](goals/core_result_persistence/evidence/public_guards003_raw.json)는
  snapshot p95 `207.013 µs / 48,591 bytes`, cached checkpoint export `0.031 µs / 18,538 bytes`로
  위 기존 한도를 통과했다. [별도 allocator 원자료](goals/core_result_persistence/evidence/public_guards_allocator002_raw.json)는
  128/8,192 retained history 각각5회에서 동일한 `627 allocations / 0 reallocations / 74,834 peak bytes / 0 retained bytes`를
  관찰했다. 이는 각 [실행 receipt](goals/core_result_persistence/evidence/public_guards003.json)의 source·명령에만 귀속되며,
  allocator 계측 실행의 시간값은 latency 판정으로 사용하지 않는다. 저장 변경 때의 encoding 비용과 cached export 비용을 구분한다.
- Bounded-history last-10 query는 1,024/8,192 retained entry에서 p95 `0.202/0.212 µs`이고 exact cap/cursor
  의미를 유지한다. Application rejection staging은 128/8,192 entry에서 p95 `23.399/22.284 µs`이며 양쪽
  allocator sample이 정확히 `675 allocations`, `0 reallocations`, `84,409 peak bytes`, `0 retained bytes`다.
  `BattleEventLog` clone은 entry vector를 `Arc`로 공유하고 첫 mutation에서 분리하므로 rejected transaction이
  retained diagnostic history를 복사하지 않는다. 원본 JSON/sequence authority와 clone mutation isolation은
  unit test로 고정한다.
- Public guard가 cap-scale history를 설치하는 method는 `test-fixtures` feature에만 존재한다. Default release
  compile-fail probe와 rustdoc은 이 method가 production API에 없음을 보장한다. Production code는 test 이름,
  benchmark identity, allocator observer 또는 guard threshold에 따라 분기하지 않는다.
- R02의 같은 public guard 재측정은 snapshot p95 `241.039 µs / 48,493 bytes`, V6 export
  `8.770 µs / 18,154 bytes`로 기존 cap을 통과했다. 별도 allocator 5개 sample씩은 128/8,192 history 양쪽에서
  `677 allocations / 0 reallocations / 84,553 peak bytes / 0 retained bytes`를 관찰했다. 이전 측정보다
  2 allocations/144 peak bytes 증가했으며 history 크기 독립성·정확한 rejection rollback은 유지한다.
  [현재 plain receipt](goals/content_authoring_pack_generation/execution/single_goal/evidence/public_guards_002_receipt.json)와
  [별도 allocator receipt](goals/content_authoring_pack_generation/execution/single_goal/evidence/public_guards_allocation_001_receipt.json)는
  실제 검증 입력에만 귀속된다. 기존 observer의 합격 기준·알고리즘은 같고 synthetic BattleEnd fixture에
  R01의 required `skill_effect_source: None`만 추가했다. 이 재측정은 event-heavy/64-session 과거 수치의 재실행을 뜻하지 않는다.
- 이 수치는 Core 내부의 permanent regression/safety guard이며 product SLA, 배포 성능 약속 또는 external
  server/Unity consumer 측정이 아니다. Source/tool/input/config가 바뀌면 해당 receipt를 다시 실행해야 한다.

### Private Finite Battle Kernel

- Default release에서 `game::battle::core`는 crate-private이다. `BattleCore`, runtime unit, kernel field,
  simulation helper와 raw event queue injection은 외부 caller가 이름 붙이거나 변경할 수 없다. 기존
  test/debug/`test-fixtures` capability만 deterministic kernel oracle을 위해 module을 노출하며 production
  authority가 아니다.
- Production battle setup과 live mutation은 validated data 및 `GameCore` application draft가 소유한
  `ActiveBattleSession`을 통해서만 kernel command/query/event/read-model port에 도달한다. Kernel state를
  독립 publish하는 두 번째 production writer는 없다.
- Continuous world position은 raw coordinate field를 노출하지 않는 `WorldVec2`로 닫힌다. Raw float 생성과
  deserialization은 component finiteness를 검사하고, unit radius/speed와 float battle-time conversion도
  checked finite scalar를 먼저 만든다. NaN과 positive/negative infinity는 typed error이며 clamp, default,
  normalization 또는 accepted total order로 바뀌지 않는다.
- `BattleEvent`의 float-bearing impact/area payload는 finite `WorldVec2`만 가질 수 있다. 따라서 private
  `BinaryHeap`의 manual `Eq`/`Ord` 전제는 public raw event 입력에 의존하지 않는다. Signed zero는 유효한
  finite 값이며 현재 equality/order 의미를 유지한다.
- Privacy/finite cutover는 boss task, movement, target acquisition, skill delivery, same-bucket priority와 event
  identity algorithm을 바꾸지 않는다. Active history cap/result lifetime, checkpoint format, transport-neutral
  public projection과 performance PASS는 각각 P05/P06/P07/P08 소유다.

### Root Capability And Parameterized Admission

- `ActionKind`의 단일 typed descriptor가 `CoreCommon`/`CoreDomain`/`CoreBattle`/`CoreContextual`/
  `HostRecovery` route, `ActionScope`와 battle UUID/revision authority 필요 여부를 함께 소유한다. 이
  descriptor는 authorization이나 payload validator가 아니다. Common/battle evaluator, concrete domain
  validator와 server host-recovery lifecycle이 각자 현재 state와 payload의 실행 가능성을 계속 소유한다.
  Root capability의 기존 deterministic order는 별도 ordered view로 유지하되 그 membership은 descriptor와
  양방향 검증한다.
- Root `action_capabilities`는 현재 context에서 관련 없는 action을 생략하고, 관련 있지만 실행할 수 없는
  action을 stable typed reason으로 disabled한다. Parameterized action은 대응 payload가 하나라도 실행
  가능할 때만 family capability가 enabled되며, 이것이 모든 payload의 실행 가능성을 뜻하지 않는다.
- Maintenance projection은 `maintenance_options`의 Core-owned operation eligibility를 재사용한다.
  Skill-fragment의 upgrade/awaken/dismantle은 각각 `enhance`/`awaken`/`dismantle`, equipment의
  dismantle/enhance는 각각 `dismantle`/`enhance`에 대응한다. Maintenance context에서 대응
  `can_execute`가 하나도 없으면 `NoEligiblePayload`, context 밖이면 irrelevant다.
- Snapshot projection과 common preflight는 같은 action-operation mapping을 사용한다. Family capability가
  enabled여도 command는 선택한 identity와 최신 domain state를 기존 Maintenance validator로 다시
  검증한 뒤에만 mutation하며, 실패는 원자적으로 state를 보존한다.

### Facility Map Authority And Persistence

- Live `FacilityTemplateDatabase`의 template/version/semantic slots/directed connections가 topology의
  단일 authority다. `RunMap`은 선택한 `map_template_id`, `facility_template_version`과 slot별 확정 node,
  derived start/typed terminal identity를 저장하지만 runtime node-ID edge list나 depth/lane 좌표를
  저장하지 않는다.
- Navigation, node completion, manifestation과 recovery-path 평가는 current live template를 ID로
  resolve하고 node `slot_id`를 authored connection에 연결한다. 좌표, node 배열 순서, terminal까지의
  거리 또는 구 procedural graph를 fallback으로 사용하지 않는다.
- Facility generation은 mode/Floor schedule이 정한 profile과 pool에서 deterministic shuffle/cycle을
  소비하고 constrained assignment를 한 번 수행한다. 남은 template order, cycle, 사용 template와
  assignment된 current map은 `FacilityRunState`/`RunState`/checkpoint/active-run payload에 포함되며
  reconnect/load에서 weight로 재생성하지 않는다.
- Checkpoint import는 current catalog에서 template ID/version을 resolve하고 모든 authored slot이 정확히
  한 번 존재하는지, category allowed set, derived start IDs와 typed terminals가 exact 일치하는지 strict
  검증한다. 누락/추가/중복 slot, version drift와 terminal 역할 drift를 silent reset하지 않는다.
- Manifestation target은 Floor 생성 시 node identity/content/seed까지 precommit한다. 마지막 prerequisite
  성공 commit과 target reveal/navigation 갱신은 같은 world transaction이며 monotonic/idempotent다.
  Retreat/failure/cancel rollback과 reconnect는 concealed assignment를 보존한다.
- Generic capstone switch는 typed template/entry input을 모두 검증하고 완전한 map과 progression을 staging한
  뒤 원자 교체한다. BossOmen은 이 seam으로 `boss_omen_capstone`을 사용하며 기존 exploration map에
  node/connection을 append하지 않는다.
- Certified power는 현재 전투 진입 가능한 roster(`alive && physical_damage <= 4 && trauma < 100`)만
  합산한다. 경계 안의 현재 HP 및 physical damage/trauma 크기는 계수가 아니며 Critical, Breakdown,
  Dead는 제외한다. eligible power가 0인 roster의 relative threat는 `VeryHigh`다.
- Facility creation은 authored floor minimum에서 Floor anchor를 먼저 확정한 뒤 assignment profile의 required
  difficulty profile로 ordinary quota/role, fixed target power와 reward/wave/stat contract를 category/content와
  같은 transaction에서 저장한다. Manifested Combat/Boss에 difficulty authority가 없거나 live model/profile과
  다르면 preview와 실행은 실패하며 Floor 번호나 current roster로 fallback하지 않는다.
- `MapViewDto`는 `difficulty_band`, `relative_threat`, `readiness_warnings`만 투영한다. Internal node의
  model/profile/anchor/target은 wire에 포함하지 않는다. U14 `entry_preview.combat_deterrence`는 적용할 HP/ATK 배율과 수량배율을 공개한다. Relative threat/readiness는
  현재 roster에서 다시 계산할 수 있지만 저장된 node difficulty는 수정하지 않는다.

### Encounter Source Reward Commit Boundary

Encounter source의 첫 보상과 repeat 보상은 combat result 완료 transaction이 직접 소유한다.

- Live source row는 stable `reward_source_id`, exact `encounter_id`/`fragment_id`,
  `first_equipment_ids`, `repeat_rewards`, optional `revival`을 가진다. 원장은 완료 source의 primary
  abnormality identity를 비교해 같은 종류의 첫 격파를 원정당 한 번만 허용한다. 보유 파편 수나 분쇄는
  이 원장을 초기화하지 않으며 새 원정에서만 초기화한다.
- 첫 branch는 작성 파편과 최초 장비, 반복 branch는 작성 반복 효과만 지급한다. 일반보스 첫 승리는
  원정 전용2단계 알도 만들며 엘리트/최종보스에는 알이 없다. 마탄은 일반 보상·파편·알·생존XP를
  지급하지 않는다. node/attempt/cycle과 같은 staged world에 적용하고 실패하면 전부 rollback한다.
- Defeat, retryable defeat와 `RetreatBattle`은 source reward를 commit하지 않는다. Reconnect/snapshot은
  committed state를 투영할 뿐 새 지급 authority가 아니다.
- 폐기된 research progress, `pending_research_deliveries`, safe-zone 수령 hook과 관련 DTO/result field는
  compatibility alias나 ignored field로 남기지 않는다.
- SavePoint completion은 staged recovery, node completion과 process-local checkpoint capture를 하나의
  world transaction에서 검증·커밋한다. Gate confirm도 staged recovery와 Floor transition을 같은
  transaction으로 처리한다. 실패하면 employee condition, inventory, map/Floor, active node와 checkpoint
  어느 것도 부분 적용하지 않는다.
- 두 recovery는 같은 tagged `TraumaRecovery`를 사용한다. `CurrentPercent`는 현재 trauma 기준
  `ceil(trauma * percent / 100)`을 saturating 차감하며 Critical/Breakdown을 포함한 생존 직원만
  대상으로 한다. Dead와 physical damage는 바꾸지 않는다.

### Manual Elite Repeats And Retired Commands

엘리트 자연 반복은 수동 전투다. 현재 원정의 격파/숙달 횟수로 후보를 닫거나 자동 강화하지 않고,
같은 지도 중복과 기존 배정/예약을 보존한다. 첫 파편·장비 이후에는 `repeat_rewards`만 지급한다.
`SweepMasteredElite`, `CompleteMasteredEliteSweep`, `ConvertMapNodeWithMonolith`는 옛 클라이언트
명령을 decode한 뒤 `ActionUnavailable`/`RetiredFeature`로 거절한다. 활성 capability, query와 상점은
소탕·모노리스 구매/교환을 제안하지 않는다. 옛 state를 새 부화 진행으로 자동 해석하지 않는다.

### Permanent Player Profile

영구 조우 지식과 기록, exact expedition lifecycle은 run-local progression과 분리된 하나의
local-install `PlayerProfileState`다.

- Profile은 `encounter_compendium.registered_ids`, 최고 깊이/무사망 기록/최종보스 기록과
  `lifecycle { expedition_sequence, active_expedition, vault, first_return_story }`를 소유한다.
  `standard_tutorial_completed`는 제거됐다. 이름·위험·authored 순서는 현재 catalog에서 투영한다.
- shared successful opponent spawn path의 attempt-local ledger는 Abnormality/CorrodedEmployee identity를
  기록한다. Expedition final Victory만 후보 지식을 permanent profile로 합친다. Tutorial 전후는 profile 전체가
  불변이고 Defeat/Retreat/preview/preload/test/summoned unit은 permanent registration writer가 아니다.
- `merge_knowledge`만 set union/boolean OR/numeric max를 사용한다. 전체 profile `merge`는 lifecycle이 다르면
  거절한다. vault/sequence/marker/story는 union이나 max 대상이 아니며 authoritative installation 또는
  CAS를 통과한 accepted paired publication이 정확 다음 값을 채택한다. persisted dormant ID/사본은 보존한다.
- SavePoint와 rollback snapshot은 영구 profile authority가 아니다. 같은 instance 복원은 최신 knowledge/story를
  보존하고 다른 instance/종료된 binding의 Continue를 거절한다. run-local record_progress만 checkpoint로 복원한다.
- root profile projection은 knowledge/record와 lifecycle을 함께 전달한다. Compendium은 current catalog의
  canonical ID/registered를 투영하고 미등록 metadata를 생략한다. Dormant ID는 저장에 남는다.
- Host profile envelope의 현재 버전·reader 지원 범위는 [Durable active-run host boundary](#durable-active-run-host-boundary)를 따른다.
  Checksum/current/backup/temp/exclusive lock을 사용한다. Supported profile의 최대
  encoded16MiB, active/journal은 각각96MiB이며 encode 이후 intent 수락 전에 검사한다. 이는 형식 admission
  상한이고 gameplay vault 사본 수 제한이 아니다. 이전 envelope/unknown fields는 원본 보호 대상이다.
- 정상 host는 profile→active lock/CAS와 위 현재 journal 형식으로 exact successor profile와 active/tombstone을 함께
  수락한다. Preaccept 실패는 준비된 Core와 successor profile/active 확정값을 게시하지 않는다.
  이 보장은 준비 중 temp 파일 쓰기나 기존 backup 복구까지 금지한다는 뜻이 아니다.
  Accepted 이후 IO 실패는 정확 다음 Core 상태와 recovery_pending이다.
  Retry/restart는 같은 intent를 끝내며 pending reset이나 unsupported original 덮어쓰기는 거절한다.
- 이미 정상 승인/open된 actor의 이후 supported active current/backup 손상은 title의 명시 reset 경로에서
  원본을 acceptance 전까지 보존하고 paired tombstone/active marker 종료로 처리한다. startup admission을
  우회해 손상/구성 불명확 원본을 여는 기능은 아니다. 개별 도감/field reset 명령은 없다.
- developer combat_traces는 compendium을 읽거나 수정하지 않는다.

### Ordinary Inventory And Quantity Boundary

- `Inventory`의 ordinary slot occupancy는 owned equipment instance 수와 서로 다른 consumable
  definition stack 수의 합이다. Equipment material, skill fragment, fragment dust와 currency는
  포함하지 않는다.
- Equipment는 owned instance UUID로 유지한다. Consumable은 `definition_id` key의 단일
  `quantity: u32` stack이며 metadata UUID가 wire selector다. 별도 owned consumable UUID stream은
  없다.
- ordinary storage에는 fixed equipment/consumable cap과 capacity admission preflight가 없다.
  `InventoryFull`은 ordinary inventory grant 결과가 아니다.
- 저장된 `reserved_slot_count`는 50에서 시작해 성공한 새-slot commit이 경계를 넘을 때 최소
  10단위로 증가한다. 수량-only mutation과 제거는 값을 변경하지 않으며 collection memory
  capacity나 overflow storage로 사용하지 않는다.
- root inventory snapshot은 `reserved_slot_count`, `occupied_slot_count`, equipment instance
  배열, definition-id 정렬 consumable stack 배열을 투영한다. 각 consumable은 full post-state
  `quantity`를 필수로 가진다.
- 첫 consumable grant는 `inventory_diff.added`, 기존 stack grant와 수량이 남은 사용/판매는
  `updated`, 수량 0은 metadata UUID를 `removed`에 보낸다. Equipment `removed` UUID는 owned
  instance identity이므로 item kind가 identity 의미를 결정한다.
- `GrantConsumable`은 explicit positive `quantity`를 authoring/runtime contract로 가진다.
  Reward, Event, Shop, admin, combat result는 같은 `GrantExecutor`와 cloned `Inventory` stage를
  사용하므로 quantity와 reserve는 전체 transaction이 성공할 때만 함께 commit된다.
- `RunCheckpointPayload`와 `BattleEntrySnapshot`은 `Inventory` 자체를 clone하므로 quantity와
  reserve high-water를 정확히 저장/복원한다.

### Shop Session, Quote, And Transaction Boundary

- `ShopDatabase`는 reusable `merchandise_presets`, Shop별 preset composition/local entries,
  stable `offer_count`, default buy percentage, optional refresh/sell policy, encounter-level Shop
  pools를 가진다. Prototype `ShopType`, fixed visible/hidden stock, `can_reroll` schema는 없다.
- Initial page와 refresh page는 authoritative run/Floor/node/Shop/page state에서 weighted
  sampling without replacement로 생성된다. 후보는 typed merchandise identity로 정렬된 뒤
  sampling되며 page 안에 duplicate가 없다.
- `ShopSessionState`는 current offers, deterministic offer identity, page index, purchased identity
  set, refresh uses와 policy를 소유한다. Root snapshot/reconnect는 저장된 offer를 읽을 뿐 다시
  추첨하지 않는다. Floor-local `RunState.shop_sessions`와 `RunCheckpointPayload.shop_sessions`가
  page/purchase/refresh continuation을 정확히 저장·복원한다.
- Purchase payload는 definition UUID가 아니라 current `offer_id`를 보낸다. Core는 offer의 exact
  total quote와 funds를 확인하고 공통 `GrantExecutor` stage, currency deduction, offer removal,
  purchased history를 하나의 commit으로 교체한다.
- `RefreshShop`은 `RerollShop`을 대체한다. Optional policy, remaining uses, funds, full-page candidate
  sufficiency를 같은 evaluator로 확인한 뒤 cost/use/page를 한 번에 commit한다. 실패 시 셋 모두
  불변이다.
- Sale payload는 `{ item_uuid, quantity }`다. Equipment identity는 owned instance UUID이고 quantity
  1만 가능하다. Consumable identity는 static metadata UUID이고 requested quantity만 해당 consumable definition stack에서
  제거한다. Intrinsic removability, tag acceptance, checked quote와 currency overflow를 commit 전에
  검증한다.
- Buy quote는 checked `ceil(base_price * buy_percent / 100)`, sale unit quote는 checked
  `floor(base_price * sell_percent / 100)`다. Startup cross-catalog validation은 모든 reachable
  merchandise에 `max_live_sell_quote < min_live_buy_quote`를 강제한다.
- `selected_event.type == "shop"`은 full display offer, purchase eligibility, refresh status/reason,
  sale candidates/quotes를 투영한다. Context capability는 실행 가능한 payload 존재 여부를 나타내고
  commit은 payload projection과 같은 evaluator를 다시 실행한다.

### Boss Egg And Hatch Boundary (U17)

- `EncounterProgressionState`의 최초 원장과 `RevivalRunState`는 별도 책임이다. 후자는 source별 다음
  알(`Stage2|Stage3`)과 현재 층의 부화 node별 source/단계/해결 여부/출처 지불 영수증을 소유한다.
  알은 거래 가능한 material이 아니며 같은 층 방 재방문은 부화를 유지한다. 하강은 부화 node를
  지우되 미사용 알을 유지하고, 소비한 알·촉매를 환불하지 않는다.
- `HatchEncounter { node_id, reward_source_id, stage, payment }`는 Expedition `NodeConfirm`의
  선택 가능한 일반 Combat encounter만 대체한다. 예약/전조/봉인/해결/다른 부화 node는 대상이 아니다.
  Gate 유일 경로도 허용하며 topology와 다른 node 배정을 바꾸거나 우회로를 만들지 않는다.
- `payment`는 `MaterialPayment { material_id, carried, warehouse }` 목록이다. 촉매 recipe의 정확한
  수량과 출처 잔고를 검증하고 알 소비, 소지/창고 debit, encounter/difficulty/session 교체를 원자
  확정한다. 실패는 UUID와 모든 비용·진행을 보존한다. 부화 자체는 전투나 입장비/후퇴를 실행하지 않는다.
- `run.revival` 조회는 알, 부화 기록과 선택 node의 가능한 source/단계·작성 배율·보상·장비 후보·다음 알,
  촉매 필요량과 소지/창고 잔고를 제공한다. 부화 후에도 영수증과 같은 단계 정보가 남는다.
- `RevivalNodeTuning`의 작성 배율은 해당 difficulty profile/role의 target power와 적 HP/공격/방어에
  합성한다. generated wave budget은 단계로 추가 확대하지 않는다. 무억제 HP/공격 강화는 여기에
  곱하고 수량형 승리 보상은2배, 장비 물품과 선택 횟수는1회다. 단계 보상 자체에는 구 모노리스의
  별도 보상 배율을 다시 곱하지 않는다. 공식140/180%는 데이터 조정값이지 엔진 상수가 아니다.
- 2단계 승리는 작성 파츠 등 고정 보상·장비 선택권·3단계 알,3단계 승리는 고정 보상·선택권만 만든다.
  장비 후보는 단계 policy에서 결과 요약과 선택 세션으로 함께 확정하며 Continue reader도 같은
  생성자를 사용한다. 후보0개는 `no_reward_candidates`와 빈 목록을 표시하고 선택 단계를 생략한다.
  `SelectReward`는 후보 선택, `ClaimReward`는1회 지급, `ExitReward`는 지급 후 복귀다.
- 부화 보류와 수동 후퇴는 단계/지불/입장 receipt를 유지한다. 후퇴2회 뒤 마지막 도전이 남고,
  실제 패배는 즉시 node를 소비하며 성공 보상·다음 알·환급을 만들지 않는다. 부활 보스는 황금가지나
  억제력 보충을 주지 않으며 유물을 자동 추가하지 않는다.
- Active Run/Continue 형식은17이다. 입장 전에 확정한 부화는 fresh Continue에 남고, 미완료 node
  내부의 부화는 기존 pre-entry snapshot으로 원래 조우/알/소지·창고 출처와 함께 되돌아간다.
  같은 프로세스 reconnect는 현재 battle을 유지한다. SavePoint는 당시 원장/부화/비용을 복원하며
  창고 전체 대신 기존 U15 원정 debit 대조를 사용한다. 결과/선택 대기/수령 상태도 즉시 저장한다.
  구 개발 저장은 typed version 오류로 거절하고 원본을 보존한다. 자동 migration은 제공하지 않는다.

### Reward And Employee Growth Boundary

Reward definition은 typed `RewardEffect` 목록이고 실제 지급은 `GrantExecutor` 하나가 수행한다.

- Reward session과 combat result는 inventory, skill fragment, roster, UUID manager, Enkephalin을 clone/stage한 뒤 모든 effect가 성공한 경우에만 한 번 교체한다. 중간 effect 실패는 앞선 XP/재화/item mutation을 commit하지 않는다.
- Live combat encounter는 명시적 `ClaimAll` 또는 `ChooseOne`을 허용한다. ChooseOne은 실제 후보와 원 grant context를 확정 결과에 묶고 일반 RewardSession에서 선택·수령한다. 후보 효과를 결과 시점에 먼저 지급하거나 consumer가 재추첨하지 않는다. 비어 있는 후보는 명시 no-reward로 진행한다.
- 시설 유물 활동이 아닌 Map Reward node는 authored pool에서 `Experience` grant kind를 제외하고 node seed로 definition 하나를 고른 뒤 `ClaimAll` session으로 고정한다. `GrantEquipmentFromPool`도 stage/reward/effect identity에서 파생한 deterministic seed를 사용한다.
- `GrantExperience.amount`는 total pool이 아니라 대상 직원 한 명당 지급량이다.
  대상 UUID를 dedupe하고 roster membership을 검증한 뒤 각 대상에게 명시된 전액을 지급하며 quotient와
  remainder 분배는 만들지 않는다. `amount: 20`이고 대상 직원이 셋이면 각자 20, 총 60 XP다.
  `CombatParticipants`와 `SelectedEmployee`는 explicit `GrantExecutionContext`가 없으면 실패한다.
- `AliveRoster`는 physical damage가 6 미만인 모든 직원에게 전액을 지급한다. Combat eligibility, trauma,
  배치, 무장과 roster slot은 별도 조건으로 암묵 추가하지 않는다. `CombatParticipants`는 context의 참가자,
  `SelectedEmployee`는 context의 한 직원, `AllRoster`는 사망자를 포함한 roster 전체에 각각 전액을
  지급한다.
- `EmployeeExperienceDiffDto`는 authored reward effect의 exact per-employee amount와 level/experience before/after다. 성공/일반 Reward result는 이 diff를 transport하고, game_server는 의미를 재해석하지 않는다. 현재 Unity에는 전용 XP diff reader가 없으며 root roster snapshot이 최종 상태 source다.
- 커밋된 전투 결과의 survival XP는 `result_stats.employees`의 employee-level `survived`와 staged roster의 `Alive`를 함께 검사하고 각 직원에게 `post_battle_survival_xp`를 `SelectedEmployee` effect로 한 번 적용한다. 승패 모두 같은 post-battle stage를 사용하고 `RetreatBattle`은 entry snapshot 복원으로 이 mutation을 폐기한다.
- Survival XP는 authored combat reward의 `employee_experience_diffs`에 합쳐지지 않는다. `NodeOutcomeEmployeeChange.experience_before/after`와 후속 roster snapshot이 그 최종 변화를 표현한다.
- `Employee.experience`는 current level 안의 잔여 XP다. `add_experience_with_policy`는 기존 잔여와 지급량을 u64 임시 합계에 정확히 더한 뒤 `xp_required_for_next_level(level)`을 반복 차감한다. 잔여가 검증된 양수 u32 레벨 비용보다 작아진 뒤 기존 u32 필드에 저장하므로, 합계가 u32 최댓값을 넘더라도 레벨업 전에 XP를 잘라내지 않는다. 한 grant가 여러 level을 넘을 수 있고 authored curve를 넘으면 마지막 requirement를 반복한다. 기존 level 최댓값의 saturating 증가 의미와 저장 형식은 유지한다.
- `Employee.level`이 persistent 성장 source이고 `battle_tier_by_level`은 level에서 파생한다. 현재 `BattleUnitDraft.level`/snapshot `battle_tier`는 분류 projection이며 stat pipeline의 HP/attack/defense multiplier가 아니다.
- `GrowthStack`은 level/XP와 별도 내부 draft state다. 현재 `KillStack`만 final stat pipeline에서 flat attack으로 적용되고, production gameplay에는 stack producer가 없다. Level-up은 GrowthStack을 변경하지 않는다.

현재 live 수치는 `game_resources/data/run/policy.ron`이 소유한다. Canonical 문서는 수치의 현재 의미를 설명하지만 runtime constant나 Unity-local 계산으로 복제하지 않는다.

## Visual Novel Runtime

이 장은 top-level VN state, pre-run flow와 trigger 평가·반복·checkpoint 정책의 runtime ownership을
소유한다. Scene의 player-facing 콘텐츠 의미와 정확한 Unity wire shape는 각 인접 canonical을 따른다.

### Presentation And Pre-Run Flow

Visual Novel runtime은 route나 run 내부 상태가 아니라 core 최상위 runtime 상태다.

- Canonical owner는 `GameCoreState.visual_novel`이다.
- `RunState`와 run checkpoint는 active VN session을 소유하거나 복원하지 않는다.
- Root `state_snapshot.visual_novel`과 `battle_update.visual_novel`은 같은 top-level VN projection에서 나온다.
- Unity는 VN 표시 여부를 `selected_event`나 현재 route로 추론하지 않고, public snapshot의 `visual_novel != null`만 본다.
- root snapshot의 Event content는 `{ type: "event", node_id, event }`이고 `EventState` command
  result는 같은 `event` inner DTO를 직렬화한다. game_server와 Unity는 별도 Event envelope
  alias나 JSON repair를 만들지 않는다.
- Event inner DTO는 `event_id`, `current_scene_id`, ordered
  `presentation.panels[] { image_id, text_ids[] }`, tagged `interaction`, optional
  `committed_check`로 구성된다. `interaction`은 `advance` 또는
  `employee_check_choices { choices[] }`다. 각 choice가 `choice_id`, `label_id`, `aptitude`, `dc`,
  optional `required_choice_key`, eligibility와 해당 choice의 `employee_candidates[]`를 소유한다.
  Candidate는 identity/name/eligibility, employee/equipment/total modifier, typed
  `modifier_components`, optional `choice_key_evidence`와 exact chance를 포함한다. Scene-common
  `required_roll`과 global employee candidate list는 없다.
- Core는 stored Profile aptitude, employee eligibility, equipped-owned equipment/key 평가, 단 한 번의
  node-seeded d20, chance enumeration, committed performer/result, success action과 failure scene의
  유일한 source of truth다. `committed_check`는 source scene/choice/employee, aptitude/DC/raw,
  typed components/key evidence, modifier/total과 outcome을 저장·직렬화한다. Unity는 panel/text
  cursor와 resolve 전 choice/employee selection만 로컬로 소유한다.
- Event 진행 명령은 scene-qualified `AdvanceEventScene`과
  `ResolveEventChoiceCheck(node_id,event_id,current_scene_id,choice_id,employee_uuid)`뿐이다.
  exact committed identity의 duplicate resolve는 결과 장면에서 capability가 노출되지 않더라도
  저장된 동일 결과를 반환하며 reroll/regrant하지 않는다. 하나라도 다른 replay는 거부한다.
- starter와 본사 채용의 authored identity는 `employee_definition_id`, roster instance identity는
  `roster.employees[*].uuid`다. 본사 채용 명령은 `employee_definition_id`, 시작 선택 명령은
  `employee_definition_ids`를 사용한다. candidate DTO와 roster DTO는 같은 authored Profile을
  투영하며 UUID나 trust state에서 personality를 재생성하지 않는다. reward option instance identity는
  `rewards[*].uuid`, 전투 결과의 대표 환상체 identity는 `primary_abnormality_id`다.
- authored personality는 Employee Profile에 보존한다. `EmployeePsychology`, trust/memory와 전용 event,
  기본값/advance/query/capability는 제거됐다. 현재와 nested SavePoint의 strict codec은 제거 필드를 null/빈객체로
  보내도 거절한다. condition/성장/개인 진행은 같은 원정 복원에 유지하고 새 원정은 새 직원 identity/초기값이다.
- `RecruitmentState`는 run-seeded ordered queue와 현재 HQ active session의 definition IDs를 유일하게 소유한다. 시작 미선택 starter와 non-starter definitions를 queue에 포함하고, roster와 합쳐 unified catalog의 각 definition을 정확히 한 번 소유해야 한다.
- HQ result/snapshot은 현재 paid availability와 정의별 deployability를 통과한 active offers만 투영한다.
  선택 ID만 roster로 이동하고 session은 최대2회 채용 동안 남은 offers/횟수를 보존한다. 모두 채용했거나 한도에
  도달하면 미선택 IDs를 session 순서로 queue tail에 반환한다. 응급지원/HQ shop은 전체 session을 반환한다.
  checkpoint/Floor carry-forward는 queue/offer/횟수 소유와 순서를 그대로 보존한다.
- `UseConsumableItem` request의 직원 key는 `target_employee_uuid`이며 active modifier의 표시
  이름은 snapshot의 `active_consumable_modifier.name`이다.
- Physical Treatment는 살아 있는 직원이면 physical damage 0에서도 structurally eligible하며 성공한
  사용은 실효 회복량과 무관하게 stack을 소비한다. 다른 지속 소비 아이템은 기존 active modifier를
  원자적으로 교체하고 `replaced_modifier`를 결과에 기록하며 별도 pre-confirm을 요구하지 않는다.
- `visual_novel.source.type`은 origin metadata다. HQ briefing, event node, battle, map, loadout, tutorial 같은 source는 overlay 표시 여부가 아니라 command 구성과 완료 hook을 위한 정보다.
- `vn_id`, `background_id`, `script_id`, `speaker_id`, `portrait_id`, choice `label_id`, hint id는 Unity-local resource/localization id다. Core는 실제 대사 본문, 이미지, 사운드, cue stream을 내려주지 않는다.
- `speakers`는 배열이다. Core는 scene initial speaker/portrait/slot/active 상태를 내려주고, Unity는 local VN script resource를 통해 세부 cue와 line progression을 처리한다.
- VN definition은 scene/presentation graph만 소유한다. Session `blocking`은 matching
  `VisualNovelTrigger.blocking` 하나에서만 오며 definition-level default/fallback은 없다.

Pre-run HQ briefing은 새 게임 시작 flow의 일부다.

- `StartNewGame(game_mode)`는 run을 즉시 만들지 않고 run-start flow를 시작한다.
- 성공 결과는 `RunStartFlowStarted { game_mode, visual_novel }`이다.
- `StartNewGame` 직후 core state는 `PreRunBriefing { game_mode }`이며, snapshot에는 run/map이 없다.
- `PreRunBriefing` 중 blocking VN이 active이면 Unity는 VN progression command만 노출한다.
- HQ briefing VN이 완료되면 core가 unified EmployeeDefinition catalog의 `starter_eligible` definition을 준비하고 `SelectingStarterEmployees`로 전환한다.
- 실제 `RunState`는 `SelectStarterEmployees` 성공 시 `pending_game_mode`로 생성된다.
- mode-specific HQ VN은 live `VisualNovelTrigger` catalog의 `RunStart { game_mode }` trigger로 시작한다. Core는 run-start VN id를 하드코딩하지 않는다.
- mode-specific HQ VN trigger 또는 trigger가 참조하는 VN id가 live catalog에 없으면 invalid live data다. Core는 briefing을 조용히 skip하거나 다른 VN으로 fallback하지 않는다.

### Trigger Runtime

VN trigger는 core 내부 runtime condition을 live-authored VN session으로 연결하는 data-driven layer다.

- Trigger live data는 `game_resources/data/visual_novel_triggers/base.ron`에서 로드되고 `content_pack::load`의 최종 generation publication 경로로 검증된다.
- Core가 trigger 조건을 평가한다. Unity는 trigger 조건, priority, repeat history, checkpoint policy를 보거나 재계산하지 않는다.
- Public DTO는 기존 `state_snapshot.visual_novel`과 `battle_update.visual_novel`만 사용한다. Trigger id, priority, repeat policy, checkpoint policy, consumed history는 Unity-facing contract가 아니다.
- `blocking`은 trigger-owned authoring이다. Core는 matching trigger 값을 session/snapshot에
  그대로 commit하며 VN definition에서 재정의하거나 fallback하지 않는다.
- Initial source set은 `RunStart`, `MapNodeEntered`, `BattleStarted`, `BattleTimeReached`다.
- Live publication은 `OncePerBattle`을 `BattleStarted`/`BattleTimeReached`에만, `OncePerNode`를
  `MapNodeEntered`와 두 battle source에만 허용한다. `RunStart + OncePerBattle/OncePerNode`와
  `MapNodeEntered + OncePerBattle`은 exact trigger/repeat provenance로 fail-fast하며, battle의
  `OncePerNode`는 기존처럼 battle UUID를 node scope identity로 사용한다. Runtime도 같은 불가능 조합을
  방어적으로 거부한다.
- Event node와 VN overlay는 분리된 개념이다. Event scene 진입은 VN trigger source가 아니며
  Event presentation은 typed `selected_event.event` route에서만 진행한다.
- Trigger match ordering은 higher `priority` first, same priority는 trigger id ascending이다.
- Active VN이 있으면 새 VN session은 interrupt하지 않고 `VisualNovelRuntimeState.queue`에 들어간다. Active session 완료 시 queued session이 승격된다.
- Trigger consumed timing은 VN session이 successfully activated 또는 queued 된 시점이다. 단순 match만으로 consumed 처리하지 않고, VN 완료까지 기다리지도 않는다.
- Repeat history는 checkpoint policy별로 분리된다. `PersistAcrossCheckpointLoad` history는 top-level runtime에 남고, `RewindWithRunCheckpoint` history는 run checkpoint payload와 함께 저장/복원된다.
- Battle trigger는 produced battle update 이후 평가하고, 새 VN projection을 같은 `battle_update.visual_novel`에 붙인다. 새 blocking VN은 다음 server tick부터 전투 진행을 멈추며, 이미 산출된 tick을 되돌리지 않는다.
- `BattleTimeReached { battle_time_ms }`는 `previous_battle_time_ms < threshold <= current_battle_time_ms` crossing일 때만 match한다.

## Boss Runtime Foundation And Core Execution

현재 구현된 Boss 범위는 strict definition compilation, battle-start setup, opt-in Core runtime kernel,
교체 가능한 Little Red live reference와 명시적 nonproduction CWBS integration reference다. Public Boss
lifecycle Wire는 구현 범위가 아니며 internal graph state를 기존 DTO/checkpoint에 추가하지 않는다.

- Boss behavior는 `PveEncounter.boss_runtime`이 존재할 때만 opt-in한다. Binding이 없으면 Normal,
  Elite, NormalBoss와 FinalBoss 분류와 무관하게 기존 preview, authored route, scenario group/event/objective
  setup이 그대로 authority다.
- Production content opt-in은 `suppress_little_red` 하나다. `primary_abnormality` slot은 같은 이름 wave의 spawn
  index 0에 bind되고, `little_red_pursuit`의 finite
  `AcquireTarget -> MoveToTarget -> RejoinForwardRoute`를 한 번 실행한 뒤 resident `RouteCombat`으로 간다.
  Query는 `ActiveHostile + Reachable`, `MinPathCost`, `LockUntilInvalid`이며 target 없음, invalidation,
  path failure와 두 finite Task timeout은 모두 authored route-combat exit로 수렴한다. 이 reference는 final
  character design이 아니다. `reference_cwbs_final_boss`는 별도 `ReferenceFinalBoss` class의 nonproduction
  opt-in fixture이며 normal map/BossOmen/reward/progression 선택기에 들어가지 않는다.
- World battle planning은 private `ResolvedCombatSetupPlan` 하나에서 encounter/preview mission,
  reward와 multiplier, battlefield/player/defense/enemy setup, optional Boss actor binding, final
  `BattleScenario` validation, `BattleCore`와 initial execution을 모두 성공시킨 뒤 기존 world commit으로
  넘긴다. Attempt policy, battle UUID, entry snapshot과 active-session lifecycle은 계속 world가 소유한다.
- `BattleScenario.boss_runtime`은 optional internal setup이며 compiled encounter definition `Arc`와
  `{ actor_slot_index, ScenarioUnitRef }`의 deterministic actor mapping만 가진다. Slot은 definition order로
  정렬되고 unit ref는 preview가 확정한 opponent spawn identity다. 같은 unit을 둘 이상의 slot에 bind할
  수 없다.
- `RouteCombat` 또는 `RejoinForwardRoute`를 사용하는 bound actor는 setup 시 non-empty authored route를
  가져야 한다. Missing route/definition/slot/wave/spawn이나 non-opponent binding은 Core construction 전에
  실패하며 world/session state를 부분 commit하지 않는다.
- Compiled Pattern candidate는 resolved immutable argument frame을 capture한다. Core construction 뒤 raw
  Boss catalog, free-form argument map 또는 source RON을 다시 조회하지 않으며 새 data generation과 섞지
  않는다.
- `BattleCore`는 optional setup에서 `BossEncounterRuntime` 하나와 definition actor slot별
  `BossActorBehaviorRuntime` 하나를 만든다. Battle reset도 같은 captured setup에서 새 runtime을 만들며 이전
  battle activation state를 복사하지 않는다. t0의 모든 spawn group과 battle-start hook이 끝난 뒤, 그 시각에
  새로 예약된 actor attack/cast event보다 먼저 bound identity를 활성화한다.
- Phase, actor behavior, control node, Pattern, State와 Task entry는 battle-local monotonic activation ID를 새로
  받는다. Phase 재진입은 이전 descendant, target lock, intent, latch/counter/timer와 FixedSequence cursor를
  폐기하고 entry node부터 시작한다. Actor별 Pattern cooldown absolute ready time과 authored-route cursor처럼
  수명이 battle scope인 값은 Phase 재진입에도 유지한다.
- Runtime은 같은 authoritative snapshot에서 candidate를 평가한다. Terminal/death 판단 뒤 Phase가 actor
  control/Pattern보다 먼저 중재되고, 같은 node에서는 publication에서 고유성이 검증된 authored priority의
  최댓값 하나만 이긴다. `Immediate | AfterCurrentState | AfterCurrentPattern`은 모든 bound actor의 해당
  boundary를 기다리며 waiting actor는 새 work를 시작하지 않는다.
- `ContinueState`가 이긴 경우 TaskResult와 Pulse source는 한 번 소비하고 `UntilStateExit` latch는 State
  lifetime 동안 유지한다. 계속 참인 `Always`, StatePredicate 또는 UntilStateExit 조건의 ContinueState는
  publication에서 bounded loop guard를 요구하며 exhaustion target은 반드시 현재 edge를 떠난다.
- 지원 selector는 compiled `FixedSequence`뿐이다. Pattern admission이 성공한 commit 시점에만 actor별
  battle-scoped cooldown을 absolute battle time으로 소비한다. 다음 candidate가 cooling/no-eligible이면
  `PatternCall.no_eligible` route를 따르며 임의 Pattern 선택, hidden wait 또는 retry를 만들지 않는다.
- Phase/State/Pattern 변화는 cloned Core 위에서 cancellation, 새 activation, target/intent, cooldown과 internal
  wake를 stage하고 bounded same-time microstep chain 전체가 성공했을 때만 교체한다. Checked activation/deadline
  overflow나 microstep limit은 runtime, unit leaf state, queue/log와 cooldown을 포함한 원래 Core를 바꾸지 않는다.
- 지원 Task는 `RouteCombat`, `AcquireTarget`, `MoveToTarget`, `RejoinForwardRoute`, `Wait`, `Summon`,
  `EmitSignal`이다. Finite Task의
  absolute deadline과 wake는 authoritative battle time/event queue가 소유하므로 Pause 중 wall clock으로
  진행하지 않는다. 같은 timestamp에서는 reached/path/target 같은 domain result를 먼저 판정하고 terminal
  result가 없을 때만 `TimedOut`을 만든다. State exit 뒤 activation ID가 맞지 않는 wake는 no-op한다.
- `EnemyUnit` query는 self와 same-owner unit을 후보에서 제외하고 closed filter를 적용한 뒤 authored
  `MinPathCost | MinHpRatio` key 순서로 lexicographic compare한다. 최종 tie는 spawn order와 runtime unit ID다.
  PathCost/reachability는 canonical neighbor order의 current static battlefield grid에서 계산하며 path 없음에
  infinity fallback을 주지 않는다.
- Target lock은 `RuntimeUnit.current_target`과 분리된 Pattern 또는 resident-State activation state다. 같은 active
  hostile identity의 relocation은 lock을 유지하지만 death/withdraw/despawn은 즉시 pulse/latch와
  `TargetInvalidated` Task result를 만들고 lock을 제거한다. Runtime은 acquisition query를 암묵적으로 다시
  실행하지 않으며 다음 행동은 authored transition/outcome만 결정한다.
- `Summon`은 definition/count/pattern-scoped batch handle/placement/occupancy/insufficient-space/route/lifetime/
  removal-signal을 typed data로 받는다. Request는 atomic all-or-none이고 실패는 UUID, batch와 total count를
  만들지 않는다. Success member는 definition/summoner/sequence/batch provenance를 가지지만 actor slot,
  BossEncounter, Wave, terminal, reward, fragment와 compendium authority를 얻지 않는다.
- Summon graph의 재귀 Summon, invalid occupancy set, unbound handle/signal과 missing definition/route는 publication
  실패다. `SummonBatchCleared`는 exact batch의 모든 member death/despawn 뒤 한 번 발생하고 승리 조건이 아니다.
  Lifetime/signal/summoner removal은 먼저 성립한 원인만 적용하며 Despawn은 death/on-kill/reward를 만들지 않는다.
- Opt-in actor movement intent는 ordinary planner보다 먼저 평가한다. Move/Rejoin은 기존 continuous
  movement/collision/block/Hard-CC leaf를 사용하고, Rejoin은 actor-owned monotonic route cursor 이상에서만
  reachable authored point를 선택한다. `RouteCombat`만 ordinary route/combat authority를 다시 열고 planner는
  cursor보다 뒤의 route cell로 회귀하지 않는다.
- RouteCombat이 아닌 State에서는 queued/pending automatic basic attack과 autocast start를 거부한다. State
  cancellation은 movement와 unreleased auto windup/focus만 epoch/owned-lock으로 취소한다. Focus 취소는 해당
  스킬의 `focus_permissions`가 금지해 실제로 소유한 basic/movement lock만 exact deadline match에서 해제하고,
  같은 deadline을 우연히 공유한 다른 lock은 줄이지 않는다. 이미 release된 basic/skill projectile, area,
  delayed damage/effect는 기존 leaf runtime이 계속 소유하며 Boss cancellation이 제거하지 않는다.
- 이 internal setup은 public Wire, active-battle checkpoint/save와 Unity DTO에 새 Boss graph field를
  추가하지 않는다. Player-visible Boss lifecycle가 실제로 필요해질 때까지 기존 battle event/checkpoint
  경계를 유지한다.

## Unity-Facing Projection Boundaries

이 장은 Core가 계산해 Unity에 투영하는 actor identity, HUD 정책, range preview와 combat preview의
생산자 경계를 소유한다. JSON field shape와 소비 순서는 외부 Unity canonical이 소유한다.

### Battle Actor Identity And HUD

전투 유닛 HUD 표시 정책과 actor identity는 core가 내려주는 DTO가 source of truth다.

- Unity는 `checkpoint.units[*].hud.bar_mode`와 `hud.threat_class`를 읽어 HP bar 또는 HP+공명 bar를 그린다.
- Unity는 owner/role/base_uuid/encounter를 조합해 HUD 정책을 추론하지 않는다.
- live actor 정체성은 `UnitSpawned.unit_source`와 `checkpoint.units[*].unit_source`가 제공한다.
- Unity는 선택 패널, actor visual, 디버그 표시, 복구 시 `unit_source`를 사용하고, wave 순서나 `base_uuid` 역조회로 적 종류를 추론하지 않는다.
- `unit_source`는 actor identity source이고, `hud.bar_mode`는 HUD bar source다.
- 침식 직원/환상체/방어 오브젝트 여부를 보고 Unity가 HUD bar 정책을 다시 계산하지 않는다.
- Battle actor HUD는 유닛 아래쪽 world anchor를 `Camera.WorldToScreenPoint`로 투영해 dedicated
  screen-space overlay HUD layer에 표시한다.
- `hud.threat_class`가 높을수록 HUD 크기와 bar 두께가 커지며, HP/공명 색상은 threat class로 바꾸지 않는다.
- HUD 자체는 screen-space UI이므로 화면 기준으로 반듯하다. World-space canvas/mesh 또는
  카메라 billboard/up-vector 정렬을 사용하지 않는다.
- 적군 공격 범위는 플레이어 조작 preview가 아니므로 normal/checkpoint unit DTO에
  `range_previews`를 내려보내지 않는다. 해당 필드는 `owner == Player` unit에만 존재한다.
- 보스 경고, 광역 스킬 telegraph, debug처럼 시각적 경고가 필요한 경우에만 core가 실제 피해/효과가 적용될 타일을 별도 event/DTO로 내려보낸다.

### Range Preview Boundary

플레이어 기본 공격/스킬 범위 overlay는 core가 계산한 `range_previews` 최종 cell이 source of
truth다. 배치 전에는 `DeploymentRangePreview.payload.facings.*`, 배치 후에는
`checkpoint.units[owner == Player]`가 운반한다.

- Unity는 `defense_tile_range`, `effective_weapon_profile`, `effective_basic_attack`, skill catalog range metadata를 조합해 범위를 재계산하지 않는다.
- `owner == Player` checkpoint unit은 `range_previews`가 필수이고, non-player unit은 필드가
  없어야 한다. Unity는 적군 필드를 compatibility로 받아들이거나 빈 preview로 보존하지 않는다.
- 범위 overlay는 이동 가능 tile 표시가 아니다.
- 전장 밖/void/invalid tile은 제외하지만, obstacle/blocked tile은 기본 공격과 스킬 범위 표시에서 제외하지 않는다.
- 장애물은 지상 이동/배치/pathfinding/충돌에 영향을 주는 정보다.
- 실제 피격 가능성은 공격/스킬의 target validation, 공중 대상 가능 여부, 유효 hostile target 규칙이 판단한다.

### Combat Preview Boundary

`CombatPreview`는 node confirm, preview panel, battle setup preparation에 쓰이지만, 모든 내부 계산 필드가 Unity-facing/stored JSON 계약은 아니다.

- `enemy_stat_scale`은 floor scaling을 enemy spawn draft에 적용하기 위한 core 내부 계산값이다.
- `enemy_stat_scale`은 serialized preview JSON에 포함하지 않는다.
- draft의 MaxHealth/Attack/Defense percentage scale은 각각 현재 base에 한 번만 적용한다. unsigned scale은
  `ceil(base * percent / 100)`이고 `u128` 중간값 뒤 목적지 범위로 saturate한다. Attack은 `u32`, MaxHealth는
  `UnitStats`의 `i32::MAX` 상한, Defense는 기존 부호와 대칭 `i32::MAX` magnitude를 보존한다.
- scale 뒤 current HP는 먼저 상한 적용이 끝난 final MaxHealth에서 wide ratio product로 계산한다. full-health
  입력은 상한에 도달한 final MaxHealth에서도 full-health를 유지하며 employee trauma/bonus 순서는 바뀌지 않는다.
- AttackIntervalMs의 signed percentage modifier는 full `u64` base에서 truncating delta를 계산한 뒤
  `1..=u64::MAX`로 saturate한다. debug/release overflow나 `u64→i64` narrowing은 허용하지 않는다.
- node confirm에서 생성한 preview와 checkpoint의 `combat_previews`는 presentation cache로만 유지한다. 선택 취소,
  SavePoint, export/Continue 뒤 실제 전투를 시작할 때는 cached `CombatPreview`를 setup input으로 사용하지
  않고 현재 run node identity, manifested difficulty, encounter, live data와 deterministic node seed로
  preview-derived setup input을 다시 생성한다. 이 규칙은 ordinary/Boss/Event-started/Revival 전투에
  동일하게 적용된다.
- 노드를 소비 완료하면 해당 `combat_previews` 항목을 해제한다. 완료된 노드는 재진입 대상이 아니며,
  결과·장비 선택권은 각 durable owner가 보존한다. 미해결 노드의 미리보기는 유지하되 완료 전투의
  geometry를 이후 Continue와 SavePoint마다 중복 보관하지 않는다.
- Unity 위험도 표시는 raw scale 값을 노출하지 않고 명시적인 표시용 DTO 계약을 사용해야 한다.

Unity-facing DTO shape, command result, battle update cadence, resync 세부 계약은 외부 canonical `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`와 `/mnt/f/unity projects/ark/docs/unity_core_contract.md`를 따른다.

## Skill effect provenance

The skill executor consumes `AuthoredSkillEffect.op` in the existing order. Its
commands carry the compiled `EffectSource` into the same command processor.
`BattleEventLogEntry.skill_effect_source` is core-only metadata, skipped during
serialization and omitted from the existing transport DTO. It links the authored
address to caster, cast sequence, AbilityCast sequence and captured equipment
proc origin. It does not change event sequences or gameplay accounting.

Cast origin is captured when invocation is accepted. Released projectile and
persistent area snapshots retain it by Arc, so source attribution survives cast
cleanup, withdrawal/death and eviction of earlier log entries. It is not
reconstructed by walking a potentially truncated event log. Instant and spatial
Damage/Heal use this shared command path. Existing deferred buff/extra-attack
consequences retain their existing cause contract; full provenance for every
future derived event is not part of this minimal effect contract.

Metadata shares definition/cast records and is evicted with the bounded event
entry. The existing encoded-byte cap measures serialized bytes, not skipped
metadata heap. No separate unbounded trace collection or persistent definition
index is added. This metadata feature adds no persisted effect-source fields to checkpoint or
BattleEntrySnapshot. R02 introduced the content-bound V6 checkpoint; R05 adds required typed resources
in V7. Historical R01 V4/V5 and R02 V6 preservation evidence remains evidence of those earlier targets.

## Opt-in authoring execution control

The synchronous authoring request owns a private control scope outside cloned gameplay state.
Ordinary core calls without that scope keep their existing execution behavior. A scoped work,
queue, geometry or retained-record rejection latches the first failure; internal state rollback
cannot restore its work allowance. Fallible event/application acceptance polls that failure.
Control failure does not become a gameplay Draw or alter a gameplay timeout rule.

The effect builder returns an error before handing a partial command vector to its consumers.
It checks effect/target work and an independent command-buffer item cap. Skill commands restore
the prior effect recording source after nested command processing, including a sticky failure.
Repeat loops and deferred-step scans/releases also check the request's work allowance.

Area projection checks an authored-cell upper bound and scan/sort work before using the existing
validated tile query; it does not inject control errors into that query's invariant expects.
Cast/projectile/area creation admits retained objects before advancing their existing identity
counters. The included runtime map entries and nested cast/delivery records are counted together;
impact-context replacement removes the old vector sizes when admitting the new retained state.
These counts are not allocator bytes and do not cover every backend or transient allocation.
Fixed-projectile accounting includes its temporarily detached runtime and hit history. Area ticks
price footprint/history copies and policy-dependent history replacement. Skill definition copies,
nested usefulness scans and item trigger payloads also check work before expansion. With authoring
control active, target candidates are inspected in ID order while the existing gameplay comparators
and current-target preference are preserved. These connected paths do not establish complete clone,
all spawn/boss operations or allocator coverage; those remaining bounds and costs require separate verification.

Rapier clone preparation also tracks body/collider population peaks, pending-handle history and
broad-phase pair events from construction. Wrapper map insertion history survives backend clear,
which retains those map allocations. Its checked structural slot bounds contribute to whole BattleCore
copy admission; overflow rejects an opted-in candidate without failing ordinary gameplay.
`max_clone_slots` defaults to65536 with a hard ceiling of2^20. These storage units are not exact native
allocation bytes or a source-plus-candidate RSS measurement.

The borrowed BattleCore visitor accounts for every direct owner, including scenario, queue, runtime
maps, boss/summon/wave state, field work, result accumulators, backend, battlefield and recording
state. Native map history preserves the original hasher/table Clone; BTree and heap bounds use the
pinned native storage model. Shared definition and log-entry Arcs contribute handles, with log COW
admitted separately. Public raw fixture maps/sets are eligible only under the closed application
construction provenance; arbitrary external core imports are not certified by this visitor.
Whole BattleCore preflight and a post-copy poll are connected at the boss-boundary candidate and
summon/spawn rollback copies. The complete GameCore visitor also accounts for world state, pending
completion/grant plans, ActiveBattleSession and its independently owned entry/checkpoint copies.
Application/tactical staging and the crate-private opaque authoring fork use borrowed admission
before native Clone and poll after copying. Fresh capture constructors, remaining transient inner
copies and result projections require their own admission and verification.

Battle-entry, checkpoint, grant, employee-attempt and non-battle rollback captures now price their
selected fields before the first native copy and poll before returning the owned result. Their
private fallible callers distinguish ineligible capture from control failure. The same projected
field list drives admission and construction; omitted profile/active-battle roots are not traversed.
Run-local rollback moves the live permanent profile back into the deliberately empty captured
profile slot, avoiding a new fallible copy/allocation while restoring after a sticky failure.
Earlier caller argument copies, checkpoint encoding, remaining transient copies and final public
trial receipt/commit boundaries are separate obligations.

Spawn normalization carries control interruption separately from domain rejection; its identity
fallback only moves the original request for a domain rejection. Reservation keys, trigger IDs,
normalized requests and cached/final settlements use admitted owned copies. The final batch-result
copy keeps the rollback snapshot alive: interruption restores gameplay and the existing Pending
reservation, including after successful callbacks. It does not cache interruption as a rejection.
Boss decisions borrow compiled phases/nodes/pattern definitions from local shared owners instead
of copying active tasks/maps. Target queries and route cells are borrowed, and active summon lookup
does not create an owned view list. Detached pending requests still use admitted copies. Retained
task construction, caller arguments, collection growth and full scan bounds remain separate.
The retained summon-view list uses one complete borrowed projection before its first native copy.
The active scope prices both eligibility scans, including unselected rows, then admits the owned
output storage before allocating its exact selected capacity. Materialization polls each actor/task.
Active ordering uses stable insertion with work admitted before each String comparison and drops
the local list immediately on interruption; ordinary calls keep the existing native stable sort.
An absent runtime is distinct from a failed projection. This does not bound every upstream task
constructor or the caller's later trigger/key copies.

U17의 활성 부화 원장은 `BTreeMap`이며 `CloneShape`가 알·node·출처별 지불을 측정한다.
부화 후보 policy/progression/map/economy/vault/debit 복제는 기존 controlled clone 경계를 사용한다.
이 활성 경로는 옛 Monolith HashMap capacity provenance를 요구하지 않는다. 과거 형식/진단용
helper가 남아 있어도 현재 runtime 부화 원장이나 저장에 그 상태를 넣지 않는다.

Controlled application/tactical/boss candidates and summon/spawn rollback owners prepare their
battle log for writing after cloning. Shared retained entries are admitted before COW by entry,
nested element and source Arc handle counts plus owned string copy work. Definition metadata
behind those handles stays shared. Ordinary callers retain lazy COW and the existing event recorder.
Spawn control interruption returns an error and restores placement mutations; it is not a cached
spawn rejection. The request reservation predates that local rollback snapshot, so the outer trial
candidate must still be discarded on failure. Direct result projection copies and public session
integration remain separate accounting obligations.

Optional authoring observation must have independent limits: trace overflow records a gap and
cannot consume gameplay work/admission or reject a mandatory result. The private control scope
does not itself provide a public trial session, semantic trace, or replay receipt.

## Finite authoring scenario input and preparation

[`TrialScenario`](../src/authoring/scenario.rs) accepts an explicit player UUID, run seed and
default memory profile, followed by finite preparation/execution steps. The checked owner retains
its trusted limits and original input byte count/SHA256. Byte admission precedes parsing; duplicate
decoded keys, excess depth/nodes and unknown authoring envelope fields are rejected. Command,
Snapshot, Clock and Assert are the only step kinds. Clock grammar bounds requested deltas and their
sum; the private runtime adapter calls the actual host speed/pause and next-event ports and checks
the lifecycle witness's cumulative elapsed time before accepting each step.

Bindings use earlier public-result JSON pointers and existing, disjoint command/context destinations.
They cannot replace a root or command type. The private [resolver](../src/authoring/resolve.rs) borrows
source values, checks actual type and source address, then measures the complete substituted request
before copying. Stored ceilings apply to copied bytes per step/session and resolved bytes/nodes/depth.
Context and nested authority reject unknown fields and validate UUID/revision after substitution;
commands use the existing `ApplicationCommand` decoder and retain its actual error. Returned copy
totals are candidate values. The resolver does not change a committed application, input or quota.

Assert observes earlier public results without creating a new binding source. An absent result is
an error; an absent valid pointer can satisfy Exists(false). Invalid array indices remain errors,
while literal object keys such as "01" remain valid. Equality is checked on bounded borrowed JSON;
Exists/Type return only a scalar observation and do not copy the referenced subtree.

The private [mandatory receipt writer](../src/authoring/output.rs) bounds each result and cumulative
receipt bytes before appending. Its fixed-width numeric usage header includes encoding and header
fill work without a recursive size calculation. `result_bytes_peak` remains the attempt-wide maximum
encoded document length. Optional trace must not enter this payload or accounting. The private trial
owner supplies accepted public results, finishes mandatory preparation, polls cancellation, and commits
the complete candidate together. These functions do not establish a public trial session, full source
observation, terminal trace or final replay acceptance by themselves.


The private trial boundary now initializes a fresh memory CoreApplication inside its provenance
scope, retains the actual initial profile, and executes finite command/snapshot/clock/assert steps
through the real public ports. It prepares bounded secondary public JSON, immutable receipt-prefix
handles and mandatory encoding before the final acceptance poll. Only both outer control success
and inner domain success replace app, ledger, history and accepted copy/work/receipt totals together.
Failed candidates preserve the previously accepted aggregate. Public TrialSession now delegates to
this boundary. Native DTO construction and owned domain errors before JSON still have separate admission obligations.

Fresh authoring applications also own a fixed lifecycle witness outside saved gameplay state.
Only the actual live-entry producer creates its ordinal/UUID, and successful real zero-time stepping
marks preparation evidence. Actual execution pre/post times contribute checked cumulative elapsed;
paused ticks add none. Before terminal removal, the producer preserves actual time and verdict.
The witness survives immediate termination and retains terminal verdict after result consumption.
Application success invalidates prior readiness on actual exit/root replacement, without inferring
a new entry from a snapshot. Gameplay rewind may reuse a UUID, so accepted entry ordinals and elapsed
remain monotonic across retreat/checkpoint/new-run; failed transactions restore the complete witness.
Ordinary constructors leave it disabled. It changes no V6/profile, gameplay UUID or RNG meaning.
The private trial checks the actual cumulative clock ceiling before receipt/aggregate acceptance.
A finishing command returns the projection already prepared before active removal, avoiding a
second capability query against the removed battle. This fixed witness is not the full terminal
source capsule, effect occurrence observer, optional trace, or public preparation/reload interface.


Private preparation now runs initial construction and the entire explicit preparation prefix in
one control scope. Intermediate receipt headers are cumulative snapshots of that scope, not costs
to sum. A separate preparation-ready receipt follows the actual final readiness check; its usage
matches the final attempt work, and it occupies no scenario result index. Successful preparation
adopts the returned usage/provenance once. A private replacement prepares an independent fresh B
and swaps the complete aggregate only after B's final mandatory receipt and acceptance poll.
Late receipt/work/actual-clock failure or a prefix ending after exit preserves A. This connection
still lacks the public failure/observation/CLI surface and full source/trace retention obligations.

The actual command-result objects, nested completion wrappers, battle result converter and seven
snapshot adapters now admit their first owned JSON projection through a shared private helper.
An active scope first serializes the same borrowed DTO into a counter with no byte buffer, checking
source-text bytes and scan work per write. It then admits conversion work before the existing native
`serde_json::to_value` and polls after conversion. Ordinary calls keep a single native conversion.
Borrowed heterogeneous objects retain the existing keys, nested envelopes and Option/null meaning;
using native conversion preserves numeric semantics that encoding and parsing could change.
Source-text size is neither exact native allocation bytes nor necessarily the final Value's wire
length. Mandatory encoding checks its own actual length. The native conversion has a precharged
finite interval and a post-poll, not cancellation polls inside every allocation. Native DTO copies
before this helper remain separate work. Diagnostic message formatting now checks each UTF-8 chunk
before retaining it and checks the escaped JSON object separately. The private result mapper returns
Result, so control interruption propagates as an error through the existing public ports. Error builders
return an allocation-free unpublished error only after a typed checkpoint confirms scope interruption;
the outer scope discards it and returns the actual first ExecutionFailure. Ordinary native recovery
keeps its existing AcceptedRecoveryPending meaning and original message. Previously created domain
error strings/vectors and arbitrary formatter-internal allocations are outside this JSON boundary.


Actual authoring live setup now enables a separate mandatory battle observer before startup;
ordinary and generated-feasibility setups leave it disabled. The real entry producer binds the
checked lifecycle entry ordinal and battle UUID. Actual damage, signed HP assignment and stat
modifier producers reserve fixed observation work before mutation and commit a checked application
ordinal immediately after mutation, before logs and nested death effects. A stat modifier's HP clamp
is one Stats application. Full-health healing may be a zero-change application; an early return for
a zero request or inactive/missing target is not one. Actual HP deltas use i64 in the witness.

This observer counts HP/Stats channels, actual ModifyResonance command changes and accepted Buff
applications/reapplications, not every authored effect. It retains two fixed witness
slots (latest application and latest nonzero sourced HP), with immutable authored/cast source handles.
Those slots are not a complete history. Source-less command batches clear and restore a separate
direct observation context, so nested item effects cannot inherit the enclosing skill's source.
Occurrence ordinals do not use log sequence/cast correlation or change gameplay RNG/order/UUIDs.
Whole native clones own independent counters/witness slots while sharing only immutable source.
A reservation failure makes the outer candidate fail; it does not promise immediate stack abort or
undo the whole leaf locally. The private owner also rejects its checked invariant fault on lookup.

Before actual terminal removal, the private GameCore owner prepares an admitted outcome-ID copy
and immutable capsule owner, then moves the validated observation into it. The capsule retains the
real entry key, terminal time, outcome, verdict and mandatory witnesses after active removal and
result consumption, including time-zero terminal. The separate lifecycle still owns actual zero-time
completion. Complete application commits and nonbattle rollback preserve this owner beside the
lifecycle, outside GameCoreState/V6/profile. Retreat does not fabricate a terminal; active observation
becomes absent, and any older terminal stays explicitly keyed to its earlier entry.

Private ready/step receipts serialize a borrowed active summary and last terminal through mandatory
output admission. This observation is not a prior-public-result binding source. Optional trace is
now connected after the mandatory witness commit and contributes no flag, rows, gaps, limits or
coverage fields to this mandatory payload. Its terminal field is explicitly skipped by serialization.

Trace is disabled by default. Trusted options validate positive inclusive limits before an attempt:
rows default 1,024/hard 8,192; encoded bytes 1 MiB/8 MiB; per-row bytes 16 KiB/256 KiB (no greater
than total bytes); optional attempt work 16 Mi units/2^30. Work is logical fixed row work plus source
string scan lengths, emitted bytes, explicit new-buffer capacity and live-prefix copying. One separate
scalar TLS allowance encloses an entire preparation prefix or one advance; internal rollback never
refunds spent optional work. It neither reads nor charges mandatory execution usage/provenance.

Optional native clones share an immutable Arc<Vec<u8>> handle and reserve the same fixed mandatory
clone shape for every mode. The row writer checks encoded caps and separate work before each write.
Before serde scans strings, their borrowed lengths are admitted from the closed source graph. Shared
buffer or growth writes explicitly reserve a new Vec and copy its live prefix after accounting for
both; no hidden realloc-copy estimate is used. Count and last retained ordinal advance only after
complete JSON plus newline. A failed partial row is truncated; the first gap and dropped count remain
outside the buffer and later applications do not restart serialization. Missing optional scope,
encoding/allocation refusal or exhausted trace allowance creates a gap, never a gameplay rejection.

Actual terminal capture moves the optional owner together with the mandatory observation. Separate
borrowed active/latest-terminal views expose complete NDJSON bytes and gap counters under the actual
entry key; they do not allocate, advance clocks or construct JSON Values. This covers HP/Stats, actual resonance commands and Buff applications,
not all effects or archival history for every exited battle. Retained byte/copy bounds are not exact
allocator/RSS or whole-session coexistence measurements. Representative external nonzero terminal
provenance is verified below; remaining effect coverage, native bounds and final acceptance are unfinished.


Public [`TrialSession`](../src/authoring/trial.rs) owns a successfully prepared private aggregate.
`prepare` publishes only the complete preparation prefix plus actual ready witness. `replace` prepares
B before moving it over A. `advance` accepts one explicit step; `run_to_end` and `simulate` exhaust the
finite input, without automatically advancing to victory. Positioned failures retain the original
diagnostic phase and global step index; simulation execution failure owns the last accepted session.
No public Clone/Serialize/Deserialize/Deref or mutable inner application authority is exposed.

Receipt and step views borrow existing documents, preserving accepted None slots versus missing
steps. Receipt iteration is initial, preparation steps, ready, then execution steps. Borrowed lifecycle,
mandatory observation, exact source and terminal views expose immutable references or Copy fields.
Every initial, preparation-ready and step payload includes `source_receipt` borrowed from its
immutable validated generation: each selected file's pack ID, captured path, byte count and SHA-256.
This exact byte provenance is separate from semantic `content` identity, so inert source-byte changes
can preserve gameplay identity while producing distinct receipts. Receipt generation does not reopen
caller files. The existing mandatory writer charges and admits these bytes before candidate acceptance;
failed output/work/cancellation preserves the prior aggregate, including its source basis.
Optional trace has a separate borrowed NDJSON/gap view and is excluded from mandatory serialization.
`snapshot` and `reconnect` use independent Inspection limits/cancellation and a copy of committed
storage provenance before calling the actual application query. Their completed bounded output and
usage are returned without adopting query fuel or storage history into the accepted trial. Repeated
queries and query failures are checked against a full finite replay. Standalone external/privacy acceptance and full effect/source/bounds verification remain unfinished.


`TrialSession::preview` accepts only [`TrialPreview`](../src/authoring/preview.rs)'s two existing
application queries, with explicit owned application context. It admits the fixed input plus skill ID
and source command ID lengths before the first JSON string scan. The private command factory
projects that closed input and moves its typed fields into the matching actual domain command,
without cloning/decoding an arbitrary Value tree. Independent Inspection limits and committed
provenance cover the controlled application fork, actual public execute and bounded output. The
query candidate and its ledger are discarded. Existing authority checks, including the query kinds'
authority exemption, and tactical focus requirements remain owned by the application. Preview does
not begin targeting or choose a deployment. Native source DTO/copy admission remains a separate
obligation. Current tests compare actual deployment preview with ordinary public decode/execute
and check input/work/result/clone/cancel failures against unchanged subsequent trial receipts.

[`render_error` / `render_trial_failure`](../src/authoring/diagnostic.rs) borrow the original typed
failure and render independently of gameplay TLS, fuel, provenance and cancellation. The caller
provides a separate cancellation token and trusted DiagnosticLimits. Defaults/hard ceilings are
1MiB/8MiB source string bytes, 1MiB/8MiB encoded output, 16,384/262,144 serde nodes, 32/64 serde
nesting levels and 16Mi/2^30 work. These are inclusive logical bounds, not allocator/RSS measures.
An allocation-free private serde preflight admits string lengths before JSON escape scanning and
container lengths before iteration. It bounds nested expected/actual/application-detail Values and
related sources; unreviewed Display serialization is rejected without formatting. Buffer growth
precharges new capacity and existing prefix copying before fallible allocation; encoded chunks and
final UTF-8 validation are also charged. A final cancellation check precedes returning a completed
document and usage. Rendering failure returns a separate Copy error, discards partial output and
leaves the borrowed original diagnostic and any accepted session intact. I/O remains caller-owned.


The explicitly registered default-release [`author_content`](../src/bin/author_content.rs) executable
uses public validate, validation_receipt, describe, explain and simulate. Its form is:

```text
author_content <validate|describe|explain|simulate> TRUSTED_ROOT SELECTION.json [REQUEST.json] [--trace-output NEW_PATH]
```

Explain and simulate require a request file; the trace option is available only for simulate. Selection
uses the complete ContentSelection packs/winners schema. The executable reads at most six arguments
of 4096 encoded bytes each, 64KiB selection/explanation files and an 8MiB scenario. It checks regular
file/size metadata and bounds actual reads too. Small typed JSON retains serde's recursion limit;
scenario decoding uses the existing depth64/node262144 parser. Defaults are trusted host settings.
The CLI adds no embedded fallback, automatic gameplay selection or content-specific runner.
validation_receipt borrows the validated identity, source receipt and validation usage, admits source
file count/string scan work, and returns an independently bounded Inspection document.

Success queries write one prepared JSON document to stdout. Simulate writes the already accepted
chronological receipt documents as NDJSON. On execution failure it still writes the accepted prefix
and independently renders the original positioned failure to stderr. Preparation failure has no
accepted prefix. Exit codes are 0 success, 1 library semantic failure, 2 CLI input failure and 3 output
or diagnostic delivery failure. A stdout failure does not suppress the original semantic diagnostic;
flush failures count as transport failures too. File I/O latency and forcible process signals are not
claimed as cooperative cancellation. The library's separate cancellation API remains available.

Optional trace output is created exclusively after simulation and never overwrites an existing file.
Its format header declares active_and_last_terminal retention. Each window has a fixed bounded
header with the actual battle key, row/gap counters, then the already encoded borrowed NDJSON rows.
The header uses a 1024-byte stack buffer. This export is the final retained windows, not complete
history across prior battles. File creation/write/flush failure is transport failure and does not alter
the accepted gameplay or mandatory receipts. The executable owns no persistent player save.


The [external combat probe](../tools/authoring_combat_external_probe.py) verifies one fixed prototype
A/B through the default-release public API and CLI. Both external copies add the same existing
OnAttack skill activation to a training weapon; their only mutual source difference is the actual
Damage amount18→36 in paradise_lost_judgement/judgement_strike/effect_01. Official resources are
unchanged. A public-result-derived deployment, explicit 8000ms/12000ms clocks and CompleteCombatResult
form one fixed input for both generations. The first sourced application has matching actual
ordinal/time/cast/proc/cause/source/target and HP79→61 versus79→43. Semantic identities differ.

Actual Victory occurs at16950ms(A) and12450ms(B). After the real result-consumption command, active
is absent and the last terminal capsule, source and key remain identical to their terminal capture.
Per generation, off/on/one-byte-gap traces have identical complete mandatory receipts, accepted
work and observations; public API and CLI receipt bytes also agree. Retained normal rows match
actual HP/Stats counts22(A)/14(B); optional byte refusal records corresponding explicit gap counts.
This is a representative Instant Damage/Item-proc/terminal proof, not full effect, delayed, eviction,
allocator/coexistence or final audit coverage. The initial discovery inputs that failed to produce
sourced HP/terminal are preserved separately; only the fixed replay is a completed simulation.


The public [replacement probe](../tools/authoring_trial_api_probe/examples/replacement.rs) retains
an actual sourced A battle at accepted index11 while B acquisition and valid-token preparation fail.
Across off/on/gap, each of nine cases repeats twice: acquisition6 and replacement48 failures total.
Pre-cancel, initial work/clone, authority at step6, invalid command at step9, absent battle readiness,
and one-under final ready work/receipt limits preserve all captured public state and receipt bytes.
Measured exact B preparation work518763/receipt74768 succeeds; one-under failures retain the original
Receipt diagnostic phase and PreparationReady position. A then completes byte-identically to its
independent replay; successful B replacement matches fresh B and completes its own identical replay.
After result consumption, reconnect's existing Inspection/invalid_action is retained as an error.
This proves the public replacement/recovery boundary for this fixture; native generation-drop costs,
concurrent late cancellation and the full invalid-load producer matrix require separate evidence.


An enabled observer reserves before the existing ModifyResonance command mutates its target.
Only an actual raw before/after change commits a Copy Resonance { before, after, max } witness;
cap/lock/inactive/zero no-ops do not increment applications. The existing gain/consume/autocast
rules remain authoritative. The command's actual authored/cast source is retained, with no source
for unsourced command batches. Its hp_delta is zero and it cannot replace the nonzero sourced HP
slot. Passive resonance gain and cast reset are not counted as these command applications.
Normal gameplay bypasses the additional observation reads and reservation when observation is off.
Trace off/on/gap, actual source, clamp/lock/no-op, controlled clone and reservation rejection are
covered by focused owner tests; this channel does not imply Buff/tactical-resource coverage.

The [external resonance probe](../tools/authoring_resource_external_probe.py) adds the same self-target
resource step to two copies of the existing combat prototype and changes only its amount17→31.
At the same actual ordinal4/time7950ms, matching source/cast/proc/unit coordinates report resonance
11→28 and11→42. Each generation completes the same finite input and consumes a real Victory at16950ms.
Actual changed resonance commands count4(A)/3(B); capped no-ops do not create applications.
HP applications remain22 each, so total mandatory applications/normal trace rows are26/25.
Off/on/one-byte-gap mandatory receipts and usage agree per generation; CLI simulation bytes agree
with the public API, and CLI explanation returns the exact resource effect/source and identity.
This representative resource-channel proof leaves other effect producers and final cost/audit work open.


The private event queue now carries an optional immutable effect source beside each public
BattleEvent. Heap comparisons and same-time bucket sorting use only the original event; source
metadata is moved through pop/buckets and visited by the complete native clone accounting. Public
BattleEvent fields and raw fixture entrypoints retain their existing shape. Sourced Buff commands
reserve the fixed handle-copy work before enqueue; ordinary observation-disabled events carry None.

Actual ApplyBuff dispatch receives that source explicitly without installing it into field-work or
nested CC context. After existing definition/duration/active checks, a new Buff key is admitted against
retained entities and the observation is reserved before the first log/map mutation. The committed
Copy BuffApplicationState records stacks, expiry and next tick before/after, with actual BuffId/kind.
Existing valid reapply can assign an unchanged state and still logs/schedules/interrupts, so it counts
as an application. Unknown/zero-duration/inactive requests do not. Buff observations have hp_delta0;
they neither infer direct HP attribution for later Buff ticks nor replace the sourced HP slot.

Delayed actual apply/reapply, source-less ambient-context isolation, ordinary/observed gameplay and
off/on/gap mandatory parity, clone independence and candidate rejection/retry are exercised in owner
tests. Queue tests compare the original heap order and confirm combined source-handle release after
clone/pop/clear and rejected enqueue. These are source-handle lifetime checks, not whole generation
or allocator/RSS measurements; the queued carrier's actual memory impact remains in the cost audit.

The [external Buff probe](../tools/authoring_buff_external_probe.py) repeats the exact resource A/B
packs and finite input on the new release API/CLI. Each generation has five actual Silence
applications from judgement_strike/effect_02, correlated to the real nonlethal Damage casts. The first
is ordinal5/time7950ms, stacks1, expiry8650ms and no tick; it is also present in the trace-off mandatory
receipt at step10. Terminal trace retains the actual sourced applications after result consumption.
Total application/normal trace counts are31(A)/30(B), including HP22 and resource4/3 plus Buff5.

Both generations retain Victory16950ms. For each generation, all13 public step results, resolved
inputs, lifecycle/authority and initial provenance match the previous observer build on identical
pack bytes and input. New observation fields and inspection work intentionally differ. Current
off/on/gap mandatory receipts agree, as do the API and CLI; separate CLI Buff explanations agree
with the exact authored source, Silence700ms operation and generation identity. This is a concrete
Buff source/gameplay-preservation proof, not full effect coverage or final currentness/cost audit.

The delivery acceptance probe executes delayed repeated Instant, Projectile and persistent TileArea
HP applications through the public trial API and CLI. The replay includes real terminal consumption;
off/on/gap and repeated runs preserve mandatory receipts. Actual application ordinals distinguish
same-cast/same-time repeats without deriving identity from log correlation. A separate producer test
appends past the real 8,192-event history bound after cast capture: source survives eviction, and
normalized semantic observations, the next cast and damage identity remain equal across trace modes
and shifted log sequences. Native projectile/area IDs and queue ordering have separate regression
checks. See [experiment 052](goals/content_authoring_tool_ports/execution/single_goal/EXPERIMENTS.md#052--delayedrepeatedprojectilearea와-실제-log-eviction).
This evidence covers actual HP application occurrences; it does not expose native delivery IDs in
the authoring witness or reconstruct every action milestone from the retained history.

Positive SkillPreview acceptance also uses an external ordinary-reward fragment, its actual equip
command, a second live battle and authoritative BeginSkillTargeting. Repeated eligible queries return
ReadOnly with the same revision and preserve public snapshot/reconnect, accepted receipts and work.
The following actual manual skill damage and complete finite receipt match a replay without queries
and the CLI. The scenario ends with the second battle still active; the first terminal remains retained.
See [experiment 053](goals/content_authoring_tool_ports/execution/single_goal/EXPERIMENTS.md#053--실제-획득장착focus-뒤-positive-readonly-skillpreview).

### Exact-source Trial receipt correction

The current initial/ready/step payload carries the generation's complete captured `SourceReceipt`.
The separate public [source-byte regression consumer](../tools/authoring_trial_api_probe/examples/source_receipt.rs)
verifies equal-semantic, different-byte packs, caller-file edits after validation, all three receipt kinds,
failed replacement preservation and successful generation replacement through the same public ports.
Current API and CLI streams match exactly; changing only inert source comments changes the source receipt
while preserving all other payload values and usage, nonzero gameplay and optional trace bytes.
The current actual Damage18/36 replacement fixture admits B ready at work564123/receipt120088 bytes;
exact limits succeed and one-under limits reject without replacing A. Earlier recorded whole-receipt
sizes/work values above describe their original runs before this additional mandatory metadata.
The correction's current measurements and changed-input reuse boundaries are recorded in
[the R03 correction](goals/content_authoring_tool_ports/execution/single_goal/CORRECTION_001.md).


## Native runtime facts and action observation

`game::battle::runtime_facts` defines immutable actual-result values independent of event-log and
optional authoring-trace retention. The current `RuntimeFactView` borrows live authority; a bounded
`RuntimeFactSnapshot` is captured before battle finalization and retained by the current world result
and authoring terminal capsule. `CoreApplication::battle_runtime_facts()` exposes this observation in
ordinary release builds. Trial observation exposes `active_runtime_facts` and terminal `runtime_facts`.
The earlier R03 `MandatoryObservation` counters keep their existing meaning as a separate sibling.

Damage records the actual combined HP transition once, with the calculated components and their
known providers. Component amounts do not claim separate shares of overkill HP loss. A target's
multiple Damage effects in one delivery iteration share a strike; a new repeat, impact, tick or
basic strike has a new ordinal. Existing command damage-target counts and HP-loss gates retain their
prior meaning. Signed Heal has requested and effective changes, including accepted zero/overheal;
it is not reclassified as Damage. Employee resonance consumption at accepted input, autonomous cast
resonance reset and tactical deploy/relocate/swap pay produce Cost. Arbitrary resonance/tactical-resource
modifiers produce separate ResourceChange facts.
Rejected operations produce neither a successful zero result nor a committed fact.

Death preparation captures the victim, pre-lethal HP/position and active buff application sources
before cleanup. Mandatory death/wave bookkeeping occurs once independently of optional reactions.
The killer source and optional actual lethal Damage remain distinct from the victim's status
providers. Signed HP/stat lethal changes and direct Kill enter this boundary; Despawn stays distinct.

Common source slots independently retain actor, actual equipment, cast and authored-effect metadata.
Absent slots remain absent. Equipment includes definition UUID, deterministic transient runtime item
UUID and, when the inventory supplied one, actual persistent copy UUID. Optional log sequence is not
causal authority. Queue, cast, buff application/tick and released projectile/area owners preserve the
immutable source; changing a buff application does not rewrite an already queued tick's producer.
These records do not invent fragment identity, a complete Flow, a root, a join or reaction history.

Native `request_skill_action` reserves a result slot at successful acceptance and returns an opaque
`ActionHandle`. Query is relative to the particular BattleCore: a snapshot clone preserves existing
handles but later mutations are independent. An aborted/divergent new occurrence has a distinct
private token even if its diagnostic ordinal is reused. Tokens are immutable, cannot be deserialized,
and do not affect RNG, ordering or serialized identity. A handle owns one immutable token containing
its attempt identity and ordinal; cloned handles share that occurrence without shared mutable validity.
`take_settled_action_result` requires both
control terminal and finite effects settlement, returns the immutable receipt and releases the
reservation. It never cancels an action. Subsequent query/take is typed unavailable. Rust drop,
reference counts, query timing and later actions never evict a promised result. Existing world
ActivateSkill uses an unreserved action; bounded latest settled-action/actor witnesses preserve
observation after internal mutable records retire.

Accepted, Released, control terminal and EffectsSettled are distinct. Control terminal is Finished,
Cancelled with the existing cause, or Failed with the actual invocation reason. Finite delivery and
queued-application tickets close once; independent buff lifetime does not keep the action alive.
Withdrawal cancels pending and unreleased cast control and removes source-bound Follow areas;
released projectile/GroundFixed effects continue. Structural side casts retain the started deliveries'
result scopes and source context until finite settlement; other active casts are removed. Retained
settlement context does not restore cancelled control. HardCC and InterruptCast cancel employee
pending and future unreleased use control; applied
finite beneficial effects keep their actual leases except channels terminated by the interruption.
Autonomous sources retain their existing cancellation and cast-end cost/recovery rules. Employee death
does not consume resonance a second time; matching old CastEnd events cannot revive the cancelled use.
A piercing projectile's empty final expiry closes its carrier ticket without fabricating the old
damage-gate terminal callback or releasing a deferred step. Settlement does not rewrite a prior
control terminal or resurrect control.

Native public mutations stage BattleCore-owned state, sources, facts, queue, IDs, RNG and logs.
A bare multi-timestamp advance commits each successful timestamp and discards only the failing later
candidate; the enclosing world request may discard its complete world candidate. Native operation
fuel remains outside candidates and is shared by nested/local copies and world reconciliation.
Retry starts a fresh operation. Limit/counter failure is a typed BattleRuntimeFault, not an action
cancellation or partial success.

Finite host limits bound active actions, promised results, finite deliveries, actor/provider rows,
control steps, segments, predeath buffs, introduced retained source/result extent, operation work
and clone contribution. Admission precedes allocation. Releases do not replenish an operation's
cumulative positive retained-growth allowance. Retained witnesses count shared payload extents
conservatively; clone fuel uses the closed RuntimeFactState ownership visitor for its actual copied
handles/containers and conservative new carrier contributions. Physical container admission includes
unused B-tree slots, promised-result tree growth and temporary timestamp bucket storage. Immutable
source sharing preserves every source slot and does not change serialized observations. Shared queue
payloads are admitted again for any actual owned copy at dispatch; original event values retain their
equality and priority. Native timestamp scratch is local to one call and requires an unchanged
tracked table lineage and key order. This does not establish a hard clone
bound for arbitrary caller-mutated legacy public maps. External immutable projections/receipts may
outlive the engine and are accounted separately from engine-owned state. Live clone/reconnect is
not durable save support; current V8 checkpoints do not serialize an active battle attempt.

Implementation and current validation are owned by
[R04 execution](goals/content_authoring_runtime_facts/execution/single_goal/EXPERIMENTS.md).
Intermediate test results are not a completed R04 audit or later-R support claim.

## Typed resources (R05)

Typed resource 정의는 [data loading contract](data_loading_contract.md#typed-resource-catalog-and-execution)를
따른다. 실제 key는 resource definition ID, Employee/EquipmentCopy의 실제 persistent UUID,
그리고 현재 lifetime 발생의 private capability로 구성된다. 같은 정의의 두 장비 사본은 다른 owner다.
관찰용 scope ordinal은 mutation 권한이 아니며 종료·삭제 후 같은 숫자/UUID를 다시 보더라도 이전 token이 살아나지 않는다.
값은 sparse하게 소유하며 아직 기록하지 않은 유효 key의 읽기는 명시한 initial을 반환하고 entry를 만들지 않는다.

| Lifetime | 실제 소유·종료 경계 |
| --- | --- |
| Run | Run의 durable root. 전투에서는 entry basis를 가진 별도 overlay를 사용하고 기존 결과 승인 경계에서 한 번 반영한다. |
| Attempt | 실제 전투 시도 전체. 후퇴·종료·재시작의 새 attempt는 이전 capability와 분리한다. |
| Deployment | 실제 직원 배치 발생. 철수·사망에서 닫히고 재배치에는 새 scope를 발급한다. |
| Activation | 실제 accepted action의 제어 수명. 제어 완료·취소·실패에서 닫힌다. |
| Delivery | 즉시 payload 또는 이미 방출한 projectile/area의 실행 수명. carrier의 queued impact/tick은 원래 Delivery를 유지한다. |
| Chain | 실제 인과 root와 승인된 유한 후속 작업. source attribution 교체는 인과 context를 보존하며 독립 root는 새 scope를 가진다. |

제어자 철수는 미방출 step을 취소하지만 이미 방출한 fixed area/projectile의 유한 작업을 되돌리지 않는다.
Delivery/Chain은 명시적 completion hold로 유지되고 마지막 작업에서 정리된다. 닫힌 Activation을 후속
payload가 요구하면 stale로 거부한다. 소유자 정체성 보존과 실제 lifetime capability의 유효성은 별도로 검사한다.
범용 Flow/fork-join 또는 파편 사본 수명 문법은 추가하지 않는다.

Grant/Decay/Consume 묶음은 authored 순서의 단일 candidate다. Consume은 내부 동기 reserve/commit으로
중복 지출을 막고, 부족·owner/key/scope/revision 불일치·예산 실패는 전체 enclosing command/timestamp를
거부한다. 이미 성공한 이전 timestamp·철수는 유지한다. 중간 예약은 durable 저장이나 public mutation API로
노출하지 않으며 abort/owner 삭제/수명 종료가 보상이나 환불을 만들지 않는다. 공통 Cost/ResourceChange에는
실제 owner UUID와 resource ID/lifetime/scope, before/after, 실제 authored source를 기록한다.

Run 자원은 같은 장비 사본의 해제·직원 이전·강화에서 유지된다. 판매·분해는 실제로 사라진 사본 key만 버리고
기존 경제 규칙의 대가만 지급한다. 직원 상태 변화는 같은 UUID가 남는 동안 owner를 바꾸지 않는다. 신규 지급·구매·
보상·채용은 새 실제 owner를 승인하지만 사용 전 sparse entry를 강제로 만들지 않는다.

전투 결과는 Run의 full before basis와 after projection을 갖는다. 기존 normal/event Victory/Defeat result
게이트에서 current basis와 일치해야 반영하며 빈 결과도 revision을 한 번 증가시킨다. 이후 reward/condition/node
작업이 실패하면 같은 candidate 전체를 복원한다. 재시도는 아직 미반영된 결과만 적용하고 중복 완료는 거부한다.
후퇴는 battle-entry root와 RNG/UUID 진행을 복원한다. 층 이동은 Run root를 유지하고 terminal은 spending을 닫으며
신규 Run은 새 빈 root로 시작한다.

V8의 required `typed_resources`는 revision/closed와 정렬된 resource/owner/balance/entry-revision 목록이다.
Scope token·reservation·활성 전투는 저장하지 않는다. Unknown owner/ID, 잘못된 lifetime/cap/순서/중복과 closed
checkpoint를 게시 전에 거부한다. 같은 ID의 선언 변경도 content fingerprint 불일치다. Continue는 기존 Run을
덮어쓰지 않고 load count를 소비하지 않는다. Explicit load의 횟수 제한과 run peak/profile의 기존 예외는 유지한다.

`CoreApplication::run_resources`와 `battle_typed_resources`, authoring observation은 borrowed readonly projection이다.
관찰이 sparse 값이나 권한을 생성하지 않는다. Store/owner/scope/operation/예약/hold에는 유한 engineering cap을
적용하며 catalog 검증, clone, Run projection/비교와 retained walk를 공통 native/authoring budget에 반영한다.
Terminal의 before/after 결과 vector와 ID는 할당 전에 retained growth와 clone budget을 인수한다.
Pending 결과는 당시 battle의 native limits를 보존하고, 다음 결과 확정의 시작·최종 보유량 검사에도 적용한다.
공유 facts snapshot은 재생성하지 않는다. World/entry/checkpoint/tactical/floor/결과 확정의 자원 소유 그래프 복사도
각 실제 복사 전에 native/authoring 예산을 인수하며, 거부는 해당 application candidate를 게시하지 않는다.
이 절의 구현·직접 시험·남은 검증과 감사 상태는 [R05 실행 기록](goals/content_authoring_resource_lifetimes/execution/single_goal/EXPERIMENTS.md)이 소유한다.


## Typed amount execution and mandatory replay (R06)

DamageExpr/HealExpr are publicly executable for the validated contexts described in the data-loading contract. Snapshot captures at release, Live at consumption, and one effect pre-evaluates all targets before mutation. Finite resolved owner views use actual Side/employee/equipment-copy relations; missing identity is a typed refusal. Effects read the settled action prefix, never a future or unrelated occurrence. Failed enclosing transactions preserve accepted state, previous amount record and evaluation ordinal.

When amount observation is enabled, one immutable latest record belongs to the active battle and one to the last terminal capsule. A record binds generation, authored site, plan fingerprint, action/delivery/source and ordinal. Removing the runtime source does not invalidate an already committed record or revive action authority. Preview/explain encode that frozen evaluation, including exact i128 decimal intermediates; optional trace and event-log eviction do not control mandatory numeric records. Query work, result/metadata limits and cancellation do not mutate accepted session state. Replacement admission accounts for old plus new payload before allocation; controlled clones share immutable Arc payloads.

SpellPower is a checked stat with zero-default wire omission. Attack and SpellPower share the existing ordered Flat/Percent stack. Weapon numeric inputs, growth, ordered Permanent effects and exact persistent-copy enhancement compose once. AttackSpeedPoints stays separate from scaled milliseconds; ceil interval scaling requires positive inputs and changes only basic attack cadence. Source/Target expression readers use the original points and effective SpellPower.

The additive Certified input fingerprint includes content identity and persistent effective inputs, ordered loadout/copy bindings and fragment input. It excludes transient current HP/readiness/consumable state and does not approve a SpellPower weight in the existing power model. R06 preserves raw old V7 checksum compatibility through zero omission; new nonzero inputs survive fresh-process restoration. R07's subsequent fragment schema transition has its own explicit migration boundary.


## R07 사본별 파편과 명시 저장 전환 (2026-09-29)

스킬 파편 정의와 소유 사본을 구분한다. 단일 UUID-keyed inventory의 `OwnedFragment`가 definition, unlocked forms, selected form, equipped owner를 소유한다. 직원 active slot은 copy UUID 하나만 참조하며 양쪽 binding을 함께 검증한다. 지급은 전용 checked UUID stream과 기존 grant 후보를 사용한다. count는 사본 목록에서 파생한다. 일반 파편 강화 command와 shared rank gameplay 경로는 제거했다.

공개 ActiveSkill RON은 `forms` map의 필수 `base` 및 선택적 형태들을 제공한다. 각 형태는 skill ID, 추가 weapon compatibility, 선택적 bloom recipe를 가진다. 사본 최대1024, 형태 최대16, recipe 항목 최대64이며 중복 map key와 알 수 없는 field/reference를 거부한다. recipe의 enkephalin과 material ID/amount는 checked 합산한다. 기존 공식 콘텐츠에 recipe가 없으면 unavailable이며 임의 비용을 발명하지 않는다. 비활성 BasicAttackModifier placeholder는 intrinsic 사용만 유지한다.

`equip_skill_fragment`/`unequip_skill_fragment`는 employee UUID와 copy UUID, `transfer_skill_fragment`는 copy UUID와 target employee UUID를 받는다. 장착 슬롯이 차 있으면 이전을 거부한다. `awaken_skill_fragment`는 Maintenance에서 exact copy/form을 해금만 하고 `select_skill_fragment_form`이 별도 선택한다. 반복 해금·동일 선택은 무비용 무변경이다. `dismantle_skill_fragment`는 장착 사본일 때 confirmed=true를 요구하며 해제·사본 제거·명시 recipe 산출·typed resource owner 정리를 원자적으로 처리한다. 사망은 exact binding만 풀고 해금·선택을 보존한다.

실제 profile에서 생성한 FragmentSourceSnapshot(copy/definition/form/skill)은 runtime actor, cast/delivery, amount preview/explain 및 trace로 전달된다. FragmentCopy/SourceFragment는 기존 R05 Run/Attempt/Deployment/Activation/Delivery/Chain 수명과 같은 lease 검증을 사용한다. 장비 identity로 대체하지 않는다. 기존 delivery의 immutable source는 이후 inventory 선택에 의해 바뀌지 않는다. 새 사본·형태·owner 상태는 다음 Certified input fingerprint에 반영된다.

R07 도입 당시 live data schema5/active-run V8와 명시 사본 변환은 역사적 계약이다. 현재는 catalog schema6,
Core Continue15/독립 SavePoint13을 사용한다. Continue12는 U14 이전의 개발 형식이다. `CoreApplication::convert_active_run_v7`/`convert_active_run_v8`은
현재 미지원으로 명시 거절하며 원본을 쓰거나 새 bytes를 반환하지 않는다. 이전 V7/V8/V9 원자료와 동결 target은
당시 변환/보존 증거로 남기며 현재 수용이나 자동 fallback의 근거가 아니다.


## U05 직원 공명 수명 계약

Player Employee source의 공명은 실제 선택한 파편 form의 `resonance_max`를 사용하며, 파편이 없으면
최대100이다. 배치 기본량은 floor(max/5)이고 그 뒤 유효 InitialSkillCharge의 ceil bonus/cap을 적용한다.
직원 저장 profile은 공명 override를 작성하지 않는다. 새 배치는 기본량과 유효 배치 효과를 다시 계산하고,
전술 이동·교환은 기존 공명을 보존한다. 적·소환물 autonomous profile의 공명 owner는 별도다.

배치 직원은 매 정수 battle-second마다 +1을 얻는다. Attack/Defense/FieldWorking/Silence에서 같은
규칙을 사용하며 기본공격 방출·가한 피해·받은 피해에 따른 직원 기본 획득과 Rest 공명 보너스는 없다.
Hard CC, 실제 skill use, cap에서 버린 pulse는 적립하지 않는다. 모든 명시 양수 변경도 같은 gate를
통과한다. 음수 변경은 그 gate로 막지 않으며 Pause에는 battle clock 자체가 멈춘다.

최신 입력 검증을 통과하면 접수 transaction 안에서 전량을 한 번 소비하고 action별 사용 자격을 만든다.
기존 SkillStartGate/start/corrosion 시각은 유지한다. 이미 접수한 자격은 잔량0이나 이후 Silence 때문에
실패하지 않는다. Hard CC·필수 직접 target 상실은 미방출 동작을 환불 없이 취소한다. 직원 사망·철수는
pending 자격과 pulse를 정리하며 적의 기존 cast-end 비용 처리는 별도 경로로 보존한다.

사용 종료는 미래 미방출 동작과 명시한 finite beneficial effect의 실제 남은 수명을 함께 본다.
마지막 projectile의 명중이나 EffectsSettled만 기다리는 상태, 독립 area/summon/DoT의 생존 자체는
추가 잠금 사유가 아니다. Repeat/Branch/Parallel 뒤의 미래 action은 남은 동작이며, 마지막 추가 공격은
실제 방출까지 기다린다. 시간제 Modifiers/Overlay/Shield/Survival/Parry와 ChannelAura는 실제 lease의
만료·소진·해제까지 유지한다. CC는 채널/미방출 동작을 취소하지만 이미 적용된 시간제 buff의 수명은
보존한다. 사용 종료 후 직원에게 별도 postcast 1초 잠금을 추가하지 않는다.

readiness의 `resonance_use`는 action ordinal, skill, consumed와 Accepted/Casting/Acting/Effects를
운반한다. `resonance_gain_block_reasons`는 HardCrowdControl/SkillUse/AutonomousRecovery를 각각
투영하고 실제 양수 gate와 helper를 공유한다. 종료 직후 projection과 admission은 즉시 풀리며,
남은 bounded credential storage는 다음 gain/pulse/admission에서 회수한다. 같은 actor reconnect와
private controlled clone은 이 상태를 보존한다. durable Continue는 계속 기존 pre-entry/confirmed
경계를 사용하며 live battle 디스크 codec을 추가하지 않는다.

현재 continuation은 V12, 독립 SavePoint payload는 V13이다. V7/V8 명시 변환과 구 직원 공명/심리
입력은 현재 미지원이며 원본을 보호하고 거절한다. Host는 이전 continuation 원본이 current/backup
어느 쪽에 있어도 publish/invalidate/reset으로 덮어쓰지 않는다.

## U06 기본 행동·초기 세트·공통 interaction

Weapon `action`은 Damage 또는 Heal(flat, spell_power_percent, footprint)이다. Heal은 Instant만 허용하며
아군의 실제 부족 HP만 회복한다. 적대 피해/proc/흡혈/부활을 만들지 않는다. 단일은 획득한 아군을 고정하고
영역은 방출 시 선택 대상 위치 중심의 아군을 판정한다. damage resolve 뒤 heal resolve 순서를 사용한다.
획득은 부족 HP 비율·거리·ID 순이며 무효 고정 대상은 재선택하지 않는다. Damage overlay를 Heal에 적용하는
비Instant delivery는 접수 전 거절한다. Instant overlay는 Heal 의미를 보존한다.

시작3명과 HQ 채용은 같은 grant/equip owner로 실제 장비와 active 파편 사본을 지급한다. 로스터는10명,
동시 배치는8명이며 출전불가 조건이 TP 부족보다 먼저 표시된다. 시작 파편은 일반 owned copy여서 호환 무기
조건과 빈 active 슬롯을 만족하면 이전할 수 있다. 여섯 역할의 초기 작성값은
[U06 결정 기록](goals/core_battle_starter_tactics/IMPLEMENTATION_DECISIONS_20261005.ko.md)에 구분한다.

지상 저지는 실제 이동 segment가 반경0.75에 처음 접촉한 위치에서 정지시킨다. 현재 반경 안에서 늦게 획득하면
현재 위치를 보존한다. 관계 용량은 기존 route/spawn/ID 우선순위를 사용하며 반경 이탈/무효 actor/Hard CC는
해제한다. 현재 저지 쌍은 적대 기본 공격만 방향·기본 범위를 우회한다. Heal·active skill 범위는 우회하지 않는다.

`interaction`은 일반 unit 선택, 배치 employee 선택, 공통 restore, 설정 창 restore를 투영한다. 설정 열기는
현재 Pause/speed를 저장하고 정지한다. 조회·미리보기·닫기·후퇴만 허용하며 닫기는 저장한 상태를 복원한다.
Disconnect는 저장된 복원값까지 Pause로 바꾸므로 설정 닫기로 재개되지 않는다. 기존 U05 강제 침식 clock
예외는 유지한다. 사라진 unit/배치된 employee 선택은 reconcile에서 제거한다.

`PreviewTacticalCommand(Relocate|Swap)`는 실제 실행 validator와 최종 좌표/방향 범위를 재사용한다. TP5,
10초 cooldown, 일반칸 강제Defense1500ms/작업칸0ms 및 typed eligibility를 읽기 전용으로 반환한다. 최종
명령은 다시 검사한다. Unit checkpoint의 `basic_action`은 종류·실제 저지 관계를 항상, 현재 선택된 unit의
유효 후보·선택 대상·기본 범위 예외를 추가로 제공한다. query는 retained/work 한도 실패를 전파한다.

## U07 독립 생성물과 분절 저장 pool

`CreateObject`는 현재 진영의 Actor/Trap/Wall을 공통 Spawn Kernel candidate에서 생성한다. Actor만 일반
HP·기본공격·자동스킬을 가지며 Trap은 비유닛, Wall은 회복 불가 HP 방해물이다. 생성물은 직원·wave·보상·
required victory 참가자를 늘리지 않는다. 생성 action과 생성물의 새 action은 서로 다른 실행 권한이며,
creation cast/fragment/equipment 정보는 유한 관측 descriptor로만 이어진다.

배치·비용·ID·형태·timer·settlement는 함께 commit한다. 같은 request의 replay는 최초 결과를 반환한다.
생존/누적 상한은 creator와 실제 fragment/equipment copy 및 family별이다. 단일 자기 사본 진화는 같은 ID와
슬롯을 유지하고 새 epoch/TTL을 부여한다. HP는 증가 시 회복하지 않고 감소 시 clamp한다. 객체는 TTL·공격
소모·파괴·명시 source 정책·전투 종료에 회수되며, replay 원장은 전투 수명까지 보존하고 새 전투에서 비운다.
HP형 객체의 파괴는 일반 직원 사망/proc/시체 사실로 대체하지 않는다.

`SegmentedShield`는 기존 sustained shield remaining 하나로 HP·스택·top·저지 보너스를 파생한다. 독립
producer의 시전이 끝나면 사용 잠금도 끝난다. Wall 비용은 정확한 현재 effect 사본 또는 명시 장착 무기의
runtime copy pool에서 top을 분리한다. pool 부재는 양의 nonlethal HP 비용으로 전환하며 피해·치유로 처리하지
않는다. Heal/RechargeShield는 pool을 수복하지 않는다. 비용과 벽 생성 실패는 같은 candidate를 복원한다.

미리보기는 같은 현재 자격/비용 규칙을 사용하고 미래 step을 실행하지 않는다. 실제 방출은 현재 상태를 다시
검사한다. same-actor reconnect/clone은 live 객체·pool·epoch·Pause를 보존하며 fresh Continue는 기존 U04
pre-entry 경계를 따른다. live 객체의 새 디스크 codec은 없다. 정확한 작성 제한과 DTO 의미는
[작성 모델](skills/skill_authoring_model.ko.md#u07-독립-객체분절-pool-작성-포트), 초기 위임 수치는
[U07 정책](goals/core_independent_objects/WORKING_POLICY.ko.md)에 있다.


## U07-M trusted measurement profile and metric boundary

The `RuntimeFactLimits` 11-field layout and strict `with_runtime_fact_limits`
validation remain. The current embedded default and `ENGINE_MAX` native retained
ceiling are 16MiB; historical 8MiB inputs and refusals remain separate evidence.
An explicitly selected smaller host profile remains smaller. `RuntimeSafetySettings`
alone owns optional `development_measurement` revision1. Its nominal native DTO is
first validated against the current strict 16MiB ceiling; a separately validated
emergency retained ceiling must exceed nominal and be at most 32MiB. A legacy
8MiB nominal profile with a 16MiB measurement override has an 8MiB warning and
16MiB hard limit. This differs from the explicit authoring preset above, whose
8MiB limit has no measurement override. `native_runtime()` exposes the selected
effective owner values; private profile entry respects every owner value,
including a smaller explicit override.
Nested/deferred operations reuse the outer budget and policy. Outside-scope fact
snapshot admission uses its already trusted owner cap; an active budget wins.
Content/save/public commands do not acquire profile authority.

Observation adds fixed metadata outside gameplay ownership/accounting layout.
Seed attempted/accepted, accepted positive growth/total, maximum cumulative and
owner request, and first soft/hard exceed site/kind/request/limit are distinct.
The legacy admitted field may include failed seed attempted; it is not relabeled
as accepted. No release, discard or rollback refunds a native operation budget.
Unscoped operations, overflow and unmeasured owner state are explicit. Prepared or
failed candidate extent is not committed current. World reuses its existing final
battle/pending resource extent calculation; no new default native scan or charge.
Generation discard/commit/rollback metadata follows the actual immutable GameData
owner and session publication. Non-battle persistence/resource rollback capture
stays outside existing native scope; actual host process allocation includes it.

Host optional measurement uses a single installed System wrapper per executable,
bounded observer storage and bounded LinuxVmRSS/VmHWM sampling. Native admission,
owner extent, allocator requested bytes and OS resident bytes have different
scopes and cannot be summed or substituted. Observation loss/collector retirement
and measurement unavailable states never alter a completed gameplay result.
Normal public playback checks expected effective profile before mutation and
exports bounded input/resolved command/result/identity/native/heap/RSS records.
Same-owner reconnect/live controlled clone and existing fresh-host checkpoint
Continue preserve their original authority and lifecycle; there is no new live
battle disk codec or profile gate imposed on ordinary saving.

[The measurement guide](goals/core_runtime_measurement_profiles/README.md) preserves
the historical U07-M 16/32MiB engineering windows, replay commands and metric scopes.
U10's current 16MiB default follows the [measured normal external workload](goals/core_relic_causal_chains/execution/single_goal/product_parts/runtime001/MEASUREMENT_EXTERNAL_002.ko.md)
and [Root's recorded decision](goals/core_relic_causal_chains/execution/single_goal/PLAN.md), implemented as
[two retained-limit values](goals/core_relic_causal_chains/execution/single_goal/product_parts/runtime001/PRODUCT_009.ko.md).
The [final selected 46 tests](goals/core_relic_causal_chains/execution/single_goal/product_parts/runtime001/FINAL_RESULT_BINDING_001.json)
passed, including nominal-default public loading and smaller-host restore boundaries;
the [separate retained replay](goals/core_relic_causal_chains/execution/single_goal/test_parts/runtime001/evidence/u10_external003_measurement/runs/RUN.json)
also completed all four modes at the current nominal
16MiB without a development override. These observations do not guarantee capacity
for every content combination or complete the Goal audit. Optional measurement
ceilings still exceed nominal and remain at most 32MiB. Schema version1 and
measurement revision1 retain their formats; changing numeric values changes the
raw/effective profile fingerprints rather than requiring a format revision.
Future numeric changes preserve original before/after values, their reason and
impact review, and independent intake. U08 owns subsequent state work,
U11 content identity, U30-C Core/device sampling, U30-P Unity play, and U31 final
integration. Local Linux samples do not establish all-content or Unity maxima.

### Relic copies, numeric authority and save graph

[`RelicInventory`](../src/game/relic.rs) owns run-local copies keyed by `copy_uuid`, with a definition
ID and `Normal` or `Corrupt` variant. A separate employee loadout has exactly three optional copy
UUID slots. One copy can occupy only one placement; duplicate definitions and incompatible
definitions follow the published catalog. Equip requires a living employee. Unequip, dismantle and
death recovery release placements; death recovery keeps the owned copies. Relics are not Shop sale
items. `RunCraftingResources` keeps `equipment_dust` and `relic_corruption_catalysts` as distinct typed
counters, separate from fragment dust.

[`World` Relic commands](../src/game/world/relic_runtime.rs) validate active-run/context authority
before mutation. Corruption requires a Normal copy with an authored corruption variant, consumes one
typed catalyst and irreversibly changes the same copy UUID to Corrupt. Preview consumes nothing and
reports the required cost; it is not mutation permission. Confirmed dismantle removes and unequips
the copy and grants the definition's equipment dust; `confirmed: false` leaves state unchanged.
Failures roll back the transaction. `RequestRelicState` returns the dedicated copies/loadouts/materials
projection; these fields are not inserted into the existing inventory snapshot.

[`RelicNumericPlanDto`](../src/game/relic/plan.rs) is a read-only `prospective_loadout` explanation,
with generation, base/effective battle stats, penalties, skill/basic-attack targets, provider copy
addresses and active/dormant effects. Available amounts describe the loadout before live aura,
target and critical state. Runtime-dependent inputs remain `unresolved`; invalid bases/final
composition remain `rejected`. These addresses carry no actor/action/epoch execution capability.

The damage path uses the original validated source and immutable
[`RelicNumericBinding`](../src/game/battle/core/relic_numeric.rs). An employee's equipped variant
selects matching effects from the same generation, actor, action occurrence and exact skill effect
site/provider. A real weapon selects basic-attack provider matching without replacing the original
trigger/action provenance. Healing, summoned actors and unrelated producers do not acquire an
employee's Relic damage authority. Corrupt penalties apply to the final battle-stat projection and
do not rewrite certified power.

[`damage_runtime`](../src/game/battle/core/damage_runtime.rs) applies the existing basic-attack aura
bonus first. Relic composition then applies once: `(base + sum(flat)) * (100 + sum(percent)) / 100`,
using checked wide integer arithmetic and the existing final magnitude boundary. Skill damage first
evaluates its original compiled amount plan and then composes the bound Relic effects; an invalid
base is not rescued by a bonus. Delayed/projectile/area consumers retain the original capture,
generation, plan and source binding instead of deriving authority from the latest loadout. Snapshot
and Live retain their existing evaluation-time distinction and exact generation/site/source checks.

[`ChooseRelicCorruption`](../src/game/world/event_node.rs) is a separate free Event authority. It
requires the actual node, Event and source scene, offers eligible Normal copies or decline, and
consumes no catalyst. The committed source-scene/copy receipt advances the authored scene once;
an exact retry returns current state without applying the selection again. History rejects repeated
source scenes/copies and invalid authored paths. A selected copy may later be dismantled; if still
owned, its historical selection requires the irreversible Corrupt variant.

[`active_run`](../src/game/world/active_run.rs) and
[`continuation`](../src/game/world/active_run/continuation.rs) persist copies, three-slot placements,
typed materials, the Relic UUID stream and Event history in current and independent SavePoint graphs.
Import checks ownership, references, variants, placement and history against the bound generation;
normal Continue remains outer document V12, while standalone checkpoint payload is V13. There is no
live battle disk codec. Strong immutable owners retain content/captures/bindings until their last
owner is released; completion, rollback/discard and carrier retirement follow existing ownership.
Test-only Weak handles can observe that a particular owner is released. A successful Weak lifetime
check does not establish process RSS, all-content memory limits or deployment capacity.


### Causal relic runtime ownership

[The U10 authoring port](skills/skill_authoring_model.ko.md#u10-원인-연쇄-작성-포트) describes the closed Subscribe,
Periodic, finite Program/Fork/Join and Wedge schema. Prospective plan queries expose causal_rules,
wedge_rules and causal_capabilities, and distinguish static Dormant conditions from Pending actual-event
predicates. These owned explanations do not carry action, delivery, resource or completion authority.

A native independent action or Periodic occurrence introduces a root. Derived actions and released carriers
retain immutable copy UUID plus authored rule-ID visits. Fork siblings inherit their parent and extend
independent paths. A real named Join waits for successful participants and their finite descendants, unions
only those visits and dispatches its admitted continuation once; cancellation is not successful settlement.
Only live owners and finite carrier membership are observed. Monotonic occurrence counters remain with the
battle without retaining completed visit/receipt history. Event-log eviction and optional trace do not
control these decisions.

A generated child has its own action/site and captured structural and numeric owners. A separate immutable
causal_cause observation keeps the original root and immediate triggering actor/provider/site, actual killer,
target and relic copy/rule. It contains no execution binding or lease and cannot restore source capabilities
removed at an observation or independent-origin boundary. New victims use existing native Death/OnKill/
OnAllyDeath ordering and eligibility; basic-only procs and Corrosion suppression keep their existing gates.

Wedge counts positive original direct strikes by captured copy/rule/cast/target. Structural signed typed
composition and guards run first. The capture preserves the original base and normalized NumericProgram,
including existing legacy flat and percentage contributions. Candidate Wedge percentages add to that
program's Scale term; Replace, Add and Scale keep their existing order and single division. The original
typed and range checks remain mandatory, so Wedge cannot rescue an invalid original composition or apply
legacy numeric contributions a second time.
The candidate advances once only after positive committed HP loss; segments share its actual strike.
It does not redefine the amount observation stage or actual post-mitigation damage results.

Cold source owners, mutable Join/progress tables, queued finite continuations and owned observations use
existing work, clone and retained admission. Failure propagates to the native transaction; accepted prefixes
and retry follow existing rollback. RuntimeFactSource keeps its inline layout, while its extended allocation
and RuntimeFactState ownership have additional storage and clone work. This is not a zero-cost assertion.
Live causal execution stays transient in candidate clones; disk Continue rebuilds derived plans from the
persisted copies/loadouts and the matching content identity. No live-battle chain codec or profile ceiling
change is introduced. Actual validation and independent audit status remain in
[the U10 execution record](goals/core_relic_causal_chains/execution/single_goal/EXPERIMENTS.md).


### U13 authored facility rooms

Expedition은 publication의 단일 `expedition_default` layout을 사용한다. layout이 없는 기존 지원 pack은
기존 pool/schedule을 사용한다. Tutorial의 고정 세 층 계약은 유지한다. 현재 공식 layout은 입구 포함89방,
Start/Gate 제외846활동을 저작한 초기 콘텐츠이며 엔진 상수가 아니다.

Run snapshot의 `facility`는 전체 방/실제 연결/현재방/공개 내부지도/활동 지원상태를 제공한다. 일반방 위치와
방향은 처음부터 공개하고 이름·컨셉은 인접/방문/영구 탐사 조건으로 공개한다. 숨은 최종방과 연결은 지식이
없으면 배열 자체에서 제외한다. `current_expedition_map`/`activities`는 이번 원정에서 방문한 방만,
`interior`/`known_reservations`는 각각 공개된 구조와 확정된 고정 활동 지식만 전달한다. 이전 원정의 추첨,
상품·보상·해결 상태를 영구 지식에서 재구성하지 않는다. 표시 좌표·asset ID는 연결이나 전장 좌표가 아니다.

완료 노드로 도달 가능한 Gate에서 `{"type":"move_to_facility_room","room_id":"B1-02"}`를 보낸다.
실제 authored edge와 목적방 지원/조건을 검사하며 동층 양방향·하강만 허용한다. 도착방을 처음 방문하면 한 번
배정하고, 재방문이면 저장된 map/progression을 선택한다. 현재 map은 `RunState.map`이 단독 소유하며
`facility_run_state.rooms.visited_rooms`는 나머지 방문방을 소유한다. node UUID 기반 entry receipt와
session/attempt 상태를 방 이동으로 비우지 않는다. 이동 자체는 억제력을 차감하지 않는다.

Gate 확인은 자동 다음층 생성 대신 Gate를 완료하고 ViewingMap에 머문다. 깊이3 이상의 U12 Continue/Return
선택은 유지한다. 방 완료 지식은 실제 방 떠남 또는 Gate 정상 귀환에서 확정한다. Gate 조회/취소, 억제력
소진 조기 귀환과 실패는 새 완료를 만들지 않는다. 하강 목적방만 현재 생존 직원 트라우마10% 올림 회복을
적용하고 동층 이동/귀환은 적용하지 않는다. `descent_recovery`는 해당 하강의 미리보기이며 직원UUID 순이다.

활동 조회는 `reward_activity`, `condition`, `support`, `first_entry_cost`, `exhausts_deterrence`,
`deterrence_already_exhausted`를 제공한다. 미지원 고유 조우/최종방과 U16의 미구현 보상 유형은 실행 전에
거절하며 보상·클리어·자동귀환을 합성하지 않는다. U16-2의 SimpleChoice·ResourceInvestment·RelicGacha와, dealer policy가 작성된 U16-3 Gambling, stress_combat policy가 작성된 U16-4 StressCombat을 지원한다. 공식 유물 공급이 비어 있으면 SimpleChoice는 빈 후보와 사유를 표시하고 미이용 종료할 수 있다. 일반 보상 풀로 대체하지 않는다. 대표 유물 효과를 작성한 지원 pack으로 실제 지급·비용·저장을 검증한다. 정적 전조 호환 자리의 일반전을 U20 전조로 자동 치환하지
않는다. 기존 실제 전장 브리핑과 전투 입장 검증을 재사용한다.

Continue V17과 SavePoint는 모든 방문방의 배정/유형/해결/입장 기록을 포함한다. U17에서는 구형
Continue/checkpoint를 원본 보존과 함께 명시 거절한다. 과거 V12 호환 규칙은 현재 import에 적용하지
않으며, V17 독립 checkpoint를 Continue로 읽는 것도 지원하지 않는다.
영구 `facility_knowledge`의 현재 저장·읽기는 [Durable active-run host boundary](#durable-active-run-host-boundary)의 profile 형식을 따른다.
시설 지식 도입 당시의 profile V5/V4 묵시 읽기는 현재 reader의 지원 규칙이 아니다. Profile 지식은
monotonic merge이며 run-local 저장/실패/SavePoint가 되감지 않는다. 지원되지 않는 버전과 다른 content
configuration의 원본 보호는 기존 approved persistence 경계를 따른다.


## U14 입장·무억제·봉인·성과와 복원

`MapNodeDto.entry_preview`와 `FacilityActivityViewDto.entry_preview`는 규정/최초 청구 비용, 예상 억제력,
직원 UUID 순서의 Breakdown 유예/입장 사망, 무억제 및 전투 배율을 조회한다. 조회는 receipt를 쓰지 않는다.
전체 시설 조회는 방문방 각각의 room/slot과 원래 node로 봉인을 판정한다. 현재 방과 다른 방의 같은 slot은
서로 대체하지 않으며 회수된 가지와 부활 노드는 봉인 대상에서 제외한다. 두 공개 preview는 같은 판정을 사용한다.
receipt가 있는 재입장·정비 반복과 완료는 유예를 소비하지 않는다. trauma100 진입 유예는4이며99로 치료한
뒤 다시100이면 새4다. 사망은 전투 피해가 아니므로 전투 사망방지로 취소하지 않는다.

`BattleDeterrenceSnapshot`은 최초 비용 반영 뒤 전투 진입에서 고정한다. HP/ATK만 조우 작성 배율을 적용하고
방어·이동·사거리·저지·패턴은 그대로다. exact stat override와 지연 spawn에도 적용한다. 부활 배율과 곱하며
1.5×1.5=2.25다. 무억제는 봉인 추가 스탯과 중복하지 않는다. Victory에만 재화·장비가루·장비재료·XP 수량을
2배로 하며 기존 XP 대상 규칙과 선택횟수는 유지한다. 촉매제·파편가루·사본·알·유물·가지·억제력 보충은 제외다.
패배 생존XP10, 후퇴0; 실제 참가 생존 직원별 기본XP는 unique UUID로 한 번이다. U16은 이 전투 전체 자격
snapshot을 소비할 수 있으며 U14에 스트레스 활동 전체가 구현되었다는 의미는 아니다.

성과는 작성된 주환상체의 실제HP 손실 단일 타격 비율과 실제 전투 종료시간으로 판정한다. DOT·여러타격의
합산을 제외하고 objective별 Victory에서 한 번만 지급한다. 보너스XP는 살아있는 roster 각자 전액이다.
실전 누적 상태는 주대상 UUID와 최고 단일 비율의 고정 크기 값이며 이벤트 로그 전체를 보관하지 않는다.

황금가지 계획은 원정 최초 facility 생성에서 seed/가중치로 한 번 선택하고 상태에 저장한다. 공개 지도는
미탐사 위치·회수·현재 경로 도달가능성만 추가하며 숨긴 방/보스 이름을 공개하지 않는다. 원래 배정 노드의
봉인 해제 진입만으로 지급하지 않고 Victory에서 회수와 억제력 보충을 한 번 확정한다. 패배·부활에는 없다.
후퇴는 비용/유예/입장사망/후퇴횟수를 유지한 입장후 snapshot으로 돌아간다. 미확정 노드 Continue는 입장전
snapshot으로 복원하되 사용한 후퇴횟수를 보존한다. 결과 확정뒤 Continue/재확인은 중복 지급하지 않는다.
SavePoint는 비용과 상태를 함께 복원한다. 현재 Core active/continuation writer는15이며 이전 개발 writer는
현재 Core resume에서 명시 거절한다. Host envelope의 이전 opaque payload 보관은 Core 호환 지원이 아니다.

U14 교정: 황금가지 보충은 초기 억제력을 넘을 수 있다. Continue와 SavePoint 검증은 각 snapshot의
초기 공급량과 이미 회수한 가지의 보충량을 함께 반영하며, 초기값으로 잔량을 자르거나 정상 저장을 거절하지 않는다.


## U16-2 기본 유물 보상 활동

시설의 `reward_activity`가 SimpleChoice·ResourceInvestment·RelicGacha이면 기존 Reward session에
`relic_activity`를 둔다. 일반/전투 Reward는 기존 흐름을 유지한다. 입장 전 시설 조회의
`relic_reward`는 종류·가루 임계치·서로 다른 미장착 사본 두 개 필요 여부를 표시한다.
현재 Reward 조회는 실제 후보의 유물 정의/효과, 지불 가루, 다음 등급/추가 비용/불가 사유,
합성 입력별 자격과 선택 조합의 확률/확정 자격을 제공한다. 아직 결제하지 않은 후보는 공개하지 않는다.

`select_reward`/`claim_reward`는 선택형과 투자형에서 최종 한 사본을 지급하고 노드를 함께 완료한다.
후보는 최대 세 종류이며 부족하면 실제 개수만 표시한다. 선택형의 후보0은 사유를 표시하고
`exit_reward`로 종료한다. `invest_relic_reward`는 다음 누적 임계치까지 차액 전액을 결제하여
해당 등급 후보로 교체한다. 후보가 없거나 가루가 부족하면 기존 비용·후보·선택권을 보존한다.
교체 전 후보는 선택할 수 없고, 최고 등급에서 추가 결제할 수 없다. 투자 후 미이용 종료는 환불하지 않는다.

`select_relic_fusion_inputs`의 `copies`는 두 UUID 또는 null이다. 두 사본은 서로 달라야 하고
장착 중이면 선택할 수 없다. 같은 종류의 두 사본과 Corrupt 사본은 허용한다. 조회는
Common/Rare/Special 순서의 고정 확률표를 표시한다. `confirm_relic_fusion`은 양수 확률의
모든 등급에 투입 종류를 제외한 후보가 있는지 먼저 확인하고, 두 사본 소비·Normal 출력 한 사본·
노드 완료를 함께 확정한다. 다른 보유 종류는 후보에서 제외하지 않는다. 입력 순서를 정규화하고
기존 node seed를 사용하므로 같은 입력의 재시도 결과는 같다.

성공 `RewardGranted.relic_activity`는 노드/종류·지급 사본 diff·소비 사본·누적 지불/잔여 가루를
반환한다. 미완료 Continue는 기존 입장 전 복원을 따라 비용·후보·선택을 함께 되돌린다.
완료 Continue는 결과와 소비된 노드를 보존하며 추가 지급하지 않는다. SavePoint는 당시 자산과
노드 상태를 함께 복원한다. 실제 paired persistence 실패는 Core 상태와 두 원본 파일을 보존한다.

## U16-3 딜러와 미니게임

`dealer` policy가 있는 시설의 Gambling은 RewardSession의 optional boxed dealer owner로 진입한다. 원정 설명 이력은 RunProgression의 dealer_introduced이며 새 원정에서 초기화된다. 첫/반복 제안과 첫 게임 뒤 갬빗은 정상 authored VN 선택을 사용한다. 첫 수락/거절과 갬빗 선택은 Core가 처리하며 UI가 포인트를 지급하지 않는다. 현재 dealer 조회는 단계·revision·잔액·기본규칙/손익·배율·상품 정의/Normal상태·가격/재고·원금·담보/회수 선택/포기와 다음 자격을 제공한다. 블랙잭 hole/미래덱/RNG는 공개하지 않는다.

정상 명령 `dealer_act`는 `{revision, action}`을 갖고 action의 type으로 choose_game/start_round/play/forfeit/continue/open_collateral/select_collateral/cancel_collateral/confirm_collateral/settle/select_redemption/confirm_redemption/purchase/leave를 구분한다. choose_game의 game은 Blackjack/DicePoker/Flip7이며 wager는 앞의두게임에서 양수 부호정수 판돈, Flip7에서는0(공개된고정손실사용)이다. gambit은 첫게임 선택 뒤VN에서 확정하며변경하지않는다. Ready에서는cancel_game이무차감이고 Playing에서는forfeit이현재판손실1회다. play.action은 hit/stand/double/roll(held5개)/target(seat0..2)이며해당게임·턴·대상에맞지않으면무변경오류다. revision이오래된중복명령은거절된다.

잔액은음수를보존하며양수에서만새판을시작한다. 갬빗은게임순손익에만적용하고기본지급·담보원금·상환·가격에는적용하지않는다. maximum_loss와double_maximum_loss는현재가능한손실이며잔액전체를배율로곱하지않는다. 정상블랙잭/dice/Flip7의판본·타이브레이크·NPC·스코어순서는 [채택 규칙](goals/core_dealer_minigames/RULES.ko.md)을따른다. 판수게임상한이나판이력누적을추가하지않으며현재판·마지막결과만유지한다.

미장착사본은loan확정에서session escrow로이동하여다른사용에서제외한다. 회수는원래UUID/variant를그대로반환하고포기는그사본만제거한다. 정산→담보회수/포기→상점→나가기순서를고정한다. purchase는현잔액/재고/공통grant를동일transaction에서반영한다. 잔액0/음수여도상점을보고나갈수있으며leave/제안거절에서만조우완료와노드소비를확정한다. 기존ClaimReward/ExitReward/CompleteNode로정산을우회하지않는다.

같은process 조회는현재게임/담보/구매를유지한다. 미완료 fresh Continue는게임난수·설명이력·포인트·담보·중간구매를함께입장전으로복원한다. 완료Continue는원금회수/포기/구매/노드소비를보존하고SavePoint는당시전체상태로복원한다. paired publish 실패는메모리와active/profile 원본을보존한다. 현재 공식 유물공급 공백은빈상품목록이며대표외부pack의효과/상품검증과공식콘텐츠채택을구분한다.

## U16-4 스트레스 연속 방어전

시설의 `stress_combat` 정책이 있는 StressCombat은 Reward 배정에서 기존 live Defense 전투로 진입한다. 전용 PVE class `StressCombat`은 일반 지도 조우 추첨에서 제외된다. `entry_preview.stress_combat`은5라운드·첫 보상·실제 수량 배율을 입장 전에 공개하고 combat_deterrence는 전 라운드에 고정할 적 강화 자격을 제공한다.

`game_state_context.stress_combat`은 battle_uuid/current_round/completed_rounds/awaiting_choice/quantity_multiplier/pending_rewards/next_rewards를 제공한다. 배율은 안내에 반영된 값이며 서버가 안내값을 다시 곱해 정산하지 않는다. `stress_combat_act` 명령은 `{battle_uuid, completed_rounds, extract}`와 공통 battle authority를 요구한다. 선택 구간에서 extract=false는 같은 BattleCore로 다음 라운드, true는 누적분을 받고 성공 종료한다. 오래된 UUID/라운드·선택 밖 호출은 거절한다. 선택 구간은 조회·재접속과 이 선택만 허용하며 전투 시간과 주기 자원/쿨다운이 진행하지 않는다.

개별 라운드에는 노드 소비·실제 보상·기본 XP를 처리하지 않는다. 성공 철수 또는5라운드 완료는 고정 누적분을 ClaimAll과 기존 grant로 정산한다. 패배는 미수령 전부 상실하며 직원 손실과 기본 생존 XP는 공통 결과에서 전체1회 적용한다. 성공 무억제는 해당 수량/기본 XP2배·유물 수량 유지, 패배 기본 XP는 기존량이다.

전투중 RetreatBattle은 기존 snapshot으로 전투 변화/소비를 복원한 뒤 노드를 소비한다. 최초 비용/입장 receipt 유지·XP 미지급·재도전 불가이며 일반 후퇴2회 안내를 이 활동에 적용하지 않는다. 미완료 fresh Continue는 입장전으로, 완료된 성공/패배/후퇴는 결과와 노드 소비를 함께 복원한다. 같은 프로세스 재접속은 현재 라운드 선택을 유지한다. SavePoint와 paired publication은 기존 owner를 사용한다.
