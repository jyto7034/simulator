# 검증 및 인수 계약

계획 v2.1 · 2026-09-09. 작성 시점의 실행 결과: 모든 신규 검증은 미실행이다. 아래는 미래 테스트의 필수 기대 결과이며 실제 통과 기록이 아니다.

## 최종 완료의 네 경계

1. **엔진 제작 기반:** 지원 어휘의 추가·교체·재조합을 최신 외부 RON으로 작성·검증·실행·설명할 수 있다.
2. **대표 공식 이식:** C01–C07의 합의된 기계적 구조를 실제 RON 및 실행 증거로 확인한다. 미정 값을 공식 수치로 확정하지 않는다.
3. **전체 표현력 대조:** CA00의 전체 원문 요구에서 미해결 공통 기능을 빠짐없이 드러낸다. 누락이 남으면 완전한 기반 완료가 아니다.
4. **별도 비범위:** 전체 공식 콘텐츠 데이터 이식, 밸런스 완료, RON 조합기, Unity/서버 실제 통합, 배포 서비스 완료는 주장하지 않는다.

## 테스트 묶음

| ID | 계약 | 필수 확인 |
| --- | --- | --- |
| T00 | 기준선·coverage | 원문별 합의/미정·작업·정책·검증의 연결과 실제 baseline 실패 구분 |
| T01 | 작성 타입·descriptor | effect ID 독립, 같은 효과 태그 AND, property/consumer 지원, 미지원 거부 |
| T02 | 외부 팩·최신 schema | 중복/미선언 override/누락 의존/순환/경로 오류/구형 문법 거부; parser·compiler 성공 없이 publish 불가 |
| T03 | generation pin | 한 바이너리에서 RON A로 런1 시작→RON B로 런2 시작; 런1은 A, 런2는 B. 실패한 B는 런1이나 팩 선택을 바꾸지 않음 |
| T04 | 도구 공통성 | API/CLI가 같은 검증·실행·diagnostic을 사용. release 로딩이 test-only builder에 의존하지 않음 |
| T05 | 소유·상태·자원 | 사본 분리, 전투당 count 유지, 배치 stack reset, atomic cost, stale event 차단 |
| T06 | 수식·집계·성장 | Attack/SpellPower 혼합과 double count 방지, max/sum/grouping, 단위·overflow·0 나눗셈, snapshot 경계 |
| T07 | 사건·피해·처치 | action/strike/segment 분리, 실제 HP/흡수/회복/비용 구분, pre-death 상태·credit·lethal 우선순위 |
| T08 | 대상·전달·flow | AoE 뒤 self heal 1회, sequence/parallel 결과, 유한 repeat, 무대상 기회 소비, 고정/추종 geometry |
| T09 | 지속·기본 공격 | channel과 preparation 분리, aura 이탈 해제, Hello timeout/reinput, source lock·modifier 소유권 |
| T10 | 방어·제어 | shared shield 비복제, 원천 생성 modifier 1회, mitigation 중복 방지, parry·HP 1·비용/처형 분리 |
| T11 | 파편 사본·형태 | 같은 정의 A만 개화, B 미개화; 사본 이전·선택 형태·Run 자원·recipe·저장 연결 |
| T12 | 유물 관리 | 공식 3슬롯, 독립 사본, 직원별 중복/비호환, 휴면 선장착, 사망 회수·분쇄 |
| T13 | 합성·활성 | equip order 독립, effect 선택 정확성, 생성 효과의 후속 강화, 전용 selector 보존, 다중 Replace 충돌 |
| T14 | 연쇄 | A1.e 재방문만 차단, 다른 사본/효과/분기 구분, 지연 이벤트 인과 유지, 새 독립 행동 재허용 |
| T15 | 매개체 | trap/object/actor/area/shield 구분, 배치/진화/HP fallback atomicity, source/점유/종료 |
| T16 | 연결·전파·집계 | 원천별 infection, death 우선 전파, 기폭에서 재감염 차단, tile grouping, 링크 진화/삭제 |
| T17 | 보스·wave | typed task skill call/result, 방향, HP skip, 상태 교체, 작업 신호, cycle/capability validation |
| T18 | 등록·획득·전투력 | 새 콘텐츠 normal grant/loadout 진입, 복수 파편, explicit replacement, effective plan 평가 |
| T19 | 침식 | 실제 시전 gate, 현재 필드 kill, 부가 효과 제외, 미출현 wave 보존, 대가 선반영, 강제 clock |
| T20 | 결과·저장·복원 | 최신 schema/fingerprint, result delta exactly once, retry와 retreat 차이, reconnect no reset |
| T21 | 공식 대표 | C01–C07 최신 RON→정상 검증→실행→결과 evidence. 이름만 같은 placeholder는 불합격 |
| T22 | 독립 재조합 | 동결 engine에서 data-only pack 변경, 네 도메인 생성 및 cross-pack 조합 |
| T23 | 안전·결정론·성능 | compile/runtime budget, typed failure/rollback, hash iteration/diagnostics 독립, 목표 환경 계측·회귀 |
| T24 | 독립 감사 | 전체 COV 연결, 필수 gate 해결, legacy/특례 없음, 실제 test·receipt·남은 제약 명시 |

