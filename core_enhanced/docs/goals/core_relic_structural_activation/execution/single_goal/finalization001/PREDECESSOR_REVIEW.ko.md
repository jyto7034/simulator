# U09 선행 영향 판단 — 제품 경로

Root Astra의 코드 검토·범위 판단이다. 최종 독립 감사가 아니며 아직 실행 중인 검사나 성능 검사를 통과로 간주하지 않는다. 실제 행 연결과 명령/원시는 같은 디렉터리의 후속 selection/impact/coverage가 소유한다. 복합 조건은 비영향 부분의 선행 증거와 영향 부분의 현재 증거를 함께 연결한다.

## 근거와 적용 방법

기존 [PLAN](../PLAN.md)의 A–G 실제 제품 검토·교정 원인·시험 범위 판단을 재사용한다. 추가로 현재 소스를 U09 baseline의 exact438 목록과 대조했고, 변경된 공통 경로의 아래 실제 diff/분기를 확인했다. 비교 원본은 baseline/PREPARATION_ENTRY_001.json의 source/files 및 그 기록이 지시하는 U08 source archive `4c7bbd317a9a12045267bd306057e771133096ccbccace6a51f484d23a46e2ac`다. 현재 소스가 이 과거 해시와 같다고 주장하지 않는다.

원 요구는 U08 PREDECESSOR_IMPACT_013의191 conditions·31 r06_acceptances와 COVERAGE_013의32 U08 문장이다. ID/리터럴/원 owner를 보존하되 현재 사용자의 영향 기반 검증 지시에 따라 과거의 일괄 전체 실행 절차를 반복하지 않는다. 원래 검사 결과·실패·감사 원본은 수정하지 않는다. 이번69개 통합 선택, U09의 실제 기능 증거와 PC/R05는 아래 영향 경로에 필요한 현재 증거다. 이전 시험 본문이 같다는 사실만으로 재사용하지 않는다.

## 실제 공통 경계

- `game/resource/mod.rs`의 제품 차이는 ID 문자열 capacity 조회 추가뿐이다. resource store의 balance/lease/owner 검증·revision·연산 의미는 바뀌지 않았다. 새 structural branch의 독립 Delivery/Chain 보유·회수는 별도 TC/D/RV 검증 대상이다.
- `game/value_expr/mod.rs`는 delivery_capabilities의 crate export만 추가했다. 타입 compiler/evaluator·rounding·grouping·stat/growth 연산을 교체하지 않았다. catalog의 기존 compile_skills는 새 공통 함수에 has_deadline=false/budget=None을 전달한다. 새 PersonalShieldGrant 및 effective compile의 별도 budget/deadline은 명시 분기다. R06 pure 산술/단위/정적 ref와 새 runtime source/amount/결과 경로를 구분한다.
- `skill_data.rs`의 기존 validate_skill_contracts는 같은 skills를 references로 전달하고 single_recipe_sites는 빈 배열이다. 기존 recharge/parry/Goodbye 참조의 원 catalog 검증 의미를 보존한다. effective skill만 별도 원 catalog와 recipe site 문맥을 받는다. skill_fragment_data의 차이는 문자열 capacity 조회 추가이며 form/recipe ledger를 바꾸지 않는다.
- `application.rs`는 RequestRelicEffectivePlan/RelicEffectivePlan enum과 결과 분류 추가다. 기존 host Rust 소스는 baseline438 목록과의 비교에서 변경되지 않았다. Host 저장 writer·intent/revision·recovery codec·auth·WebSocket·registry를 새로 작성하지 않았다. 그러나 host가 소비하는 Core의 clone/withdraw/fault/owner는 바뀌어 현재 서버4검사를 수행한다.
- `world/application.rs`는 기존 staged operation 이후 특정 loadout/Continue/복구 route에서 structural preflight를 추가한다. `validate_structural_loadouts`와 `capture_structural_battle_owner`는 장착 유물이 없으면 즉시 반환하며, 장착이 있으면 roster를 순회한 뒤 해당 직원의 structural rules 유무로 분기한다. 따라서 numeric-only 구성의 gameplay/저장 의미 보존을 작업량 완전 동일 주장으로 확대하지 않는다. 비어 있지 않은 owner·입장·복구·실패 비용은 현재 U08 owner/carrier 검사 및 U09 admission 증거로 연결한다.
- `damage_runtime.rs`의 regular occurrence 분리는 현재 regular_aura_count가0이면 기존 aura_basic_bonus 경로다. capture_regular_support도0일 때 None으로 반환하고 기존 clamp 산술을 유지한다. 새 RegularOnly가 있을 때만 checked 범위·선택된 contributor 캡처를 사용한다. 기존 aura/Triggered와 새 regular 지원의 경계는 현재 R11/U08 carrier/RV의 실제 공격으로 검증한다.
- `ability/flow.rs`의 기존 Flow는 빈 side_branches를 생성한다. 기존 Compare와 새 EffectCompare는 다른 분기다. result_scope와 side owner의 저장·clone 비용은 새 실제 비용으로 검토한다. D의 cancel_structural_side_cast_for_withdrawal은 side branch가 없으면 false를 반환하여 원 direct-removal path를 유지한다. 새 side branch가 있는 경우의 방출 payload·settlement·취소는 TC/RV의 현재 증거다. 공유 원 Flow는 현재 R10군으로 검증한다.
- runtime source의 effective/regular support는 optional 강한 binding으로 추가됐다. 기존 numeric/context의 identity를 대체하지 않는다. amount_query G는 같은 불변 frame을 structural→numeric 순서로 재평가한다. 기존 numeric-only·Snapshot/Live·방출 뒤 source 소실·query 불변은 현재 carrier와 legacy064/G056/CLI066으로 확인한다. 새 필드의 layout/retained 비용이 없다고 단정하지 않는다.
- safety 설정 parser/cache/ledger·배포 기본값·native ceiling은 그대로다. 명시 authoring 개발 파일만 추가되어 같은 측정 profile의 effective identity를 실제 CLI에서 확인했다. 현재 profile3검사 및 F의 boundary/cancel/retry를 연결한다. exporter·Grafana·bounded registry·collector·heap 관측 구현의 비영향 증거는 재사용한다. 신규 owner의 actual cost/회수와 관측 projection은 해당 현재 증거가 필요하다.

