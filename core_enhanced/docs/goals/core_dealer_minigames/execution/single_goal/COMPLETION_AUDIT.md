# U16-3 독립 완료 감사 — independent001

판정: **Complete / High**. 확인된 범위 내 제품 결함 0, 열린 완료 차단 0. 이 판정은 아래 exact target에 귀속한다.

감사자는 `/root/u163_independent_audit`이며 U16-3 설계·구현·시험 교정·최종화에 참여하지 않았다. 사용자의 “U16-3 착수 및 완수 후 서브에이전트를 소환하여 독립 감사”와 “현재 명령 우선”을 GEX-20 직접 위임으로 인수했다. 추가 에이전트를 소환하지 않았다. GEX Reader Contract·§5 currentness·§6.4/6.7·GEX-20, 연속 실행 지시서와 로드맵 §1.9를 적용했다. source/시험/fixture/설정/기존 evidence/동결 target을 수정하지 않았고 이 문서와 `audit/independent001/`만 작성했다.

## Exact 대상과 현재성

GS-CDMG-001 / GOAL-CDMG-01 / PHASE-CDMG-01. HA-CDMG-001, FPF-CDMG-001, PP-CDMG-001, Goal Quality Contract v2와 [Goal 원문](../../P2G_50_GOAL_SET.ko.md), [판본·수치 계약](../../RULES.ko.md)을 직접 읽었다. 5 policy/decision 영역, 21수용, 10차원, 5IVA, 실행노드 AG-CDMG-RUN·WP-CDMG-VERIFY 전체가 범위다. single_phase이므로 별도 GEX-30에 남긴 조건이 없다.

