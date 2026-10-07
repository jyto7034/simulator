# F03 C04/C05 시도 이력

## 시작

F02 사본 검토·PLAN·NOTES와 교정 지시서 C04/C05를 읽었다. 직전 동결 verify 통과 후 실제 before를 보존했다.
기존 Snapshot/Live 시험은 adapter 평가만 대조하거나 관측이 꺼져 있어 기록 조회 근거를 추가해야 한다.
실제 소비 경로는 DamageSourceSnapshot.value_expr_capture를 우선 사용한다. 기존 private encode_record_for_test는 selector를 통과하지 않음을 확인했다.
이번 scope는 C04/C05이며 시험 결과에 따라 연결 결함이 발견되면 해당 계약 안에서 교정한다.

## 최초 시험 빌드

f03_replay_001은 실행 전 컴파일 실패(exit101, 시험0건). 시험용 grant helper 인자를 i64로 선언했으나 실제 ResourceMutation::Grant는 u32다.
실제 API 타입으로 helper를 교정한다. 실패 로그·영수증과 당시 변경 source archive를 evidence에 보존했다.

## 실제 fixture 계약 대조

f03_replay_002: 2 pass / 3 fail. 공격력/HP 실제 소비·기록·query는 통과. 완료 result fixture는 필수 delay_ms 누락으로 RON parse 거부, 자원 fixture는 builder 검증 시 meter 미등록으로 거부됐다.
기존 core_with_units_and_equipment helper가 build 뒤 resources를 주입하던 순서를 with_resources→build로 바꿔 실제 resource-aware compile/validation을 거친다. 변경은 cfg(test) 안이다.
RON step에는 기존 계약의 delay_ms:0과 delivery:Instant를 명시한다. 원자료와 당시 source archive를 보존한다.

## 앞 effect 결과 조사

f03_replay_003: 4 pass / 1 fail. Snapshot/Live의 네 입력 종류는 통과했고, 앞 effect의 Physical7→완료 피해4 기대가 실제0으로 나왔다.
기대값을 낮추지 않고 실제 피해 command/event·최종 record를 출력해 원인을 구분한다. release template과 source binding, damage 계산 경로를 읽었다.

## delivery 실행 scope 교정

f03_prefix_probe_001: 진단용 단일 시험 실패를 보존했다. 직전 기록을 정정하면 실제 피해는4이고 실패한 것은 EarlierEffect를 읽은 Heal 기대4/실제0이다. 실제 projectile caller는 runtime_fact_source를 delivery source로 바꾸고 복원하지만 새 consume fixture는 carrier만 전달했다. command가 delivery None으로 기록되어 정확한 prefix 관계에서 제외된 것이다. fixture에 실제 caller와 같은 source scope를 넣고 복원한다. 제품 코드는 변경하지 않는다.

## 동적 replay 집중 검증

f03_replay_004: 신규5개 시험/11개 시나리오 통과. 공격력·대상HP·자원·완료 action result의 Snapshot/Live 8개와 앞 effect HP/resource/실제 mitigation result의 Live 3개다. 실제 소비와 record, 두 encoder의 수치·trace 및 이후 상태 변화 후 동일 출력/usage를 확인했다. 최종 source에서 전체 검증을 진행한다.

## 최종 검증 및 인계 준비

최종 source에서 check, authoring112, value_expr99, 전체1550(기본 병렬 약124초), Clippy, 변경 파일format을 통과했다. 신규 집중시험5개도 같은 source다. FINAL_VALIDATION.json에 영수증/로그/전후source를 대조해 판정했다. Clippy의 시험 helper 인자수/기존 큰 GameError 경고는 원문을 보존한다.
PC36/R05는 재실행하지 않았으며 source340개와 변경 parent의 비시험 구간 byte 동일, 신규파일 cfg(test) 연결 및 선행 원자료·도구·환경을 대조했다. EVIDENCE_REUSE.json에 근거를 연결했다.
기존 freeze.py를 이번 변경1/추가1, C04/C05 판정/재사용 근거에 맞춰 조정했다. 전후 archive와 patch를 고정하고 격리 복원을 검증한다. 최종 freeze/verify 결과는 target/RESTORATION_VERIFICATION.json 및 루트 영수증이 소유한다.
