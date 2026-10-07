# 현재 판단

- 기존 공개 legacy 시나리오 작성 helper를 cfg(test)에서 재사용한다. C07용 입력만 fixture 파일로 변경하여 실제 로더·TrialSession·선택 경로를 통과시킨다. production publication 차단 유지.
- 공개 fixture에서는 첫 Damage1 기록 A, 다음 Heal0 기록 B를 각각 공개 clock 경계에서 조회한다. 이전 기록/미래 ordinal을 혼동하지 않는다.
- runtime fixture에서는 실제 projectile queue를 진행한다. 0값의 적중은 기록이 생긴다. 적중한 관통체 만료는 추가 impact 없이 carrier를 닫고 순번·Arc를 보존한다. 한 번도 적중하지 못한 D04-F01 impact는 별도 시험에서 후속 resource·step·delivery 완료를 확인한다.
- C07의 selector 사례가 C08–C15의 권한/손상/terminal수명/비용 전체 증거를 대신하지 않는다.
- 선행 C06 실패 교훈: canonical node ID에는 predicate도 포함된다. 이번에는 해당 trace 번호를 바꾸지 않는다.

- 사용자 확정: C07-OBS01은 C09 소유이며 F03 전체 완료 전 교정 필수. 만료 경로의 pending cast1을 정상 수명 보장으로 해석하지 않는다. 실제 hit-limit terminal로 끝나는 대조 사례에서는 시전 정리까지 확인한다.
