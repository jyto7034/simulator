# U17 독립 완료 감사 — 2026-10-16

사용자 요청: `CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md 문서를 지침 삼아 u17 독립 감사 작업에 착수해줄래?`

GEX-20 stage-run. 이 실행자는 U17 구현·교정에 참여하지 않았으며 이번 pass에서 제품/시험/동결 target을 수정하지 않는다. 서브에이전트와 자동 교정/U18 착수는 없다. 원본 U17도 아직 감사 전이므로 GS-CMER-001 / GOAL-CMER-01 / PHASE-CMER-01의 A01–A22, 10차원, 6IVA, 3실행 node와 저장·복원 후속 완료 조건을 함께 판정한다. 후속은 새 formal Goal이 아니다.

기준은 현재 연속 지시서, GEX Reader Contract/§5/§6.7/GEX-20, P2G_50_GOAL_SET.ko.md, 원본 single_goal/PLAN.md 및 storage_refactor/PLAN.md, 로드맵 U17/후속이다. 외부 임의 변조 전용 사례는 제외하고 지원 명령/제작 입력/실제 IO 장애만 finding 근거로 삼는다.

## 입력·현재성

- 원본 `single_goal/target/u17_current_001`: source979/evidence688.
- 후속 `storage_refactor/target/u17_storage_001`: source626/evidence1103.
- ENTRY_CURRENTNESS.json: 후속 전부 일치, 기존 evidence 전부 일치. 기존 source의 12개 변경은 후속 source에 귀속한다. 추가 테스트 continuation_contract.rs도 후속에 포함된다.
- 기존 source 중 후속 외 변경은 ROADMAP.ko.md 한 개다. ROADMAP_DELTA.diff에서 U17 기존 요구는 동일하고 승인된 후속 계약/인계 링크만 추가됨을 확인한다. 후속 계약을 이번 감사에서 직접 소비한다. 과거 target을 현재 전체 source라고 주장하지 않는다.
- 작업 root와 HEAD는 ENTRY_CURRENTNESS.json에 기록한다. 무관 dirty 작업을 그대로 보존한다.

## 검토·실행

1. 최초/반복 원장, 부화 자격/지불/commit, 결과 선택권, 예약·후퇴·입장 전 복원과 SavePoint를 실제 producer/consumer로 추적한다.
2. 후속 source delta 전부: 공통 복사, native admission, typed decode/Value/normalization/bytes/Arc, canonical checksum, cached Continue 재사용, 정렬 resource lookup, 완료 node 수명과 relic receipt 보존을 검토한다.
3. 현재 raw 검증을 원문·runner assertion·binary/config/content binding과 대조해 재사용한다. original workspace 실패와 교정, 후속 socket EPERM/권한 재실행, ignore와 진단 성능의 한계를 유지한다.
4. 새 집중 실행은 runtime-observation의 U17·Continue owner·allocation·runtime profile 복원 경계를 한 Cargo 호출로 선택한다. 예상 캐시 빌드 2–8분, 첫 진단 60초; 10분 초과 시 로그/진행 확인. 테스트 목표는 기본 한도에서 no-relic typed decode 계량, 독립 checkpoint alias, 실제 후퇴/결과/보상/복원과 exact/one-short 원자성이다. 기존 전체 결과는 유효성 검산 후 재사용한다.
5. 필요한 공개 probe는 실제 발행된 저장본의 사본과 고정 host를 사용한다. 원시 저장 필드/체크섬을 편집하지 않는다. 시험 추가 전 구체 경로와 oracle을 여기 기록한다.
6. archive/member/actual preimage/delta/정역복원 근거를 검산하고, 출구 현재성 후 별도 COMPLETION_AUDIT.md로 판정한다. finding은 구현 owner에게 반환하며 audit 중 수리하지 않는다.

## 추가 발생 경로 확인 — authored kind ID

`world/revival.rs::hatchable_node`는 category/실제 encounter class/예약 metadata를 확인하면서 `kind_id.contains("elite")`도 사용한다. FacilityActivityDefinition과 MapNodeDefinition은 kind ID를 참조로 사용하고 generator의 기존 special case는 exact `combat_elite`다. 의심만으로 finding을 만들지 않고 지원 입력으로 검증한다.

- 준비: 기존 정상 RON fixture를 감사 전용 디렉터리로 복사한다. 기존 `combat_monster` 정의를 보존하면서 같은 필드의 `combat_elite_training` 정의를 추가하고 facility의 `ordinary` activity가 이 새 kind를 참조하도록 한다. category Combat, normal encounter, activity `ordinary`, 예약/전조/비용/연결/전투/보상은 그대로다. package/validator/정상 host calibration/admission 통과 여부부터 확인한다.
- 경로: 동일 공개 일반/엘리트/첫 보스 승리→상점 촉매 구매 확정→SavePoint→미예약 ordinary slot6 선택. 저장/메모리/내부 state를 편집하지 않는다. baseline은 기존 current 정상 full-flow와 새 집중 공개 U17 검사다.
- 기대: 알 Stage2와 촉매4개, 미예약 Available normal slot6은 이름과 무관하게 부화 가능하고 명령이 성공해야 한다. 실제 query와 직접 hatch 결과/거절 뒤 자산 불변을 기록한다. loader가 의도된 계약상 거절한다면 지원 경로 실패라고 주장하지 않는다.
- 예상: 기존 333명령 전체보다 짧은 준비 경로, host 응답당180초; package/calibration/admission subprocess는 기존 유한 제한을 사용한다. 원본 issuer가 있던 /tmp는 현재 없어 새 issuer를 /tmp에 생성하고 private key를 증거로 복사하지 않는다.

결과: `kind-package001.stderr`에서 정상 loader가 `allocation role 'normal' activity 'ordinary' has incompatible semantics`로 거절했다. `live_ron.rs:1216`의 normal role 규칙도 동일한 이름 제약을 검증한다. 따라서 의심한 상태는 이 지원 제작 경로로 생성되지 않는다. 원시 입력과 거절을 보존하고 제품 finding을 만들지 않는다. calibration/admission/gameplay 분기는 실행되지 않았다. 이 관찰은 외부 변조를 실제 수행한 증거가 아니라 작성 오류 거절 증거다.

## 새 프로세스 공개 Continue probe

집중31개는 통과했다. 기존 고정 `restore_cost` 실행파일과 source/build/비용 BINDINGS를 대조한 뒤 별도 process4개에서 공개 CoreApplication Continue를 수행한다. 정상 writer의 `save_before_hatch`(SavePoint alias), `Stage3_choice_selected`(선택권), 확장64후보 writer의 `Stage3_choice_pending`, 변경 전 V17 writer의 `Stage2_hatched`를 원시 저장 그대로 소비한다. envelope에서 base64 decode만 하며 current/checksum/profile 의미를 수정하지 않는다. expected는 같은 writer가 기록한 공개 snapshot이다. inventory/map/roster/egg/hatched node와 requested heap 회수를 확인하고, 선택권의 전체 snapshot/다음 명령은 새 집중 시험의 fresh_application helper 및 유효한 host fresh raw로 확인한다. 기존 전체333명령·IO·비용12회는 재실행하지 않는다. 네 프로세스는 순차 실행하고 각600초 기존 제한을 사용한다. 이것을 새 host IO 검증이나 release 성능 보장으로 표기하지 않는다.