## 원 조건군별 판단

| 원 조건군 | 재사용할 비영향 범위 | 현재 영향 범위·필요 증거 |
| --- | --- | --- |
| A08, X01 | boss actor 선택/방향/state route·autonomous authority, 원 validation의 비신규 opcode 분기 | 공유 Flow wait/settlement·source/취소·clone은 R10/RV/TC 및 PC/R05. 새 structural 지원을 기존 보스 고유 구현으로 확대하지 않음 |
| A09, X04 | 침식 시작 predicate·적 suppression·강제정지 clock·Breakdown/결과 정책과 해당 코드 | 공통 피해/보호·source 종료/rollback만 현재 R12·RV·World/host에 연결. R09 전용 branch를 재작성하지 않았다는 근거와 구별 |
| A10, X06 | 유한 Flow의 기존 정적 지원 목록/기존 오류 의미 | 전체 R10 선택8, 구조 occurrence/side branch TC/RV와 부족/취소/반복·withdraw 검증. 새 effect-local 결과를 기존 action aggregate와 혼동하지 않음 |
| A11, X02 | 기존 modifier/base·overlay 작성 schema·subscriber/재입력 원 권한 | R11 선택9, U08 carrier12, RV support63/71 및 보강하는 fixed HP-cost/Pause/stop/CC. LegacyBasic/RegularOnly의 의도된 구분은 새 U09 정책으로 명시 |
| A12, X03 | slow/root/taunt/pull·parry 전용 predicate·floor/death-prevent의 변경 없는 분기 | shared/personal 순서·mitigation·HP loss·HealthCost·회수 R12 선택과 World, RV personal 및 Monk. 새 personal pool·source_creation을 기존 shared recharge로 대체하지 않음 |
| IVA-CAVER01–04/07–12/16의 pure kernel·정적 부분, R06 E01/E03/E04/E06의 같은 부분 | 산술·unit·rounding·집계·equipment stat·growth·resource ID/slot binding·기존 type/codegen | runtime capture/admission/basic speed와 결합되는 문장은 현 source/regular/PC 경로를 함께 연결. catalog wrapper false/None과 unchanged core evaluator가 pure 재사용 근거 |
| IVA-CAVER05–06/13–14/17–20, R06 E02/E05/E07/E08 및 E09의 actual 경로 | 원 malformed syntax, unsupported opcode, save envelope 자체의 동일 구현 | Snapshot/Live exact owner·settled occurrence·rollback·금액 조회·clone/admission은 U08carrier, TC/RV, legacy064/G056/CLI066, F boundary 및 profile3/PC로 연결 |
| F01–F06/F09–F10의 asset/recipe/codec 부분 | 파편 UUID·재료/가루·개화 unlock·사본 역결속·checksum/형식·grant 원장 | World preflight가 끼는 form/equip/transfer/restore는 World/U08owner/RV invalid generation·form/current loadout; 불변 ledger와 변경된 호출 경로를 분리 |
| F07–F08/F11–F12 | 파편 공개 schema·관리 비용의 비영향 분기 | 원 copy/form/source는 U08carrier 및 U09 public normal reward/equip/query/cast/fresh Continue. 新 structural parser/recipe의 실패는 TA/TB/F/RV |
| PE-CRSO01–07, PE-CMRP01–03/06 | 설정의 신뢰 경계·parser 상한·기존 default/native/development-retained 구조·history window | 이번 explicit authoring 값/identity·publication 예산·기존 profile 혼합 거절은 preset CLI와 profile3/F/RV. original10m 실패를 상향 profile PASS로 소급 변경하지 않음 |
| PE-CRSO08–11, PE-CMRP04–05/07–08 | exporter/registry/heap observer/RSS sampler·host session ownership·profile전송·drop/회수 방법 | 새 Core owner usage는 현재 observation1, RV trace/amount 모드, actual cost/회수/PC/R05. RSS와 admitted extent를 구분 |
| PE-CRFI01–11 | 기존 최초 fault 보관·gate/authority·postcommit transport/observer·다른 player isolation 코드 | Core source/clone/withdraw가 유발하는 실제 오류와 재접속/저장 보호는 현재 host tick/manual4 및 RV rollback. 기존 fault routing의 폭을 새 검사 몇 개가 모두 재실행했다고 쓰지 않음 |
| PE-CRRP01–15, X05 | 결과·보상·후퇴/load횟수·file intent·revision 경쟁·codec/미지원 원본 보호의 구현 | 현재 Continue/SavePoint/retreat에 추가된 structural 재구성은 RV World/normal public freshContinue·generation 원자거부와 U08owner/host. 새 battle plan을 durable authority로 저장하지 않음 |
| PE-CRSL01–14 | 기본 공명 정책·시계·silence/hardCC/recharge·기존 형식과 일반 target rules | E의 selected usefulness·접수/방출 owner·직원소실/withdraw/sideflow는 target5, R10/R11/RV, host paused reconnect. 원 공명/접수 모델 재구현 없음 |
| PE-CBST01–20 | TP·배치상한·이동/교대/작업/Rest·막기 geometry·settings playback의 별도 정책 구현 | 새 personal/고정 지원의 선택 reach·recipient identity·공통 target·source withdrawal과 비용/clone 경로는 target/World/RV/host. 기존 물리 이동/타일 algorithm을 구조 recipe가 전역 치환하지 않음 |
| PE-CIOB01–18 | object registry/배치·pool분리·진화·TTL·wave/보상·walls·출처 lifetime 정책과 실체 코드 | 공통 source/피해/보호·clone에 의존하는 부분은 current shared consumers/PC/R05와 결합. structural Grant의 closed union에는 CreateObject가 없어 새 독립 객체 기능으로 확대하지 않음; 기존 parent/object 분리 의미 유지 |
| PE-CRNV01–04/09–13의 asset/economy/codec 부분 | 유물 UUID·슬롯 소유·촉매/가루·Event 권리·회수·저장 원장 | relic schema6/current U08owner5·실제 Corrupt80/30·World/public restore로 변경 preflight/owner를 검증. 새 rules가 없더라도 추가 scan 비용은 무시하지 않음 |
| PE-CRNV05–08/14 | 기존 numeric selector/flat/percent·immutable definition·U08 unknown-effect 진단 | current carrier12·G/legacy64·F publication反례·RV samecopy/form/variant/public consumer. U09 rules의 유한 closure 추가는 opt-in 지원 확장, legacy effects의 조용한 fallback이 아님 |
| PE-CRNV15, X08 및 각 군 budget/clone 조건 | unchanged ledger/engine ceiling·기존 hard error 분류 | 새 owner/임시compiler/cache/source의 actual extent·누적 work·one-short/cancel/retry는 F/TB/RV/current integration/PC/R05. exact 과거 비용과 다르면 원 실패로 남기고 원인 분류 |
| X07 | 원 readonly·trace/amount의 독립 관측 설정 계약 | G056/CLI066·legacy064·RV mode parity와 observation1. amount 관측을 켠 비용을 optional trace 비용으로 오인하지 않음 |
| X09–10, IVA-CAVER21, PE-CRSO12/CRFI12/CRSL14/CBST20/CIOB19–20/CMRP09–10/CRNV16 | 선행 완료 자체와 과거 원본/감사의 재사용 | 이번 영향 기반 검증·raw 수집·선정/coverage·복원/freeze·비참여 Goal 전체 감사는 현재 완료 경계에서 새로 닫아야 함. 과거 audit로 U09 완료를 대신하지 않음 |

