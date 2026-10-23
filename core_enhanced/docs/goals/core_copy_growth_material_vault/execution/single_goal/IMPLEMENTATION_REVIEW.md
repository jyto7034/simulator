# U15 구현자 코드·증거 검토

이 기록은 GEX10 자체 검토이며 정식 완료 감사 또는 독립 감사 판정이 아니다. 기준은 FPF-CGMV-001의21수용, PP-CGMV-002의5통합, Goal의10차원이다. 직접 호출·상태 소유·원시 실행을 대조했고 현재 미해결 제품 finding은 없다.

`material_economy.rs`는 요구량과 선택행을 전부 확인한 뒤 소지/창고를 지불한다. 중복행·누락·합계불일치·금지출처·부족·checked overflow에서 다른 출처로 대체하지 않는다. `MaterialInvestment`는 사본별 실제 소지/창고 지불량만 쌓으며 환급은 재료별 총합을2로 나눈다. 개별 결제마다 내림하지 않고 현재 recipe 가격으로 재평가하지 않는다. 엔케팔린은 투자 원장에 기록하지 않는다.

`world/maintenance.rs`는 개화의 staged inventory/roster/fragment/crafting roots와 warehouse/debits를 함께 수락한다. 장비 강화도 공통 지불을 이용하고, 분쇄는 기본보상과 원장환급을 checked 합산한 reward effects를 staged grant로 적용한 뒤 사본을 제거한다. 미확인 장착분쇄와 환급 overflow는 소멸 이전에 거절한다. 상위 application candidate는 이후 오류도 폐기한다. 공개 quote는 같은 요구량/지불계획과 eligibility를 사용한다. free grant, 형태선택·직원이전·기존해금의 재요청은 새 투자를 만들지 않는다.

예약 `equipment_dust`의 owner는 `RunCraftingResources` 하나다. 기존 typed grant, generic material grant, 유물 분쇄, 강화, 환급과 query가 그 잔액을 공유한다. 임의 inventory mirror를 두지 않았다. 지원 작성 오류로 예약 id의 목적을 바꾸는 입력은 loader에서 거절한다. 다른 authored dust id는 기존 material stack owner를 유지한다. 공개 이벤트의 실제 typed grant가 강화되지 않았던 초기 결함은 이 생산자·소비자 연결로 교정했다.

`world/state.rs`의 checkpoint 복원은 현재 창고에 현 원정 소비를 더하고 목표 시점 소비를 빼는 차이 조정이다. 결과를 먼저 계산하고 나서 run/copy/소지와 함께 복원한다. 무관한 보관과 영구지식은 전체 snapshot으로 덮지 않는다. `world/active_run.rs`의 호스트 저장 rollback은 정확히 한 번의 미수락 transaction 전 lifecycle를 별도 controlled clone으로 보관한다. 이 토큰의 자산 복원은 SavePoint의 장기 복원과 다른 수명이며 현재 단조 지식은 유지한다. RunProgression/native map을 필드별 clone shape로 등록해 이전 serialized-leaf 경계를 제거했다.

`world/expedition_lifecycle.rs`는 실제 귀환 자격과 재료 목적/수량을 검증하고 파츠 합계2·부활촉매1 범위의 선택만 저장한다. 빈 선택은 자동 회수가 아니다. 성공 반출 장비는 강화0과 새 빈투자로 재대여되고, 파편은 같은 copy UUID/해금/투자를 보존한다. 실패·포기는 이미 소비한 창고재료를 복구하지 않고 원정 debit만 종료한다. 미사용 창고와 미대여 사본은 유지한다.

활성/Continue Core 저장16, profile envelope6, paired application journal4를 현재 계약으로 사용한다. 필수 투자필드에 구저장용 serde default나 가격추정 변환을 추가하지 않았다. 실제 version5 저장은 현재 호스트에서 명시 거절하며 원본4파일이 보존됐다. 새 grant의 투자0과 구저장 이행은 구별된다. canonical `core_runtime_contract.md`와 `data_loading_contract.md`에 새 공개 명령/조회/목적/저장 경계를 반영했고 게임 규칙의 승인 의미는 변경하지 않았다.

전체 Core002의2097 library+72 integration 통과를 재사용하고 실패6개는 목적을 보존한 fixture/기대값 교정 후 focused005에서 모두 재검증했다. focused37·socket1·native4·server29의 원문과 정확한 중복 제외는 BASELINE_REUSE/TEST_RESULTS_INDEX에 있다. 과거 실패와 ignore6을 숨기지 않았다. 외부 작성 valid1/invalid9, 공개 새프로세스6개, 실제 default host7개/165명령, 실제 IO실패3건 및 재시도가 정책 oracle을 서로 다른 경계에서 확인한다. 내부 owner fixture만으로 공개/저장 검증을 대체하지 않았다.

비용은 current9·동일 전후18·유료원장6개의 새 프로세스, 각12연산단계와 정확한 요청메모리 회수로 검토했다. 유료확장768장비+768파편 중64개 투자원장이 포함된다. calibration은 실제 host identity의 plain/instrumented 반복과 누락counter/overflow 검사를 통과했다. 내부 work와 allocator/RSS를 혼동하지 않았고 한도를 올리지 않았다. dev 진단을 release/device SLO로 확대하지 않는다.

최종 대상에는 실제 구현 전519파일, immutable 원본 supplement, 변경 없는 외부 의존성과 현재 구현/시험/계약을 묶는다. 사용자의 기존 dirty 및 실패 증거는 보존한다. source 동결과 정역복원 결과는 `target/u15_current_001`에 기록하며 이후 별도 GEX20 요청을 기다린다. 자동 감사나 U16 착수, Goal Complete 판정은 하지 않는다.