[Audit Target Record](target/u16_3_current_001/AUDIT_TARGET_RECORD.json): `u16_3_current_001`, HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`, SOURCE814/EVIDENCE639/DELTA251. SOURCE.tar.gz SHA256 `e1710de6d91b62cdd70d4ba4fd04e491d2d421b0b51786aa20609f5a4ce53351`.

[입구](audit/independent001/CURRENTNESS_ENTRY.json)와 [출구](audit/independent001/CURRENTNESS_EXIT.json)에서 두 manifest의 **모든 실제 파일** 크기·SHA256, manifest/archive/restore receipt hash, archive 전체 member와 HEAD를 대조했다. source roots와 current Goal/test_parts의 신규·누락, source 외 실행 evidence 목록도 확인했다. 판정은 **current / unchanged**이며 불일치·누락 0이다. 추가로 bound된 Cargo/config/외부 도구는 source manifest에 열거되며 누락을 숨기는 임의 범위 축소가 없다. mutable HANDOFF/현황/GEX10 종료 기록과 이번 감사 산출물은 명시된 제외다.

[독립 검산](audit/independent001/EVIDENCE_RECHECK.json)은 BEFORE585/SOURCE814의 정확한251개 add/replace/delete와 모든 pre/post hash, 실제 제품·시험·데이터·문서27개 delta를 확인한다. 실제 수정 전 BASELINE527은 U16-2 고정 archive의 해당527 member와 전부 byte 동일하다. 추가58개 BEFORE dependency도 원본 archive에 연결된 불변 도구다. [격리 정역복원](target/u16_3_current_001/RESTORE.json)의 full forward814/reverse585/operations251을 이 동일성에 근거해 재사용했다. 사용자 tree에 patch를 적용하거나 원본 최종화 도구를 재실행하지 않았다. 기존 dirty의 변경을 이번 성과로 귀속하거나 되돌리지 않았다.

## 수용 조건별 독립 판정

아래 소스는 `core_enhanced/src/game/` 기준이다. 시험 기대값과 실제 호출 및 원시 출력을 직접 대조했으며 COVERAGE의 satisfied 표시는 판정 근거로 대신하지 않았다. 시험명은 `dealer_games_tests.rs`, `world/tests/u163_dealer.rs`의 실제 함수와 focused007/core005 원시 selector에 대응한다.

| 조건 | 직접 확인한 구현·기대·원시 결과 | 판정 |
| --- | --- | --- |
| A01 | `facility_rooms`가 authored Gambling 지원을 결정하고 `map_content→resolve_dealer_reward→start_dealer_dialogue`가 정상 Reward/VN으로 진입한다. blocking VN은 직접 Accept 우회를 거절한다. offer/decline/repeat와 새 원정 초기화 시험, host decline/repeat의 노드소비·설명이력 대조. | Pass |
| A02 | `DealerSession::new`에서 기본30과 조우 배율을 한번 생성하고 첫 ChooseGame 뒤 Gambit VN을 거친다. `finish_round`만 게임delta에 배율을 곱한다. cancel→다른 게임에도 선택 유지, raw host30−10×3=0과 대출원금20 비증폭 확인. | Pass |
| A03 | ChooseGame/StartRound/Continue의 양수 조건, signed checked 잔액과 음수 보존. host 전체30판돈 손실 후−60, Continue 거절, 정산/퇴장 가능. 비용 workload10판과 게임변경 시험은 판 이력 누적 없이 반복한다. | Pass |
| A04 | Ready Cancel은 자산·포인트 불변, Playing Forfeit은 현재 유효판돈 손실1회. revision을 clone/transaction 전에 대조하고 성공에만 증가. 오래된 StartRound/Leave와 중복 Forfeit의 roots 불변 assertion 및 raw rejection 확인. | Pass |
| A05 | `BlackjackRound`의52장, Ace1/11, S17, natural3:2 floor·일반1:1·push0, player hit/stand/최초2장double, NPC 각자 대딜러 정산 검토. 손계산 Ace/natural/dealer-natural/push/bust/odd3→4/S17/double 기대와 원시 통과, 정상host double−2×3=−6 확인. | Pass |
| A06 | `dice_rank`의8서열·multiplicity/face/kicker·straight상단, 5주사위/최대3회/hold 재선택/early stand/NPC 잔여행동과 공동최고0을 직접 검토. 순위·동률·hold·3회 시험과 정상 public 3게임 경로 통과. | Pass |
| A07 | `FlipShoe`의94장과 `FlipRound::pump`의 duplicate/SecondChance 즉시소모·추가전달, Freeze, FlipThree 지연처리/버스트폐기, 활성대상, 7숫자 즉시끝을 확인. 특수 fixture는 순수 규칙 oracle이며 정상 command 경로도 별도로 존재한다. 40판 shoe94장 보존과 지연action/7즉시종료의 수작업 시험 통과. | Pass |
| A08 | 숫자합×2 뒤 가산과+15, 비버스트만 공동1위 계산, bonus/ties 정수내림 후 게임delta에 갬빗. 36×2+10=82,49×2+10+15=123,3명동률5+20/3=11 등 독립 산술과 시험 대조. 실제host Flip7+10×3=+30 및 완료Continue. | Pass |
| A09 | view는 세 참여자 공개패·점수·상태와 적격action을 생성하며 BJ dealer hole·shoe·seed/cursor는 포함하지 않는다. BJ threshold는 자기패, dice_hold/rank는 자기 dice, Flip npc_target은 public tableau만 소비한다. hole 바꿔도 진행중 view 동일 시험과 public-view-derived 실제host 계획 확인. | Pass |
| A10 | `dealer_copy_selection`은 길이/중복/existence/equipped 검사, 사본별 공개원금·합계·지급후양수 preview. 동일·없는·장착사본/0잔액 확정 거절과 취소불변 시험. host 다중선택과 raw preview/확정 관계 대조. | Pass |
| A11 | ConfirmCollateral이 EconomyGrantState의 live copy를 escrow로 옮기고 포인트와 같은 transaction에서 확정한다. live inventory에서 사라져 장착/침식/합성/분쇄·사용 경로의 사본조회로 소비할 수 없다. 장착/침식/Claim/Exit/Complete 우회 거절 시험과 host 두사본 담보/67동시담보 raw 확인. | Pass |
| A12 | Settle 뒤 Redemption/Shop에서 게임·새대출 복귀 arm 없음. selected_principal/points_after_selection/forfeited_copies를 owner가 생성한다. 부분회수·전부포기, 부족 시 선택·자산 불변, host foreclose 거절 후 빈선택확정 확인. | Pass |
| A13 | `return_dealer_escrow`가 UUID/definition/variant 원본을 반환하고 포기는 escrow owner만 폐기한다. Corrupt 원본 동일성·다른 사본 제거·전액원금10·다른rarity17원금 시험. 이번 정상host에서도 원래 Corrupt사본 완전동일과 포기사본 부재 직접 재현. | Pass |
| A14 | authored offers를 entry에서 한번 복사하고 가격/재고/유물definition/Normal상태를 게임 전 공개한다. 재추첨 arm 없음. 공식layout offers=[] 유지, 외부pack4상품과 확장68상품 구별. bundle/common/rare/special의 실제 authored 효과 정의와 source 검토. | Pass |
| A15 | Shop에서 잔액·재고 검사→공통 ensure_relic_grant/EconomyGrantState grant→가격/재고차감, 전체가 transaction. bundle+cheap+cheap 후1점, 품절/부족/stale 무변경 시험. 이번 actualIO 실패 재시도에도6사본·원본/포기관계 정확하고 중복Leave 거절. | Pass |
| A16 | 원금·상환·가격·기본지급에 multiplier를 쓰지 않는다. 음수 Shop에서도 offers/leave, Leave만 잔여0/노드완료. host negative_complete가−60을 다른 자산추심 없이 소비하고 원래 inventory 유지. | Pass |
| A17 | `live_ron→DealerPolicy::validate`의 유효참조·고유offer·양수·가중합계 overflow·terminal VN choice 계약을 검토. 실제 package valid1/reject14의 changed_input과 stderr/exit를 대조. 잘못된 policy를 silent default로 변경하지 않는다. | Pass |
| A18 | same-process 조회는 현재 session을 보존한다. 원래 entry checkpoint/has_active_relic_history가 새 dealer owner를 포함한다. incomplete offer/Gambit/loan/정산·구매 freshContinue를 raw 원본과 비교했으며 최초 dealer view/난수 재입장 동일성과 자산 전체 rewind 시험 통과. | Pass |
| A19 | 완료 후 RewardSession을 제거하고 original UUID 회수/포기/구매/노드·설명이력을 기존 checkpoint에 발행한다. raw completedContinue/SavePoint의 전체 자산과 Run을 비교. 이번2process 거래완료→freshContinue에서도 Run/inventory 정확히 동일, 재지급 없음. | Pass |
| A20 | 실제 paired host가 profile journal 임시파일 위치에 디렉터리를 만들어 OS21 실패를 일으킨다. 실행 전 자격을 만족한 Leave/Decline/LoadCheckpoint에서 memory/active/profile bytes 불변과 같은명령 재시도 성공. 기존4회 원시와 이번1회 직접 재현. Core2213/통합72/기존native4·server31의 영향근거 별도 검토. | Pass |
| A21 | 정상4상품/3초기사본, 확장68상품/67초기사본,67동시담보·1회수/66포기의 고정10판64명령을 9process/1152창으로 측정했다. allocator/RSS와 내부work를 분리하고9회 정확회수 검산. 기본10M/16MiB 불변, source/복원/선행17수용 continuity와 구현비참여 감사 완료. | Pass |

## 차원·통합·구조와 품질

10차원 모두 applicable·Pass이며 N/A는 없다. success=A01–19의 전체 공개경로, invalid_input=A04/10/15/17의 잘못된 단계·게임·대상·사본·revision·RON, failure_rollback=A10–15/20의 부족·자산·실제IO, retry_idempotency=A04/13/15/18–20의 중복·같은명령재시도·복원, ordering_concurrency=현재 직렬 command revision과 paired publication의 원자성이다. 병렬 gameplay 지원을 새로 주장하지 않는다. boundary_empty_max는 공식 빈상품/0·음수잔액/전액판돈/가중합계/자연21/dice tie/Flip7 경계, disconnect_restart_persistence는 same-process·fresh 미완료/완료·SavePoint, live_data는 정상 RON packaging과14작성거절, producer_consumer는 validated data→CoreApplication/DTO→실제host, legacy_fallback_absence는 기존 일반 Reward·U16-2 활동·빈공급 보존과 대체지급/재추첨 부재에 대응한다.

5IVA FLOW/GAME/LOAN/SHOP/STORE는 위 수용 표의 실제 작성→명령→조회→transaction→host→복원 seam을 공유하며, AG-CDMG-RUN의 동시구현 SCC와 WP-CDMG-VERIFY의 durability/비용/보관 책임이 실제 결과에 존재한다. 무관한 후속 Phase나 UI로 미룬 조건은 없다.

Implementation Quality Guardrails를27개 실제 delta와 현재 production 호출에서 판정했다. fixture/test 이름에 따른 제품분기, silent invalid 보정·catch-all 성공, consumer의 도메인 재계산, 미래전용 abstraction/registry, 임의 timestamp/retry 완화, assertion 삭제/신규ignore, 중복자산권한과 미승인 fallback을 발견하지 못했다. projection55→56은 기존55 유지+DealerState 하나 추가이며 실제 transport contract를 검증한다. 다른 authored rarity 원금교정은 “모든 미장착사본”을 유지하고 authored 기본원금을 공개한다. 준비경로 교정은 노드 자격·10battle clocks·같은 거래를 유지하며 제품한도를 높이지 않았다.

상태의 단일 소유를 검토했다. immutable DealerPolicy는 validated layout, 설명이력은 RunProgression, 현재판/points/revision/escrow/stock은 optional boxed RewardSession, live asset은 기존 RelicInventory/GrantState, 저장은 기존 paired store다. 사본 반환 helper는 원본을 돌려줄 뿐 새 asset engine이 아니다. clone_shape·extent/preflight·active relic history가 dealer owner를 함께 계산하며 일회65536 workspace와 유한 카드/task 처리 및 checked 돈 계산이 있다. 현재판·마지막결과만 보존하고 누적판로그를 추가하지 않는다. None/false serde default·skip은 이전 save/pack의 정규화 계약을 보존한다. 새 unsupported-save 대체경로나 별도 store는 필요하지 않았다. runtime/data-loading 문서도 이 계약과 일치한다.

## 새 실행·재사용과 실패 이력

- 새 실행: [REPLAY_RESULT](audit/independent001/REPLAY_RESULT.json), 정상 frozen host/admission과 보관된 actual preimage를 새 `/tmp/u163-independent-*` 사본에 복원했다. **2fresh process/20commands/실제OS21 1회 및 동일명령재시도**, 두 실행 exit0. 전체19명령 trace가 기존 raw와 동일하고 독립산술30−30=0→담보20→상환10→5+2+2구매후1, Corrupt원본/포기사본/6사본, freshContinue Run/inventory를 직접 assert했다. 재현 argv·stdout/stderr/명령·IO_FAILURE 원문은 감사폴더에 있다. 원본 save/issuer는 변경하지 않았다.
- 새 read-only 검산: currentness.py와 recheck.py로 모든 고정파일, archives/delta, 원시 시험 selector, host request/trace/reject/IO, 주요 미완료·완료·SavePoint 상태, authoring exit와9개 비용 원시를 직접 검산했다. 기능시험 재실행과 구별한다.
- 재사용: core005 **2138pass/2fail/6ignore(exit101)**, focused007 **35pass/0fail/0ignore**, integration009 **8target/72pass**. 원시 named 결과를 중복 제거하면 현재 library2140+integration72+unchanged socket1=**고유2213**이다. focused35 또는 이번host를 중복 가산하지 않았다. 단일 all-targets green을 주장하지 않는다.
- 두 실패 중 projection은 정확히 후속 focused에 성공하고 추가 다른rarity 시험1개도 존재한다. socket은 sandbox EPERM이며 snapshot.rs·tests.rs byte 동일과 기존 허용환경socket002의1pass를 확인해 재사용했다. ignored6은 실제 calibrated issuer/exclusive scenario를 요구하는 기존 사례이며 PASS가 아니다.
- native4는 이전 동일 G11 fixture/native profile와 현 default G11 성공에 더해 이번 dealer actualhost·비용이 새 active owner를 검증한다. server31은 writer/reader·server 구현/시험 불변과 이전31pass 원시, 새85명령/56결과 compile·projection, 최종통합과21freshhost/153commands/4IO의 새 dealer 실제 소비를 근거로 재사용한다. 새 server/native 전체 실행을 했다고 표기하지 않는다.
- 선행 U16-2의17수용 Complete/High는 immutable historical 원문을 인수했다. 시작시 baseline527 전부가 그 target과 동일하다. 후속 변경은 공통 command/session/layout/VN/escrow return과 측정에 귀속하며 U16-2 8시험·시설9·일반reward/72통합/정상host로 현재 공유경계를 확인했다. 원래 U16-2 전체를 새로 재감사했다거나 이전 audit target이 현재 source와 byte동일하다고 주장하지 않는다. 이번 U16-3 판정이 관련 새 delta의 continuity를 담당한다.
- 재사용 정상host 합계21process/153commands/실제IO4, 작성정상1/거절14, 비용9process/1152창을 원시와 대조했다. prepared build/receipt·실제binary hash·admission/calibration의 source/compiler/graph/worker pair가 fixed target과 결속되어 있다. 이번 실제host replay도 그 승인된 binary·기존 local cache로 성공했다.

실패 원문과 교정 경로는 [EXPERIMENTS](EXPERIMENTS.md)와 frozen runs에 유지되어 있다. 초기speaker 누락6실패는 정상 로더의 작성거절이며 speaker 추가 후 원래 검증목표를 유지했다. projection fixture 불일치와 기타rarity 제품범위 교정을 구분했다. 잘못된 target/bin 명칭의0test·빌드실패, offline yanked dependency resolver, packaging parent/pilot 입력오류, unavailable route,10M 준비한도 도달, collector None집계 오류는 기능 PASS로 세지 않았다. prepare003은 중복관측을 scoped 관측으로 바꾸고 같은 실제거래/10clocks를 유지했다. 현재 제품한도 상향이나 실패 원시 삭제를 발견하지 못했다. 이번 유효 host probe와 증거검산에는 실패가 없었다.

## 비용·한계·제외와 종료

9개64명령+64Run 창의 각 accepted/nonzero와 allocator 종료 live bytes=baseline을 raw에서 검산했다. 실제 requested heap peak 최대103,078,554bytes, RSS peak168,042,496bytes, native cumulative admitted7,676,300bytes 및 owner request1,122,142bytes는 서로 다른 수치다. retained16MiB를 process heap/RSS 상한으로 오인하지 않았다. 준비 authoring9,227,138/10M과 별도 정상 release CMVA calibration/adoption 및 기존 reviewed limits 유지가 확인된다. dev opt0/debug0 diagnostic 시간/실제heap·RSS를 release기기SLO나 성능개선비율로 주장하지 않는다. 확장 입력은 public default loading/측정이며 확장CMVA 채택을 주장하지 않는다. 정당한 한도변경/한도도달 은폐는 없다.

발생근거는 정상 authored 오류/명령오용/자격오류/실제IO를 **범위 안 — 지원 경로**로 판단했다. 외부 save/checksum/메모리/내부상태의 임의변조만으로 가능한 방어는 **범위 밖 — 외부 임의 변조**이며 새 finding이나 완료 차단을 만들지 않았다. 발생근거 미확인을 제품결함으로 승격한 사례나 필수조건의 미검증을 면제한 사례는 없다. 완료를 침범하는 inaccessible/unknown evidence는 남지 않았다.

공식 유물공급은 계속 빈상점이며 대표 외부pack 검증은 U21 콘텐츠/밸런스 완료가 아니다. private issuer credentials는 제외되어 새 환경은 자기 calibration/adoption이 필요하고, 기존 local cache에서만 이번 frozen host 재현을 확인했다. 상속된 ignored6과 환경별측정 한계는 남는다. U16-4/전체U16/Unity/개별최종보스/최종SLO는 이 판정에 포함하지 않는다.

모든21수용·10차원·5IVA·2노드가 현재 직접 의미검토 및 유효성 확인 후 재사용한 근거로 충족되었다. 독립 감사자는 코드 교정이나 target 변경을 하지 않았으며, 출구 currentness가 유지된 **이 U16-3 Single Goal만 Complete / High**로 종료한다. 추가 구현을 시작하지 않는다.