## 남은 판정

69개 통합은019의Core65와020의서버4로 통과했고 Root가 원시 결과를 확인했다. PE06.1의 동일 SkillId 두 직원은022의rv006에서 실제Normal30+75/Corrupt40, 즉시·지연, Pause 접수, 직원별105/40 피해·source copy·query·공유 원형 불변으로 통과했다.022 전체는 다른 두 실패 때문에 FAIL이며 이 한 행만 재사용한다.

추가 PE11.2의 처치 사례에서 실제 제품 누락을 발견해023으로 교정했다. `sim.rs`의SkillStep가 구조 유물의 독립된 후속 Self 단계에도 이미 죽은 원래CastTarget의 생존을 요구했다. Root는 원래cast도`use_control=false`로 생성될 수 있음을 확인했으며, 이 bool의 의미를 넓히지 않고 기존`is_side_occurrence`로 현재 소유된 구조 후속 단계만 원래 타겟 검사에서 분리하는 diff를 검토했다. 선행 source 조회의settlement_only 거부·실제 유한 lease, cast 취소 gate, 단계 자체 target 해석과 사망/철수/CC 취소는 그대로다. 구조 side가 없는 기존 Flow는 lookup 계량 전 반환하므로 원래 타겟 검사와 비용 경로를 유지한다.

