# Simulator 콘텐츠 제작 기반 — 전체 의존성 검토·개정 보고서

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

## 결론

**기존 M0 성과와 GOAL-CAMEC-01은 유지하고, 후속 로드맵은 필요한 최소 계약 중심으로 재분할하는 안을 권고한다.** 기존 `ValidatedGameData` 게시 경계, 계산/전달 kernel, 장비 사본/loadout transaction, 세계 상태 복제/commit, 보스 graph, SpawnKernel과 wave bookkeeping은 재사용할 수 있다. 현재 근거로 코어 전체를 다시 써야 한다고 판단하지 않는다.

가장 큰 변경은 **보스 최소 호출·침식을 앞당기고**, **개인별 유물 합성의 지연 실행 소비자를 동시 전환에 포함하며**, **새 상태의 저장/복원을 도입 단계에 명시적으로 배정**한 것이다. R20/R23은 각각 복합 저장·최종 부하 검증이며 최초 저장/예산 도입이 아니다.

## 제출물과 확인 수준

| 결과 | 내용 |
| --- | --- |
| A. 권장 순서 | [R00 인수 + R01–R24 기본 순서](../roadmap_v2.2-review/ROADMAP.ko.md) |
| B. 전체 대응 | [M0–M8 / CA00–CA23 전체 대응·재사용·한계](M_CA_MAPPING.ko.md) |
| C. 관계 근거 | [기존 49개 edge disposition + 새 76개 typed 관계 + 16개 최소 동시 전환](../roadmap_v2.2-review/DEPENDENCIES.ko.md) |
| D. 단계별 전환 | [실제 개정 JSON](../roadmap_v2.2-review/GOALS.json) 및 [R00–R24 작업 카드](../roadmap_v2.2-review/goals/R01.ko.md) |
| E. 최소 Goal 영향 | [A 권고·exact artifact·보존/수정 owner](GOAL_CAMEC_IMPACT.ko.md) |
| F. 개정 배포본 | [별도 경로 적용 안내](../APPLY.ko.md) · [현황 추가 기록안 — 미적용](../ROADMAP_STATUS_ADDENDUM.ko.md) |

현재 저장소 **97개 파일의 기재된 구간**을 본문으로 읽었다. 전체 파일/전체 저장소/전체 COV의 완독률은 아니다. 기존 GOALS 946줄, A01–A08, 로드맵 계약/검증/정책 gate, 선택된 formal 원본과 핵심 code/RON 경계를 대조했다. **게임 테스트·빌드·runtime probe·성능·독립 감사는 실행하지 않았다.** 실제 수행한 검사는 배포 문서/JSON/그래프 정합성 검사이며 결과는 별도 receipt에 둔다.

읽은 source·범위·앵커/일부 Git 식별자는 [근거 색인](../evidence/SOURCES.ko.md), 중요한 조사 묶음은 [조사 기록](../evidence/RESEARCH_LOG.ko.md)에 있다. [미검토·조건·재개 위치](LIMITATIONS.ko.md)의 제한을 함께 적용한다.

## 현재 작업공간과 권한