각 묶음에는 정상 결과뿐 아니라 경계·실패·재시도·재배치·철수·사망 등의 해당 거부·실패 시험을 둔다. 같은 이름의 private helper 호출 성공만으로 실제 제작 경로를 검증했다고 보지 않는다.

## 실제로 사용할 기존 테스트 진입점

아래는 이번에 확인한 package/파일을 기준으로 구현 AI가 실행할 **명령 후보**다. 이 문서 작성 과정에서는 실행하지 않았다. 실행 시 root와 환경·target을 기록하고, 테스트 이름/feature가 변경됐다면 최신 명령으로 갱신한다.

```bash
# simulator 작업공간 루트 기준
cargo check --manifest-path core_enhanced/Cargo.toml
cargo check --manifest-path core_enhanced/Cargo.toml --release
cargo test --manifest-path core_enhanced/Cargo.toml --test ron_loading
cargo test --manifest-path core_enhanced/Cargo.toml --test skill_runtime_contract
cargo test --manifest-path core_enhanced/Cargo.toml --test live_item_skill_activation
cargo test --manifest-path core_enhanced/Cargo.toml --test live_skill_catalog_audit
cargo test --manifest-path core_enhanced/Cargo.toml
```

`test-fixtures`가 필요한 test/support 경로는 해당 feature로 별도 검증한다. `event_log_checks`는 현재 opt-in이므로 영향 범위를 확인하고 필요한 경우 따로 실행·기록한다. opt-in 실패 또는 미실행을 숨기고 all-features PASS라고 주장하지 않는다. 실제 0 tests 실행된 filter는 완료 증거가 아니다.

새 content CLI의 validate/describe/explain/simulate 명령은 CA03에서 구현·등록한 뒤 정확한 사용법을 기록한다. 위 명령에 없는 새 옵션이나 CLI가 이미 존재한다고 가정하지 않는다. 이 범위에서는 `game_server`나 Unity build/probe를 실행 완료 조건으로 요구하지 않는다.

## 성능·실행량

CA00/CA03의 기준선부터 decode/compile/effective-plan 작성 시간, battle step 지연, event 수, active state 수, peak memory, retained generation 수와 artifact/log 크기를 계측한다. CA22에서 현실적인 보스·물량·projectile·area·link·chain 혼합 부하와 공격적인 한계 입력을 모두 측정한다.

허용 수치는 PG08에서 목표 하드웨어·빌드 모드·시간 step·부하를 명시한 뒤 정한다. 임의의 60fps, 몇 ms 또는 동시 N만으로 성공을 미리 보장하지 않는다. 캐시는 generation/형태/loadout/원천 상태 dependency를 포함하고 전역 정의를 가변으로 만들지 않는다.

실행 예산 초과는 더 이상 처리하지 않은 효과를 조용히 생략하고 정상 승리로 끝내는 방법으로 처리하지 않는다. typed content/runtime error로 원인·scope·예산을 보고하며 해당 authoritative mutation을 실패/복원하는 계약을 검증한다. balance amplifier cap과 CPU/메모리 방어는 별개다.

결정론 합격은 실제로 검증한 engine revision/toolchain/target/환경 범위를 명시한다. 다른 운영체제·CPU·물리 backend까지 측정 없이 동일하다고 확대하지 않는다.

## receipt 최소값

| 필드 | 의미 |
| --- | --- |
| test/requirement/goal ID | 무엇을 증명했는가 |
| revision 및 relevant dirty diff | 어떤 코드를 검사했는가 |
| engine contract/schema, resolved pack fingerprint | 어떤 실행·콘텐츠 계약인가 |
| seed·initial state·입력 시퀀스·clock | 재현 가능한 실행 입력 |
| exact command/runner, 환경 | 실제 수행 경로 |
| executed/pass/fail/skip 수 | 실제 runner의 값. 없으면 보고되지 않음 |
| expected/actual state·trace·error | 결과와 비교 |
| resource/performance measurement | 해당하는 경우 실제 측정값 |
| artifact 경로와 남은 제한 | 재확인 가능한 증거 |

