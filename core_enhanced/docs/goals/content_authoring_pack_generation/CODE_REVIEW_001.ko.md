# R02 외부 팩·세대·저장 — 코드 기반 검토

2026-09-09. P2G-20 코드 검토 본문. Exact input은 `WR-CAPG-P2G10-001`, 요구 source plan은 `RSP-CAPG-P2G20-001`이다. Current 선택은 별도 complete Review Batch 검증 결과가 소유한다. 정책 의미·14개 acceptance는 [P2G-10](P2G_10_POLICY_001.json)의 각 current 요소가 소유하며 이 검토에서 변경하지 않는다. 이 문서는 구현·runtime PASS가 아니다.

## 1. 근거와 조사 경계

현재 `/mnt/f/work/simulator`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`의 dirty tree를 검토했다. R01 current 완료 감사와 source/test/data 해시를 재확인했다. P2G-00/10 이후 R02 production mutation은 없다. Root는 loader·application·checkpoint·catalog 선언과 직접 호출을 읽었고, 별도 `roadmap_intake_review` agent는 같은 코드의 제한된 source 조사를 했다. 이는 구현과 독립 완료 감사가 아니다.

[preliminary evidence](P2G_20_PRELIMINARY_EVIDENCE_001.json)는 앞선 read-only 조사에 대한 currentness 대조다. 최종 Source Observation은 현재 Input Working Revision과 source plan에 새로 결속해야 한다. R01 full-003의 1,166개 검사와 fresh audit probe 4개는 R01 현재 기반의 실제 근거로 재사용하며, 아직 없는 외부 팩 기능의 성공으로 세지 않는다.

공개 library와 core 내부 소비자는 접근 가능하다. 서버·Unity·배포된 저장 집합·개인 파일은 사용자 scope 밖이며 released/deployed compatibility를 주장하지 않는다. `game_server/src/main.rs:123`과 서버 시험 helper는 `GameDataBase::load_live_embedded`를 직접 호출한다. 이는 `ValidatedGameData`의 별칭이 아니며 외부 host가 새 opaque generation/public application API로 옮겨야 하는 실제 adaptation 지점이다. Core API 변경에 필요한 적용 방법은 문서에 알리되 여기서 서버 source를 수정·배포하거나 전체 workspace/server 빌드 호환을 주장하지 않는다. 이 제외를 이유로 core production에 embedded fallback을 남기지 않는다.

## 2. Current Actual

**B01 — 게시.** `data/live_ron.rs:122–191`은 24개 catalog와 bundle을 고정한 borrowed `GameplayRonSourceSet`을 생성하고 `include_str!`로 embedded 원문을 제공한다. `data/mod.rs:803`의 `load_live_sources`는 parse → `into_validated` → `Arc<GameDataBase>` 구성 → generated combat preview 검증 후 opaque `ValidatedGameData`를 반환한다. 이 마지막 게시 경계와 domain/cross-reference 검증은 재사용 가능하다. 외부 manifest/선택/snapshot/세대 fingerprint는 없다. `GameDataBase::new`의 expect/index 생성은 검증 전 외부 입력의 진입점으로 삼을 수 없다.

**B02 — 합성 전 raw와 typed identity.** `skill_data.rs::from_ron`과 custom Deserialize는 range preset 해석·skill validation을 즉시 수행한다. `corroded_employee_data.rs:242`도 유사한 raw→preset lowering을 한다. 팩별 완성 DB를 먼저 만든 뒤 merge하면 cross-pack preset이나 폐기된 loser의 참조 때문에 유효한 effective catalog를 거부할 수 있다. 초기화된 OnceLock index를 가진 Vec를 나중에 수정하는 방식도 부적절하다. 문법/타입 검사와 최종 참조 lowering 사이에 raw resolution 경계가 필요하다.

Catalog는 단일 `Vec<{id}>` 형식이 아니다. Skill/corroded range preset, equipment item/material/recipe, reward/shop pool, PvE encounter/reward source, map kind/template/pool/profile, facility tag catalog와 standard/endless schedule, singleton run policy가 있다. `EquipmentDismantleRecipeMetadata.equipment_id`는 독립 새 ID가 아니라 장비 참조 키다. Item UUID는 기존 전역 typed identity이며 표시명·UUID에 일괄 namespace prefix를 붙일 수 없다. 현재 string ID와 그 참조는 exact string으로 해석된다.

**B03 — 실제 순서.** `map/facility_generator.rs:1028`은 candidate Vec 순서로 seeded weighted roll을 처리한다. `world.rs:735`는 employee definition Vec에서 starter 후보를 만든다. 실행 배열은 그대로 두고 fingerprint만 정렬하면 다른 실행이 같은 fingerprint를 가질 수 있다. 이름공간·팩/파일 열거 순서와 실제 gameplay 배열 순서를 구분해야 한다.

**B04 — application과 pin.** `src/application.rs:782`가 공개 application root이고 `GameCore`는 bare database Arc를 가진다. `world/application.rs:128–220`은 state와 seed만 prepared commit에 넣는다. Draft의 game_data만 바꾸면 실제 커밋에서 빠진다. `world.rs:226–280`의 기존 behavior 경로는 action gating 후 domain 처리를 수행한다. `handle_start_new_game:680`은 RunState 생성 전에 RunStart VN과 briefing에 데이터를 사용한다. Starter:735–805는 같은 데이터로 직원·장비·recruitment·map을 만든다. RunState 생성 시점에 늦게 pin하면 충분하지 않다.

**B05 — 저장.** `world/active_run.rs:25–186`은 V4 reader와 V5 checksum writer/reader를 가진다. Checksum은 checkpoint payload의 무결성이지 콘텐츠 identity가 아니다. `node_flow.rs:1237`에서 capture/cache, 1309 이후에서 explicit load/continue의 공통 검증을 수행한다. `active_run.rs:240–330`과 공개 application checkpoint 포트가 export/continue를 연결한다. rollback·continuation test snapshot은 현재 state/seed 중심이다. Generation과 실제 database pin을 포함한 오류 원자성 관찰이 필요하다.

**B06 — 현재 저장 정책의 귀속.** R01 Freeze TR30-02와 Goal I06의 V4/V5 보존은 ‘이번 최소 slice에서 제거하지 않음’이다. 같은 Freeze는 새 generation 도입 owner가 저장 전환을 소유한다고 명시한다. R02 카드 §5와 현재 U04/PG07은 새 format-only와 content 일치를 요구한다. 따라서 R01 P2G 원문을 다시 쓰거나 이전 `ACTIVE_RUN_V4_READER_REMOVAL_CERTIFIED=false`를 true로 꾸밀 이유가 없다. 기존 정책/검증을 역사로 보존하고 R02가 strict 최신 reader/writer와 대체 rejection/roundtrip을 소유한다.

## 3. Approved Target과 delta

| 요소 | 현재와의 차이 / required boundary |
| --- | --- |
| CAPG-E01 | embedded 고정 input에서 동일 바이너리의 선택된 외부 최신 팩 input으로 전환. 공식 데이터도 같은 public loader를 통과 |
| CAPG-E02 | typed raw definition 단위의 add/replace/remove·명시 conflict resolve·최종 참조 검증. 의존 순서/파일 순서 last-wins 금지 |
| CAPG-E03 | opaque 검증 결과에 세대 identity/source receipt를 결속하고 briefing부터 실행/저장까지 동일한 강한 참조. 실패 B는 오류이며 A 무변경 |
| CAPG-E04 | capture/cache/export/continue/explicit load를 최신 schema+계약+resolved fingerprint로 함께 전환. 이전 형식·동일 ID 다른 내용 거부 |
| CAPG-E05 | 공식 source packaging과 사용자 live_data를 분리하고 기존 개인 profile/도감·파일을 보존 |
| CAPG-E06 | owned source 진단, 한계가 있는 파일 확보·parser·세대 수명, 지원 capability 검증과 deterministic resolution. 데이터에서 외부 코드를 실행하지 않음 |

이 표는 정책 요약이다. Exact acceptance는 upstream 요소별 배열을 그대로 사용한다. 새 gameplay 수치·저장 손실 예외·legacy fallback을 승인하지 않는다.

## 4. Proposed Implementation — 현재 API가 아닌 권장 구조

**L01 — 입력과 안전.** Core data 계층에 typed selection/manifest와 owned immutable source snapshot을 둔다. Manifest는 pack identity/version, authoring/execution capability, dependency, 상대 inventory와 명시 연산을 기술한다. 공식 source는 명시한 별도 경로에 pack으로 패키징한다. 실행 로더는 source tree와 live_data를 자동 동기화하거나 embedded 내용을 fallback으로 쓰지 않는다. 기존 embedded 시험 입력은 명시적인 fixture 경계로만 유지한다.

파일을 효과 실행마다 읽지 않는다. Selection·manifest·inventory를 고정하고 bytes/hash를 확보한 뒤 원문을 재확인한다. 파일 수·개별/총 bytes·파서 깊이·pack/dependency 입력량은 trusted host가 정한 budget으로 제한하고 실패는 typed source 오류다. Pack 자체가 budget을 올릴 수 없다. 절대 경로·root 탈출·symlink component/escape·case 충돌·중복 inventory·누락 파일·읽기 중 변경은 명시적 거부 대상으로 둔다. 느슨한 폴더를 여러 번 읽는 것을 원자적 filesystem snapshot이라고 주장하지 않는다. 유한한 재확인/실패 및 immutable directory/manifest 교체 사용법을 제공한다. 정확한 기본 한계와 비용 측정은 GEX가 실제 official 입력과 비정상 fixture를 계측해 기록할 기술 선택이다.

**L02 — typed resolve.** 기존 raw catalog root를 문법/타입까지만 파싱하고 typed kind/key로 객체·부속 설정을 나눈다. Add/Replace는 완전한 typed definition을 사용한다. Remove는 target key를 갖는다. 임의 field path patch나 새로운 evaluator는 필요하지 않다. 원문별 범위 검사는 가능하지만 최종 참조·range lowering·compiled boss/index 생성은 winning effective set이 정해진 다음에 수행한다. 모든 manifest dependency는 별도로 검증하고 cycle/누락을 거부한다.

동일 target의 모호한 다중 writer는 오류다. 명시 selection winner 또는 명시 replacement 관계가 있을 때만 결정한다. 중복 Add, 없는 target Replace/Remove, 잘못된 winner와 remove/replace 경쟁을 조용히 무시하지 않는다. Final reference 검사는 폐기된 loser를 보지 않으며 실제 winner/removal 결과를 검사한다. Resolution은 기존 공식 catalog 순서와 replacement target 위치를 보존하고, 새 독립 항목의 순서를 명시한 안정 규칙으로 구성한다. File enumeration의 우연한 순서가 승자나 실행 순서를 바꾸지 않는다.

**L03 — namespace.** 권장 최소안은 명시 `Global` canonical string key와 `Named(namespace)`의 완전히 적힌 `namespace::local` key다. 작성자는 선언과 참조에 같은 exact canonical key를 적는다. 숨은 shorthand 해석·namespace guessing·UUID 변환을 추가하지 않는다. 공식 원본은 명시 global key를 사용해 기존 의미/순서를 보존할 수 있고, 공식 namespace에 별도 보호 권한을 두지 않는다.

Typed key adapter는 독립 string identity, 기존 identity를 참조하는 extension key, singleton configuration, definition-local key와 UUID를 구분한다. Namespace Add 규칙은 독립 string identity에 적용한다. 기존 global 장비에 recipe를 추가하는 참조형 extension은 명시 equipment key를 유지하고 최종 참조/충돌 검사를 적용한다. UUID는 기존 전역 유일성을 유지한다. 이 제한은 namespaced 신규 string 정의의 충돌 분리와 기존 typed identity 계약을 함께 보존하며, 모든 string/UUID를 일괄 prefix하는 migration을 피한다. 정확한 enum/adapter 목록은 GEX 상세화에서 모든 catalog를 대조해 완성한다.

**L04 — source와 fingerprint.** Winning definition마다 원래 pack/file/raw field locator를 보존하고 효과 출처를 여기서 compile한다. Merged Vec의 새 index를 원래 파일 위치인 것처럼 기록하지 않는다. LiveRonError/parser source 인자는 owned/Cow 값으로 확장하며 동적 path를 leak해 static lifetime을 만들지 않는다.

Source byte receipt와 semantic fingerprint는 별개다. Fingerprint는 전체 effective typed authored catalog(compile 전 boss/summon 원문 포함), 실제 의미 있는 배열 순서, resolved selection/winner 및 authoring/execution contract를 해시한다. Map/object key만 canonical 정렬하고 배열은 실행 순서대로 기록한다. 주석/공백/파일 위치 변화만으로 의미가 바뀌지 않으면 같은 semantic fingerprint를 사용하되 source receipt hash는 다를 수 있다. Derived index, Arc 주소, 증가 counter, cache와 EffectSource 진단 metadata를 의미 해시에 넣지 않는다. 같은 실제 실행 의미와 순서를 얻도록 runtime 구성도 같은 resolved representation을 소비한다.

**L05 — publication와 application.** Opaque validated generation이 database Arc와 contract/fingerprint/source receipt를 함께 소유한다. 기존 final validation·generated setup이 성공한 뒤에만 생성/게시한다. App/GameCore가 token identity를 버리고 bare database만 보관하지 않게 한다. Global 무제한 generation registry 대신 현재 selection과 실제 런의 강한 참조가 lifetime을 소유하고 미참조 세대는 회수된다.

권장 전환은 기존 application의 generation-aware prepared commit이다. Candidate token은 StartNewGame 요청의 비영구 typed 인자로 전달하고 live setter나 pending global slot을 만들지 않는다. `action_capability.rs:127–148`의 blocking VN/NotStarted gate를 먼저 통과한 draft의 StartNewGame 분기에서만 token을 설치한 뒤 tutorial 자격·RunStart VN·briefing projection을 검증한다. 후처리/context 검증까지 성공하면 state·seed·generation·출력을 하나의 prepared commit으로 전달한다. Database와 fingerprint를 각각 교체해 불일치가 생기지 않게 token을 단일 authority로 둔다.

실패하면 B는 draft와 함께 폐기되고 live A의 token/state/seed/profile은 그대로다. 단순 새 Core 생성이나 현재 사용하지 않는 `replace_active_run` bool로 active run/briefing/RunFailed gating을 우회하지 않는다. 기존 허용 AbandonRun은 run/pending/checkpoint를 비우고 profile을 보존하며 마지막 token이 idle configuration으로 남을 수 있다. 다음 generation 선택은 새 요청이 소유한다. 새 briefing 취소 동작은 추가하지 않는다. 통상의 명령·전투·후속 VN/starter는 해당 런의 pin을 유지한다. 별도 새 application은 검증 token으로 생성하고 matching-content checkpoint를 가져올 수 있다. R03 test-session reload는 이 publication/transaction 계약의 후속 소비자다.

**L06 — 저장.** 새 최신 checkpoint schema와 contract/fingerprint identity를 도입한다. Capture, encoded cache, export, latest importer, continue 및 explicit load가 같은 identity를 확인한다. Missing fingerprint·구형 V4/V5·unknown field/version·checksum/content mismatch는 state/profile/seed/generation 변경 전에 거부한다. 현재 checksum·seed 일관성·load count·run lifecycle·도감/profile 보존 검증은 새 형식에서도 유지한다. 이전 reader/importer와 임시 certification flag는 R02 승인 전환의 귀속을 기록하고 제거한다. 개인 파일 읽기/삭제/자동 변환이나 추정 fingerprint를 추가하지 않는다.

## 5. 최소 동반 전환과 대안

R02/AT02의 유효한 중간 상태는 외부 snapshot/resolve/compile/publication, 새 런 pin, fingerprint 저장이 함께 작동하는 상태다. 이 경계는 여러 내부 transaction을 하나로 합치라는 뜻이 아니다. Source/compile와 application/save의 reader/producer 계약을 분리 완료하면 미검증 세대가 소비되거나 같은 ID 다른 내용의 저장이 복원될 수 있다. P2G-40은 이를 최소 동시 전환 node와 직접 durable validation node로 배정해야 한다.

팩별 완성 DB 후 merge는 final-reference 정책을 깨뜨리므로 채택하지 않는다. 임의 JSON/RON reflection patch는 typed 권한과 enum/source 위치를 잃으므로 채택하지 않는다. 전역 ID 일괄 prefix는 UUID/reference-keyed extension을 깨뜨린다. State-only commit이나 무조건 fresh-core replacement는 세대 선택 또는 eligibility를 잃는다. Profile 초기화·구형 reader 유지·B실패의 A fallback은 승인된 대안이 아니다.

## 6. 검증 seam과 완료 관찰

아래 V는 후속 durable 시험 배정이며 아직 실행되지 않았다. P2G가 문서·type 존재를 게임 PASS로 세지 않는다.

| 검증 | 입력→직접 관찰 / 정책 coverage |
| --- | --- |
| V01 | 동일 바이너리·실제 A/B directory, 공식 pack packaging과 외부 원문 수정 → 실제 서로 다른 정의/실행 및 source path, embedded fallback 부재. 격리 공식 source·별도 output·기존 custom live_data fixture를 준비해 packaging 전후 source/custom bytes 보존을 대조. E01/E05/E06 |
| V02 | 독립 namespace의 동일 local 이름, global override, recipe extension, explicit winner/chain, shuffled pack/file 목록 → exact typed key·안정된 실행 순서/지문. 참조를 함께 정리한 유효 Remove 뒤 최종 catalog·조회 index·등록/획득 소비자에서 제거 결과를 관찰. 모호 writer·중복/누락 dependency·cycle·없는 target는 오류. E02/E06 |
| V03 | cross-pack skill/corroded preset, loser의 폐기된 dangling ref, winner가 남긴 dangling buff/reward/registration → 최종 목록만 검증, late compile/generated setup 실패는 미게시. E01/E02/E03 |
| V04 | A 런/briefing 유지 중 B 게시, 새 B 런, invalid B와 B briefing 준비 실패·허용되지 않은 StartNewGame → A state/profile/seed/generation/출력 무변경, 성공 B는 briefing/starter/전투 pin. A/B export fingerprint가 각 pinned 세대와 일치함을 직접 대조. E03/E05 |
| V05 | matching 최신 콘텐츠의 새 application restart/continue/explicit checkpoint load → 복원 전후 run identity·상태·난수 진행 또는 이어지는 결정론적 결과를 직접 대조. 같은 ID 다른 내용, contract mismatch·old V4/V5·unknown fields·checksum·load-limit는 기존 상태/파일 무변경 오류. E04/E05 |
| V06 | 실제 외부 manifest/catalog의 구형 schema·미지원 authoring/execution capability·실행되지 않는 unsupported opcode, 누락/빈 selection·duplicate ID/path·case collision·절대/탈출/symlink·동시변경·file/byte/depth/dependency 한계 경계 → source 진단·token 미반환·embedded fallback 부재. 같은 입력 재시도/주석 변경·명시적 순서 변동의 지문, strong reference 회수 관찰. E01/E06 |
| V07 | 기존 catalog/item/skill/fragment/preview/map/VN/new-run/profile/checkpoint 회귀와 release lib/실제 pack 소비 binary. 전체 core baseline을 최종 owning node가 수행. E01–E06 |
| V08 | R01 IVA02/04/05/06에 대한 source/parser/publication/store 변경 영향 map, 필요한 nonzero 회귀와 current target의 fresh R01 GEX-20. 과거 V4 보존 delta와 현재 R02 전환을 구분. R01 accepted outputs 보존 |

R02는 모든 14개 acceptance가 위 seam 중 하나 이상에 대응하고, 성공/거부/부분실패/재시도/순서/경계/저장/실데이터/실제소비/legacy부재를 관찰할 수 있어야 한다. 부하 숫자는 hardware/build/input을 기록한 실제 측정만 사용하고 임의 gameplay cap을 만들지 않는다.

## 7. findings·unknown·권한

| Finding | Primary classification / disposition |
| --- | --- |
| F01 raw/compiled 분리와 모든 catalog key | implementation_finding — R02 code/data 동시 전환과 typed adapter 상세화 |
| F02 owned source·namespace·canonical fingerprint·배열 순서 | technical_decision — L02–04로 승인 의미 보존, GEX가 실제 producer/consumer와 경계 fixture로 검증 |
| F03 generation-aware application commit | technical_decision — L05. 기존 gating과 state/profile/seed 무변경을 보존하는 최소 authority 확장 |
| F04 V4/V5에서 최신 content-bound format로 전환 | implementation_finding — 이미 승인된 E04/R02가 소유, certification을 새로 가장하거나 R01 원본을 재작성하지 않음 |
| F05 R01 evidence 영향 | implementation_finding — R02에서 V08·fresh 감사 currentness 복구. 과거 evidence 삭제/소급 변경 없음 |
| F06 public CLI/test reload와 새 Flow/복사/유물/전체 gameplay | out_of_scope — 해당 R03–R24 owner. 현재 외부 pack·new-run/save 인수를 그쪽으로 미루지 않음 |
| F07 서버/Unity·배포된 개인 저장 population | out_of_scope — 사용자 제외. API/schema host adaptation 필요를 문서화하되 배포/외부 호환 PASS 주장 없음 |

현재 새 material policy나 required inaccessible source는 발견되지 않았다. 정확한 manifest 필드·enum·각 typed adapter·budget 기본값·신규 test selector는 구현자가 승인 의미와 이 동시 전환 경계를 유지하며 상세화한다. Namespace 또는 seed 순서 의미를 보존할 수 없거나 legacy/저장 손실 예외가 필요하면 P2G-10으로 반환한다. 필수 catalog/consumer/lifetime을 단지 미래 구현으로 남긴 채 Review Batch를 닫을 수 없다.