대상은 연결된 simulator의 dirty 작업 트리다. 첫 확인은 branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414`, dirty=true였다. `ROADMAP_STATUS.md`는 v1.9 / 기록 0010이며, CA01의 P2G-60이 완료되고 다음 카드는 NEXT-0009다. 원래 자료의 당시 값만 되풀이한 것이 아니라 현재 본문을 읽어 인수했다. [S001](../evidence/SOURCES.ko.md#s001) · [S002](../evidence/SOURCES.ko.md#s002) · [S066](../evidence/SOURCES.ko.md#s066) · [S093](../evidence/SOURCES.ko.md#s093).

도구는 현재 workspace의 read/search/list/status/diff와 과거 실행 기록 조회만 제공한다. 코드 실행 기능 또는 repository write 기능은 없었다. 따라서 이 배포본은 **로컬 산출물**이며 저장소에 기록했다고 주장하지 않는다. 사용자 dirty 변경·정식 원본·production/RON/save는 수정하지 않았다. 전체 dirty 파일의 원자적 snapshot/hash는 없으므로 일부 source 식별자로 전체 현재성을 인증하지 않는다.

## 하나의 권장 기본 순서

| 묶음 | 권장 순서 | 주요 이유 |
| --- | --- | --- |
| 인수/최소 전환 | R00 기존 M0 인수 → R01 현 최소 Goal | 현재 게시·실행 kernel을 이용하는 metadata slice는 독립 유효하다. |
| 제작 입력/관찰 | R02 외부 팩·세대 → R03 공개 도구 | 새 런/저장과 실제 외부 RON 검증 포트를 먼저 닫는다. 도구 전체가 kernel 기능 선행인 것은 아니다. |
| 공통 의미/상태 | R04 사건·결과·ActionHandle → R05 자원/수명 → R06 계산/성장 → R07 파편 사본 | 현 출처·실제 결과를 바탕으로 owner와 상태/계산을 분리하고 사본의 정상 지급/저장까지 닫는다. |
| 큰 후반 묶음에서 분리 | R08 보스 최소 호출 → R09 침식 | 전체 Flow/shield/trap/유물 chain을 기다릴 이유가 없다. 침식 정책 미정 때는 해당 분기만 유예한다. |
| 실행 표현력 | R10 Flow/대상/전달 → R11 가역/집중/기본 공격 → R12 보호막/방어 → R13 매개체 | 각 primitive가 실제 실행·종료·실패를 소유하며 후속 recipe를 위한 안정 계약을 만든다. |
| 유물과 공간 반응 | R14 관리+최소 numeric 실행 → R15 closure/구조 recipe → R16 연쇄 → R17 관계/감염/집계 | 개인 plan과 async provenance를 먼저 닫고 compile-time closure와 runtime chain을 구별한다. |
| 통합/제작 인수 | R18 보스 고급 확장 → R19 등록/획득/평가 → R20 결과/저장 통합 → R21 공식 RON/표현력 → R22 독립 제작 → R23 안전/부하 → R24 독립 감사 | 각 단계의 초기 정상경로/저장을 다시 통합 확인하며 독립 증거로 마무리한다. |

기본은 **순차 진행**이다. 기능상 필수 DAG와 위 전체 순서는 다르다. 정책/기능별 대안과 병렬화 조건은 DEPENDENCIES 및 각 카드에 따로 기재했다. R09가 미정이면 R10–R19의 독립 기능은 진행 가능하지만 R20 이후 관련 전체 완료는 주장할 수 없다.

## 반드시 같이 바꿔야 하는 핵심 경계

| 동시 경계 | 분리하면 생기는 문제 | owner |
| --- | --- | --- |
| metadata/compiler + latest RON/catalog + 직접 consumer | parser만 새 정의를 받고 runtime은 잘못된 기존 값/위치를 소비 | R01 / 기존 AG-CAMEC-01 |
| 외부 게시 + 새 런 데이터 선택 + fingerprint save/restore | state-only commit에서 세대 선택 소실, 같은 ID 다른 내용 저장 오복원 | R02 |
| 사본 aggregate + grant/equip/form/dismantle/recovery + profile/snapshot/save | 개화/소유는 사본인데 장착·분쇄·저장에서는 종류로 합쳐짐 | R07 |
| 개인 effective plan + cast/queued/projectile/area + 개인 preview/평가입력 | 투사체 적중 때 전역 SkillId 원형으로 되돌아감 | R14 |
| 침식 대체/kill/suppression + withdraw/attempt 비용 + clock + 결과/Breakdown/save | 마지막 적 처치 후 대가 누락, wave 중단, physical damage 잘못 증가 | R09 |
| 복수 보상 schema + plan/commit/validator + normal grant/save | 등록만 되고 실제 선택·재지급·완료 기록의 의미가 엇갈림 | R19 |

위 표는 최소 경계를 요약한 것이며 전체 16개 묶음은 DEPENDENCIES에 있다. RelicCopy 관리와 Composer는 도메인 책임을 분리한다. 관리-only를 먼저 구현하는 분할도 유효하나 그 단계의 완료를 “효과 합성 실행 완료”로 부르지 않아야 한다. 본 기본안은 공용 코드 재작업을 줄이고 유물의 실제 동작까지 닫기 위해 R14에서 함께 인수한다.

## 세 가지 의존성의 구별

**현행 호출 의존**의 예는 projectile impact가 SkillId로 정의를 다시 조회하는 것이다. **목표 전환 의존**은 개인 plan을 도입할 때 그 impact consumer도 plan reference를 소비하도록 같이 바꾸는 것이다. **진행상 권장**은 공개 CLI를 먼저 준비해 뒤의 RON 인수를 일관되게 하는 것이다. 이 셋을 같은 굵기의 필수 화살표로 표현하지 않았다.

기존 CA03→CA04, CA08→CA10, CA09/13/16→CA18 등은 전체 기능 선행으로 유지하지 않았다. 반대로 source binding·지연 실행·저장 reader/writer·직원 대가처럼 분리하면 무효가 되는 부분은 명시적인 최소 계약/동시 묶음으로 남겼다.

## 상세 조사 결과

아래 “관찰”은 기재된 현재 본문, “설계 판정”은 목표에 대한 권고다. “잘못 나누었을 때”는 전환 위험이며 모두 현재 게임에서 발생한 버그라는 뜻은 아니다.

## F01 — 현재 최소 Goal을 첫 구현으로 유지

**관찰.** Formal Freeze/Plan/Goal/Handoff는 효과 metadata와 literal Damage/Heal/Instant의 실제 연결을 한정하고, 새 사본/generation/save/전체 Flow를 제외한다. 현 executor와 validation/publication은 이미 존재한다.

**설계 판정.** 권고 A. AG-CAMEC-01과 WP03은 재사용하며 미래 계약 전체를 필수 선행으로 더하지 않는다.

**잘못 나누었을 때.** metadata 주소를 영구 copy나 runtime occurrence로 사용하거나 전체 후속 기능을 stub로 고정하면 이후 전환이 막힌다.

**후속 owner:** [R01](../roadmap_v2.2-review/goals/R01.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S007](../evidence/SOURCES.ko.md#s007) · [S008](../evidence/SOURCES.ko.md#s008) · [S009](../evidence/SOURCES.ko.md#s009) · [S010](../evidence/SOURCES.ko.md#s010) · [S011](../evidence/SOURCES.ko.md#s011) · [S065](../evidence/SOURCES.ko.md#s065) · [S066](../evidence/SOURCES.ko.md#s066) · [S092](../evidence/SOURCES.ko.md#s092) · [S093](../evidence/SOURCES.ko.md#s093).

## F02 — 외부 팩은 기존 게시 경계를 확장하되 런 생성까지 연결

**관찰.** live_ron은 고정 embedded source. data/mod는 모든 검증과 generated setup 뒤 opaque token을 반환한다. GameCore가 Arc DB를 보유하며 PreparedApplicationCommit은 state/seed만 옮긴다.

**설계 판정.** R01 embedded 최소 slice는 독립 유효하다. R02는 외부 source·publication·briefing부터의 pin·content fingerprint reader/writer를 동시 전환한다.

**잘못 나누었을 때.** draft에 B data만 대입하면 state-only commit 뒤 B선택을 잃을 수 있다. 같은ID다른내용 저장을 checksum만으로 허용하면 잘못된 정의로 복원한다.

**후속 owner:** [R02](../roadmap_v2.2-review/goals/R02.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S012](../evidence/SOURCES.ko.md#s012) · [S013](../evidence/SOURCES.ko.md#s013) · [S014](../evidence/SOURCES.ko.md#s014) · [S015](../evidence/SOURCES.ko.md#s015) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S085](../evidence/SOURCES.ko.md#s085) · [S094](../evidence/SOURCES.ko.md#s094).

## F03 — 효과 metadata 도입은 현재 계산·공유 전달 의미를 보존

**관찰.** SkillStep은 target/delivery를 공유하고 effects가 순서대로 처리된다. ModifyDamage는 step의 Damage에 묶이고 signed Heal·HP-loss gate·repeat가 실행 경로에 존재한다.

**설계 판정.** R01은 주소/descriptor만 연결한다. R10/R15에서 effect와 carrier의 분리·변형을 명시하고 원형 영향을 시험한다.

**잘못 나누었을 때.** 피해 effect footprint 수정이 자기회복이나 공유 투사체 수·타격 인정까지 바꾸면 승인받지 않은 동작 변경이다.

**후속 owner:** [R01](../roadmap_v2.2-review/goals/R01.ko.md), [R10](../roadmap_v2.2-review/goals/R10.ko.md), [R15](../roadmap_v2.2-review/goals/R15.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S016](../evidence/SOURCES.ko.md#s016) · [S017](../evidence/SOURCES.ko.md#s017) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S024](../evidence/SOURCES.ko.md#s024) · [S070](../evidence/SOURCES.ko.md#s070) · [S097](../evidence/SOURCES.ko.md#s097).

## F04 — 개인별 합성은 지연 전달의 전역 SkillId 재조회와 함께 전환

**관찰.** projectile과 area 적중 경로는 전역 skill_data.get_by_id로 다시 정의를 읽는다. sim/queued step/preview/readiness에도 정의 기반 소비가 있다.

**설계 판정.** R14의 최소 numeric effective plan부터 cast·queued·projectile·area·개인 preview/evaluation input이 같은 immutable plan/binding을 소비하게 한다. 전역 catalog 설명 자체는 유지한다.

**잘못 나누었을 때.** 장착/Explain에는 강화가 보이지만 지연 적중은 원형으로 실행되거나 같은 SkillId의 두 직원 효과가 섞인다.

**후속 owner:** [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R15](../roadmap_v2.2-review/goals/R15.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S020](../evidence/SOURCES.ko.md#s020) · [S021](../evidence/SOURCES.ko.md#s021) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S091](../evidence/SOURCES.ko.md#s091).

## F05 — 영구 장비 사본 ID가 전투 source에 그대로 전달되지 않는다

**관찰.** inventory는 OwnedEquipment.instance_uuid를 가진다. player_spawns는 base UUID와 enhancement를 전달하고 build는 전투 owner/slot 기반 RuntimeItem ID를 만든다.

**설계 판정.** R04에서 실제 장비 copy→draft→runtime binding을 추가한다. 파편 copy binding은 R07이 같은 포트로 확장한다.

**잘못 나누었을 때.** base UUID/슬롯을 copy ID로 간주하면 동일 장비 두 사본의 자원·출처·유물 재방문이 합쳐진다.

**후속 owner:** [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S034](../evidence/SOURCES.ko.md#s034) · [S064](../evidence/SOURCES.ko.md#s064) · [S022](../evidence/SOURCES.ko.md#s022) · [S027](../evidence/SOURCES.ko.md#s027).

## F06 — 사건·결과와 기존 basic trigger는 서로 다른 계약

**관찰.** damage_runtime/commands는 HP commit과 결과 기록, triggers는 현재 occurrence별 기본공격 의미, death_runtime은 wave 기록 뒤 optional OnDeath/OnKill 등을 실행한다.

**설계 판정.** R04에서 request/fact/receipt를 구별하고 R09/R16이 suppression과 새 반응을 소비한다.

**잘못 나누었을 때.** OnHit를 모든 Damage로 확대하거나 사망 경로 전체를 억제하면 기존 발동과 wave 진행이 바뀐다.

**후속 owner:** [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R09](../roadmap_v2.2-review/goals/R09.ko.md), [R12](../roadmap_v2.2-review/goals/R12.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S023](../evidence/SOURCES.ko.md#s023) · [S024](../evidence/SOURCES.ko.md#s024) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S028](../evidence/SOURCES.ko.md#s028) · [S053](../evidence/SOURCES.ko.md#s053).

## F07 — 진단 로그는 새 인과/결과 authority의 저장소가 아니다

**관찰.** BattleEventLog는 개수/바이트/시간 제한의 diagnostic window다. recording은 accumulator 관찰을 append 앞에 두지만 sequence exhaustion에서는 앞서 반환한다.

**설계 판정.** 새 ChainContext/receipt를 retention과 분리한다. log on/off·eviction뿐 아니라 sequence exhaustion 영향도 R04/R23에서 검사한다.

**잘못 나누었을 때.** eviction으로 재방문 보호가 사라지거나 진단 출력 여부가 새 결과를 결정하면 재현성과 안전성이 깨진다.

**후속 owner:** [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md), [R23](../roadmap_v2.2-review/goals/R23.ko.md). **확인 한계:** 정상 append/retention 경계는 본문 확인. 극단 sequence exhaustion의 전체 게임 영향·게임 테스트는 미검증이며 현재 안전하다고 인증하지 않는다.

근거: [S071](../evidence/SOURCES.ko.md#s071) · [S072](../evidence/SOURCES.ko.md#s072) · [S027](../evidence/SOURCES.ko.md#s027).

## F08 — 파편 사본화는 종류별 장부와 모든 소비자의 동시 전환

**관찰.** SkillFragmentInventory와 loadout/progress는 definition ID를 중심으로 수량·개화·형태를 공유한다. grant/분쇄/profile/snapshot/checkpoint도 이를 읽는다. 장비에는 사본/양방향 loadout transaction이 있다.

**설계 판정.** R07에서 독립 OwnedFragment와 정상 grant/equip/awake/select/dismantle/death recovery/profile/save를 함께 바꾼다. 장비 장부는 재사용한다.

**잘못 나누었을 때.** A사본만 개화해도 B가 바뀌거나 저장/분쇄/장착에서 copy identity를 잃는 중간 상태가 된다.

**후속 owner:** [R07](../roadmap_v2.2-review/goals/R07.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S031](../evidence/SOURCES.ko.md#s031) · [S032](../evidence/SOURCES.ko.md#s032) · [S033](../evidence/SOURCES.ko.md#s033) · [S034](../evidence/SOURCES.ko.md#s034) · [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S038](../evidence/SOURCES.ko.md#s038) · [S041](../evidence/SOURCES.ko.md#s041) · [S042](../evidence/SOURCES.ko.md#s042) · [S064](../evidence/SOURCES.ko.md#s064) · [S091](../evidence/SOURCES.ko.md#s091).

## F09 — 계산·성장·표시/평가 입력은 같은 owner를 가져야 한다

**관찰.** stat_pipeline이 여러 성장/장비 입력을 합친다. 현재 UnitStats에는 제안된 SpellPower가 없고 Certified는 active skill의 존재를 정액 평가한다.

**설계 판정.** R06에서 typed 식·스탯/무기성장·projection/평가 입력을 연결하고 R07에서 파편 rank를 제거한다. 새 Certified 가중 모델 완료는 R19/PG08이다.

**잘못 나누었을 때.** 무기 성장 이중합산, Magic=SpellPower 혼동, 원형 스킬 기준 평가를 개인 effective 평가 완료로 오인한다.

**후속 owner:** [R06](../roadmap_v2.2-review/goals/R06.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R19](../roadmap_v2.2-review/goals/R19.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S030](../evidence/SOURCES.ko.md#s030) · [S043](../evidence/SOURCES.ko.md#s043) · [S034](../evidence/SOURCES.ko.md#s034) · [S062](../evidence/SOURCES.ko.md#s062) · [S063](../evidence/SOURCES.ko.md#s063) · [S064](../evidence/SOURCES.ko.md#s064) · [S095](../evidence/SOURCES.ko.md#s095).

## F10 — 전체 작성 도구는 kernel 기능상 선행이 아니다

**관찰.** Cargo autobins=false이며 명시 target들은 test-fixtures 기반이다. 기존 skill_runtime_contract에는 4step·자기회복1회·buff스택반복3회를 assert하는 실제 kernel fixture가 있다.

**설계 판정.** R03을 먼저 권장하되 CA03→CA04를 기능 필수에서 권장/공개 인수 검증 관계로 바꾼다. 각 opcode는 descriptor/negative fixture를 함께 소유한다.

**잘못 나누었을 때.** CLI가 없다는 이유로 resource/action kernel 자체를 불가능하다고 판정하거나 privatebuilder 성공만으로 외부 제작을 완료 선언한다.

**후속 owner:** [R03](../roadmap_v2.2-review/goals/R03.ko.md), [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R05](../roadmap_v2.2-review/goals/R05.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S068](../evidence/SOURCES.ko.md#s068) · [S069](../evidence/SOURCES.ko.md#s069) · [S070](../evidence/SOURCES.ko.md#s070) · [S013](../evidence/SOURCES.ko.md#s013) · [S085](../evidence/SOURCES.ko.md#s085).

## F11 — 보스 최소 명시 시전은 독립적으로 앞당길 수 있다

**관찰.** BossTaskData와 실제 runtime match에 명시 Skill invoke가 없다. 기존 계층 graph·task result/activation/cancel·bounded microstep은 존재한다. 일반 수동 스킬 경로는 Player 전용이다.

**설계 판정.** R08에서 raw/compile/action authority/result/facing을 동시에 확장한다. 기존 전달체 handle만 필요하고 전체 shield/Flow/trap은 후속이다.

**잘못 나누었을 때.** Player manual API를 그대로 호출하거나 invoke_ability를 직접 호출해 eligibility를 우회하고 Accepted를 Completed로 오인한다.

**후속 owner:** [R04](../roadmap_v2.2-review/goals/R04.ko.md), [R08](../roadmap_v2.2-review/goals/R08.ko.md), [R18](../roadmap_v2.2-review/goals/R18.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S018](../evidence/SOURCES.ko.md#s018) · [S019](../evidence/SOURCES.ko.md#s019) · [S022](../evidence/SOURCES.ko.md#s022) · [S084](../evidence/SOURCES.ko.md#s084) · [S094](../evidence/SOURCES.ko.md#s094).

## F12 — SpawnKernel은 재사용 가능하나 상위 소환은 보스/적에 결속

**관찰.** SpawnKernel은 batch 배치·idempotent request·검증/원자성 경계를 가진다. summon_runtime은 Boss task와 Side::Opponent, route/actor behavior를 소비한다.

**설계 판정.** R13은 공통 하위 kernel을 재사용하고 actor/trap/fieldobject capability와 source/lifetime adapter를 분리한다.

**잘못 나누었을 때.** 아군 함정을 적 보스로 생성하거나 speed=0으로 actor validation을 우회하고 despawn을 kill로 세는 문제가 생긴다.

**후속 owner:** [R13](../roadmap_v2.2-review/goals/R13.ko.md), [R18](../roadmap_v2.2-review/goals/R18.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S049](../evidence/SOURCES.ko.md#s049) · [S050](../evidence/SOURCES.ko.md#s050) · [S051](../evidence/SOURCES.ko.md#s051) · [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053).

## F13 — 전장 작업 완료 신호에는 raw→runtime의 동시 확장이 필요

**관찰.** field_work의 실제 완료 효과는 RepairProtectedUnit이며 작업 진행·assignment·완료소모를 소유한다. Wave는 actor death epoch와 binding으로 진행을 평가한다.

**설계 판정.** R18에서 typed completion signal을 schema/compile/executor/boss 구독까지 연결하고 진행 authority는 유지한다.

**잘못 나누었을 때.** 작업 완료로 boss 상태만 임의 바꾸거나 소환/보스가 wave/보상을 직접 승인하는 분할이 된다.

**후속 owner:** [R18](../roadmap_v2.2-review/goals/R18.ko.md), [R19](../roadmap_v2.2-review/goals/R19.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S052](../evidence/SOURCES.ko.md#s052) · [S053](../evidence/SOURCES.ko.md#s053) · [S047](../evidence/SOURCES.ko.md#s047) · [S048](../evidence/SOURCES.ko.md#s048) · [S084](../evidence/SOURCES.ko.md#s084).

## F14 — 침식의 kill 억제와 정상 전투 진행은 함께 보존해야 한다

**관찰.** 최신 정책은 최종 보스 파편을 제외하고 실제 시전 시 현재 전장 적만 즉시 kill, 선택적 처치/사망 효과 억제, 필수 wave 진행을 요구한다. 이미 release된 projectile의 기존 수명은 유지한다.

**설계 판정.** R09는 큰 ordinary Damage가 아니라 special action·mandatory death/suppression·withdraw·result 비용을 같은 전환으로 묶는다. 전체 보스/chain 완성은 선행이 아니다.

**잘못 나누었을 때.** OnDeath/OnKill 폭주 또는 미래 required wave 소실/정지, 연출 actor의 잘못된 보상/승리 관여가 생긴다.

**후속 owner:** [R09](../roadmap_v2.2-review/goals/R09.ko.md), [R16](../roadmap_v2.2-review/goals/R16.ko.md), [R18](../roadmap_v2.2-review/goals/R18.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S046](../evidence/SOURCES.ko.md#s046) · [S022](../evidence/SOURCES.ko.md#s022) · [S025](../evidence/SOURCES.ko.md#s025) · [S026](../evidence/SOURCES.ko.md#s026) · [S053](../evidence/SOURCES.ko.md#s053) · [S075](../evidence/SOURCES.ko.md#s075).

## F15 — 침식에는 기존 정책인 Breakdown countdown의 실제 구현이 필요

**관찰.** apply_incapacitation은 trauma와 함께 physical damage도 증가시킨다. 현재 condition/psychology.advance_node는 collapse countdown을 소유하지 않는다. 룰북은 countdown·회복폐기·node완료·save/snapshot·일반사망 회수를 규정한다.

**설계 판정.** R09에서 trauma100 대가/attempt→result와 새 countdown·노드 회복/사망·asset회수·save/projection을 닫는다. 침식에 기존 incap 함수는 그대로 사용하지 않는다.

**잘못 나누었을 때.** 침식 때문에 신체손상까지 늘거나 trauma100을 곧바로 사망으로 만들거나 시한부가 영구 정지한다.

**후속 owner:** [R09](../roadmap_v2.2-review/goals/R09.ko.md), [R20](../roadmap_v2.2-review/goals/R20.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S059](../evidence/SOURCES.ko.md#s059) · [S060](../evidence/SOURCES.ko.md#s060) · [S061](../evidence/SOURCES.ko.md#s061) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S077](../evidence/SOURCES.ko.md#s077) · [S091](../evidence/SOURCES.ko.md#s091).

## F16 — Pause 중 cast admission과 침식의 실제 시전 시점은 구별

**관찰.** 룰북은 Pause 중 일반 command의 resource/skill cast 시작을 즉시 반영하고 pending resolve는 정지한다. 최신 침식 정책은 Pause input만으로 침식을 시작하지 않으며 일반 command 전체 지연을 요구하지 않는다. sim에는 pending start_seq와 ManualCastEnd 경로가 있다.

**설계 판정.** R09 기술 후보는 admission을 보존하고 resume 후 첫 적격 battle advance에서 release 이전의 once-only cast-start 결정을 연결하는 것이다. focus 시작/종료·공명 소비 의미는 exact owner에서 확정한다.

**잘못 나누었을 때.** UI 접수나 focus 종료를 임의로 실제 시전이라 정하면 발동 시점·자원/취소 의미가 바뀐다. 강제 연출은 모든 mutation/advance 경로를 함께 막아야 한다.

**후속 owner:** [R09](../roadmap_v2.2-review/goals/R09.ko.md). **확인 한계:** 시점의 기술 후보이며 승인된 hook 이름/정책이 아니다. P2G-20은 sim/manual admission·cast end·world/combat clock의 표를, 의미 충돌 시 P2G-10은 최소 질문을 소유한다. R01 선행 blocker는 아니다.

근거: [S018](../evidence/SOURCES.ko.md#s018) · [S044](../evidence/SOURCES.ko.md#s044) · [S046](../evidence/SOURCES.ko.md#s046) · [S077](../evidence/SOURCES.ko.md#s077).

## F17 — 지급 원자성은 공통 Executor 자체가 아니라 바깥 commit 경계

**관찰.** EconomyGrantState가 inventory/fragment/roster/UUID/enkephalin roots를 복제하고 prepare/validate 후 commit한다. GrantExecutor의 순차 mutation 자체를 독립 all-or-nothing으로 볼 수 없다.

**설계 판정.** 새 fragment/relic/resource를 관련 clone/validate/commit/preview/save roots에 포함시킨다.

**잘못 나누었을 때.** 여러 보상 중 후속 실패로 일부 자산·UUID만 남거나 preview가 실제 상태를 바꾼다.

**후속 owner:** [R05](../roadmap_v2.2-review/goals/R05.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R19](../roadmap_v2.2-review/goals/R19.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S035](../evidence/SOURCES.ko.md#s035) · [S036](../evidence/SOURCES.ko.md#s036) · [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038).

## F18 — 복수 파편·선택은 등록뿐 아니라 보상 plan/commit/validation 변경

**관찰.** PveEncounterRewardSource는 단일 fragment_id와 source/encounter/fragment uniqueness를 소유한다. 첫 보상 plan은 그 하나를 GrantSkillFragment하고 완료 branch를 재검사하여 commit한다.

**설계 판정.** R19은 해당 schema와 planner/commit/validators, normal grant와 save/projection을 동시 전환한다. 기본 grant는 R07/R14부터 존재해야 한다.

**잘못 나누었을 때.** 새 정의만 추가하고 실제 normal reward가 선택을 만들지 못하거나 완료/재지급 분기가 엇갈린다.

**후속 owner:** [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R19](../roadmap_v2.2-review/goals/R19.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S073](../evidence/SOURCES.ko.md#s073) · [S074](../evidence/SOURCES.ko.md#s074) · [S042](../evidence/SOURCES.ko.md#s042) · [S036](../evidence/SOURCES.ko.md#s036) · [S067](../evidence/SOURCES.ko.md#s067) · [S094](../evidence/SOURCES.ko.md#s094).

## F19 — 저장·복원은 최종 통합에서 처음 붙이는 기능이 아니다

**관찰.** durable checkpoint, battle-entry rollback, attempt roster, live-process reconnect의 상태 수명이 서로 다르다. 기존 A07/CA19도 새 상태 도입 시 저장을 동반하도록 요구한다.

**설계 판정.** 각 R이 자기 상태의 저장/복원/결과를 소유하고 R20은 복합 증거를 닫는다. 기존 의도를 구체 owner/atomic group으로 강화한다.

**잘못 나누었을 때.** 후퇴/RNG/ID/직원 대가가 다음 단계 전까지 잘못되는 분할을 독립 완료로 볼 수 없다.

**후속 owner:** [R02](../roadmap_v2.2-review/goals/R02.ko.md), [R05](../roadmap_v2.2-review/goals/R05.ko.md), [R07](../roadmap_v2.2-review/goals/R07.ko.md), [R09](../roadmap_v2.2-review/goals/R09.ko.md), [R14](../roadmap_v2.2-review/goals/R14.ko.md), [R20](../roadmap_v2.2-review/goals/R20.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S037](../evidence/SOURCES.ko.md#s037) · [S038](../evidence/SOURCES.ko.md#s038) · [S039](../evidence/SOURCES.ko.md#s039) · [S040](../evidence/SOURCES.ko.md#s040) · [S044](../evidence/SOURCES.ko.md#s044) · [S086](../evidence/SOURCES.ko.md#s086).

## F20 — 대표 RON과 전체 표현력 기준선은 다른 완료 주장

**관찰.** 기존 CA00는 세 원문 및 공통정책의 요구/COV와 boss별 행을 만든 조사 기준선이다. A08은 제안 문법/시험값이며 actual fixture가 아니다.

**설계 판정.** R21은 C01–C07의 실제 외부 RON 실행과 전체 COV의 지원/미지원/정책대기 대조를 따로 제출한다. R22는 다른 작성자의 data-only holdout이다.

**잘못 나누었을 때.** 몇 개 이름 같은 RON 또는 privatebuilder 통과가 전체 독립 제작/표현력 완료로 둔갑한다.

**후속 owner:** [R00](../roadmap_v2.2-review/goals/R00.ko.md), [R21](../roadmap_v2.2-review/goals/R21.ko.md), [R22](../roadmap_v2.2-review/goals/R22.ko.md), [R24](../roadmap_v2.2-review/goals/R24.ko.md). **확인 한계:** 이번에 354/127 모든 행과 각 콘텐츠 원문을 새로 연속 재검증하지 않았다. baseline 출처/조직을 인수하고 24CA owner를 새 R에 매핑했다.

근거: [S005](../evidence/SOURCES.ko.md#s005) · [S006](../evidence/SOURCES.ko.md#s006) · [S054](../evidence/SOURCES.ko.md#s054) · [S057](../evidence/SOURCES.ko.md#s057) · [S058](../evidence/SOURCES.ko.md#s058) · [S078](../evidence/SOURCES.ko.md#s078) · [S087](../evidence/SOURCES.ko.md#s087) · [S096](../evidence/SOURCES.ko.md#s096).

## F21 — 인접 저장 계약 문구 불일치를 최소 Goal에 과잉 전가하지 않는다

**관찰.** data_loading_contract의 durable 단락은 Core payload V4를 서술하지만 actual active_run은 V5 writer와 V4/V5 reader다. 기존 P2G-50 검토도 이 불일치를 기록했다.

**설계 판정.** 해당 저장/document owner의 인접 정합성 문제로 남긴다. 현재 최소 Goal은 save 비변경·index 비영구성이므로 그 자리에서 모든 reader 제거를 요구하지 않는다.

**잘못 나누었을 때.** 인접 오래된 문구만으로 새로운 호환 정책이나 전체 최소 Goal 재설계를 대신 결정하게 된다.

**후속 owner:** [R01](../roadmap_v2.2-review/goals/R01.ko.md), [R02](../roadmap_v2.2-review/goals/R02.ko.md), [R20](../roadmap_v2.2-review/goals/R20.ko.md). **확인 한계:** 정적 본문 확인이며 실행 검증은 하지 않았다.

근거: [S039](../evidence/SOURCES.ko.md#s039) · [S065](../evidence/SOURCES.ko.md#s065) · [S092](../evidence/SOURCES.ko.md#s092) · [S094](../evidence/SOURCES.ko.md#s094).

## 기존 성과·검증의 재사용

CA00의 요구/COV 기준선, CA01의 Working Policy·Review·Freeze·Phase Plan·검증 Receipt·Goal Set·Handoff는 새로 만들지 않는다. 기존 기록의 표준 검사 결과와 감사 PASS는 그때의 근거로만 보존한다. 실제 source의 변경 영향이 있으면 해당 판단·시험만 다시 확인해야 하며, 단순히 세션이 바뀌었다고 전체 P2G/GEX를 반복하지 않는다.

현재 최소 Goal이 독립 유효한 이유를 조사했지만, 새 로드맵의 나머지 R들이 정식 Plan/Goal로 작성되거나 구현 승인됐다는 의미는 아니다. 후속 source/계약을 바꿀 때 가장 이른 owner의 정상 절차를 거쳐야 한다.

## 이번에 완료하지 않은 범위

전체 COV/BOSS-COV 행 및 세 콘텐츠 원문을 이번에 다시 전수 재검증하지 않았다. 정밀 line-crossing geometry/성능, 모든 original CA markdown 카드의 별도 재독, 모든 old audit의 fresh 실행/해시 확인, 게임/부하/제3자 제작/독립 감사도 하지 않았다. 원문 승인/미정과 현재 실행 지원을 혼동하지 않고 해당 제한을 따로 표기했다. 이 때문에 본 배포본은 **코드 기반 개정 권고**이지 “전체 콘텐츠 제작 기반이 완성됐다”는 인증이 아니다.

그 제한이 CA00/01 인수를 무효화하거나 모든 의존성 판정을 불가능하게 만들지는 않는다. 기존 49개 관계·24 CA·9 M의 disposition과 새 단위별 계약/검증 owner는 실제 개정 파일에 모두 제공했다.

## 요청의 마지막 두 질문에 대한 답

**1. 어떤 순서로, 어디까지 함께 바꾸어야 하는가?** 위의 R00→R24 순서를 기본으로 삼되, 각 단계가 소비하는 최소 계약을 구별한다. 핵심 동시 범위는 metadata/실행 연결, 세대/new-run/save, 사본/장착/지급/save, 개인 plan/지연 소비자, 침식/death bookkeeping/직원 비용/clock/result, 복수 보상 plan/commit이다. 각각의 저장·거부·종료 경로는 나중 단계가 없어도 유효해야 한다.

**2. GOAL-CAMEC-01을 첫 구현으로 유지해도 되는가?** **A 권고: 유지해도 된다.** 이번 실제 source 대조에서 먼저 수정해야 할 필수 Plan 오류는 발견하지 않았다. 정확한 영향과 보존할 artifact는 별도 영향 보고서에 있다. 이 답은 실행 승인이 아니며 NEXT-0009는 실행하지 않았다.