구현 완료 시 각 Goal의 독립 검토 문서에 증거를 연결한다. 기존 테스트를 삭제해 통과 수를 줄였다면 어떤 승인된 의미가 바뀌었고 어떤 새 테스트가 coverage를 대체했는지도 기록한다.


## v2 추가 인수 T25–T29

| ID | 계약 | 필수 기대 결과 |
| --- | --- | --- |
| T25 | 설계 준비·변경 정합성 | 각 CA의 current-path·R 대조·L 타입/상태/순서·DG 근거·설계 변경 영향이 K revision과 일치한다. 문서 existence만으로 통과 아님 |
| T26 | 보스 task 포트 | Accepted/Released/FlowFinished/EffectsSettled 구분, wait의 finite 검증, stale 완료·timeout 차단, unreleased cancel/released 존속, multi-actor phase wait·typed signal |
| T27 | effect/carrier scope·state binding | 피해 footprint를 바꿔도 self-heal·독·carrier가 암묵 변경되지 않음. 서로 다른 사본의 template cache가 runtime state를 공유하지 않음 |
| T28 | 합성 유한성·인과 분리 | grant 자체 확대·negative activation cycle 진단, branch-local 방문, true join의 명시 ancestry·delayed root 정책, 진단/log 정리와 execution 독립 |
| T29 | source/publish/save/fault | raw/effective 검증 분리, 합법 override와 dangling 참조 구분, 읽는 중 변경·예산 오류 시 미게시, 시험 session 검증 후 교체, 의미 fingerprint mismatch·공개 atomic boundary 복원 |

T26은 A05와 E02의 보스 trace를 포함한다. 최소 t0 admission/t100 release/t300 impact를 시험 입력으로 주입하고 Released/EffectsSettled waiter의 서로 다른 정상 결과를 확인한다. 수치는 시험 입력일 뿐 공식 공격 시전시간이 아니다.

T28은 lineage join의 PG02 결정을 기록한 뒤 해당 기대값을 검사한다. 문서에서 제안했다고 구현자가 모르게 새로운 gameplay 의미로 확정하지 않는다.

T29는 이전 timestamp에 이미 확정된 모든 결과를 소급 rollback한다고 요구하지 않는다. 해당 command/advance가 외부에 공개하는 atomic boundary에서 state/queue/RNG/ID/log를 일관되게 처리하고 runtime fault를 정상 승리로 감추지 않는지 검사한다.

### 설계 수정 이후 증거 유효성

CA22 등의 최적화로 source revision이 바뀌면 바뀐 경계와 K 계약에 영향을 받는 receipts를 재실행한다. CA21 동결 뒤 실행 의미·compiler·schema가 바뀌었으면 재동결과 필요한 독립 재조합을 다시 수행한다. 단순 계획 DAG가 evidence 재검증을 막는 순서 장벽은 아니다.

### 문서 패키지 점검과 프로젝트 검증의 구분

이 패키지를 생성하며 수행하는 파일 링크·CA ID·dependency cycle·JSON·해시 검사는 문서 내부 정합성 검사다. Rust 빌드·RON parser/compiler·전투 실행·성능 시험을 수행한 것으로 취급하지 않는다.

## v2.1 — 순서·준비도의 검증 근거

T00/T25와 각 작업의 기존 dependency/validation 요구에 다음을 포함한다. 새로운 P2G/GEX Gate나 전역 검증 통과를 요구하는 것은 아니다.

- 초기 상위 경계 조사, 선택 범위의 코드 검토, 정식 Phase Plan, 실제 선행 구현·감사 준비를 구분한다.
- 관계마다 생산자·소비자·필수 결과·source와 무시할 때 깨지는 상태/완료 주장을 확인한다. 진행상 권장은 필수 edge로 꾸미지 않는다.
- 다음 Phase 없이도 현재 범위가 유효한지 확인하고, 필요한 최신 reader/writer·저장·실패/복원·실행 fixture를 같은 전환에 배정한다.
- 후보 edge를 수정할 때 이유와 영향을 남긴다. 정식 current Phase Plan 변경은 P2G owner에서 수행하고 summary/JSON만 바꾸지 않는다.
- 문서 링크·배열 일치·cycle 부재 같은 정적 검사만으로 코드 의존성·순서 타당성·실제 실행 준비를 통과했다고 보고하지 않는다.

CA00 완료가 후반 모든 세부를 검증했다는 뜻은 아니다. 필수 선행 검토는 선택한 범위에 적용하며 무관한 미래 미검토 관계로 그 범위의 완료를 막지 않는다.