024는 실패한rv005 두 개와 이 변경의 원래 target/지연CC·구조 side 취소/resource 회귀7개를 묶어9PASS였다. Root는 전체raw를 읽었다. lethal 실제HpLost10 후Self회복37 한 번, empty/invulnerable에서회복0, nonlethal회복1을 확인했다. 소비1ms tick의 최소nativework326456·한도326455에서`application_deployed_context` fault 후 접수 상태 전체 복구·exact retry도 확인했다. 이는 전체tick 경계이며 lookup 단독 비용이나 별도의 authoring 최소값 측정이 아니다. Fixed channel의Self HPcost10·Pause·정상ReinputSustainedSkill·실제Stun·부족 비용 rollback도 통과했다. Flow/resonance/budget와 공통 failure 소비 경로의 현재 증거에024를 추가하며, 기존69 또는 다른 모든 검사를 다시 실행했다는 주장은 하지 않는다.

PC의 현재36조건/576행은31성공·5typed refusal를 유지했고, Root가 actual probe 종료와 두 comparator 원시를 확인했다. 직전 U07M의 시간 외 비용/동작은 delta0이며17개 음성 대조군이 거절됐다. source136/object152/placement48과 기존 U06 대비200바이트 accounting 차이는 기존 그대로다. 새 구조 조합 자체의 무비용 증거로 확대하지 않는다.

R05 현재002는 원 exit101을 보존한 `warning_concurrent_peak_only`다. event-heavy ops44206/retained266102/peak271927는 원 cap 내이고, concurrent ops370928/retained4196928은 직전 U08과 같다. concurrent peak9709348은 원 한도9030188보다679160바이트 높고 직전 U08의9683689보다25659바이트 높다. 다른 적용 hard guard 실패는 없으며 parity/repeatability는 유지된다. 원래 allocation-observer 모드에서는 latency gate가 비적용이지만, 관측 concurrent p9510014462ns는10ms cap 초과/false임을 그대로 남긴다. event-heavy의 직전 대비+12 allocation operations/+256 retained/+512 peak는 실제 증가이며 원인 입증 없이 무비용으로 표현하지 않는다. 기존 warning 허용 범주·원 cap·실패 원본을 유지했고 새 예외나 한도 변경은 없다.

현재 비용 증거를 이 범위로 인수하며 최종 target 복원과 비참여 감사는 남아 있다. 이 문서는 기대 결과를 관측 결과로 대체하지 않는다. 새 제품 수정이 생기면 해당 분류만 재검토하며, 미확인 상호작용이나 비용 차이를 일반적 “기존 테스트 통과”로 덮지 않는다. 최종 독립 감사가 요구한 구체 누락은 담당자가 보완한다.
