# U08 Continue 교정 회수 — 2026-10-06/008

Sol은 game/relic/allocation.rs와 world/active_run/continuation.rs 두 파일만 교정했다. typed decode의 기존 map/string/sequence/encoded4 reserve와 독립 JSON Value 저장 reserve를 구분하고, 두 named set의 실제 sort 전에 cached tuple/key Vec 공간을 checked 사전 예약한다. 공통 profile/owned projection 식·encoded Vec/Arc/envelope 계량은 원문 그대로다. root가 실제 diff와 해당 블록 원문, cfg(test) 불변을 대조했다. 8MiB·누적 admission/무환급·wire/checksum·정렬 의미·권한·기존 원본을 유지한다. product_continue_allocation_fix001의 RECORD/SOURCE_BASIS/INPUT_SHAPE_ANALYSIS를 따른다.

source434/683f1d6d…에서 758179d74cf0bec51270481173856c0ee378d8ee78dcafce9957a60544e96ce1로 두 파일만 변경, membership0/목록 밖0이다. relic_continue_allocation_fmt001 rawexit0/3.287684초, fmt source 변경은 승인 syntax-only이며 cfg/test/fixture 변경0. root가 receipt/raw/전체 입력맵을 대조했다. Sol은 빌드·시험을 하지 않고 runner0/writer HOLD로 Astra에 넘겼다.

Astra relic_public_e005는 source758179d7… 불변/117.704351초/exit101이다. E004와 E005의 content manifest/scenario/전 checkpoint 및 child 입력 전 필드(parent PID만 제외)가 실제 동일하다. 같은49,775byte checkpoint가 fresh child PID318(parent316)에서 Continue를 통과했고 generation/relic/inventory/fingerprint/Corrupt plan210/history/loads oracle를 만족했다. Normal180으로 돌아오는 Load 검증은 parent에서 수행했으며 child에서 별도 Load를 실행한 것은 아니다. child raw는1/1 PASS/3.33초, stderr0이다. 메모리 한도 실패를 fixture 축소나 cap 변경으로 피하지 않았다.

전체 E005는 이후 step47 begin_skill_targeting에서 InvalidAction으로 실패했다(raw59ba1023b8fd636163914d411552e3beb173e837db80eda7f62d1bfeda266bb7). 후속 actual HP210 소비/trace parity는 아직 성공 증거가 아니다. parent의23초 대기는 원 resonance25 helper의 준비조건이고 신규 fragment는 resonance1이며 유물은 basic에도 적용되므로 실제 step46 phase/readiness를 먼저 확인한다. 이는 미확인 원인 가설이며 Astra가 실제 관찰로 분류한다. 실패 원본·export는 보존하고 승인 E1 범위의 일반 fixture 교정을 자율 진행한다.

신규 world/tests/u08_continue_owner.rs+tests/mod.rs 등록 및 두 시험을 승인했다. 실제 codec의 current/SavePoint 비어 있지 않은 두 set 배열 permutation에도 checksum/재export 동일, Continue decode→checksum→stable recapture 누적 exact/one-short와 공개 gameplay roots 원자성을 검사한다. typed boundary fixture는 정상 공개 획득 증거와 구분한다. transaction_revision은 application.rs1724의 권한barrier라 실패에도 증가하며 rollback oracle에 넣지 않는다. 실행 budget/fault도 환급하지 않는다. 기존 allocation2/durable3/Eventowner4 및 cfg owner75 교정을 이어서 시행한다. 아직 신규시험 결과는 없다.

U08 GEX10 partial을 유지한다. 기존 집중25 및 각 보상회귀의 유효 범위는 보존하되 새로운 제품변경 관련 재검증은 진행 중이다. 최종 통합/성능/동결/전체 독립 감사 전이다. 현재 Astra 단독 test writer/runner, Sol 제품·통합 HOLD; root는 계획/읽기/문서만 담당한다.
