# D07-B F02 수행 이력

## 인수 및 기준선

F01 사본 검토의 PARTIAL_SCOPE_PASS를 읽고 predecessor freeze.py verify 통과를 기록했다.
341개 실제 before 원본 archive, manifest 및 dirty Git 상태를 baseline/entry_001에 보존했다.
legacy 관측 분기의 Vec::with_capacity는 admission이 없고 expression에는 기존 site가 있음을 확인했다.
기존 transaction clone/prepare 후 실행하며 native 누적 allowance는 rollback에서 환급되지 않음을 확인했다.
새 operation마다 경계 시험을 실행하고, 성공 실행에서 관찰한 누적 요구량을 사용한다.

## 배열 연결 전 회귀

공통 helper와 expression 연결, cfg(test) 누적 admission 관찰 및 C02 시험을 먼저 추가했다.
f02_before_001은 1 pass / 2 fail로 실제 legacy 경로의 배열 admission 부재를 검출했다.
이 시점의 변경 세 파일을 BEFORE_REGRESSION_SOURCE.tar.gz에 보존했다. 이제 legacy Vec를 같은 helper에 연결한다.

## 경계 시험 계측 보정

f02_after_001은 2 pass / 1 fail. 누적 trace에서 배열 admission이 첫 항목일 수 있는데 시험이 이전 index를 요구해 underflow했다.
제품 교정 결함이 아니라 시험 계측 가정 오류다. native cfg(test) 관찰값에 해당 admission 직전 누적값도 포함해 정확 delta를 확인한다.
한도/제품 admission/실행 순서는 바꾸지 않는다. 원 실패 로그를 보존하고 새 이름으로 재실행한다.

## C02 교정 후 검증

f02_after_002: 신규 시험3개 모두 통과. Damage 다중 대상 배열은16 bytes, 누적24531 bytes에서 admission, 전체 operation46785 bytes. Heal은 배열16 / 누적19563 / 전체36811 bytes.
각 배열 누적 요구량보다1 byte 작은 새 operation에서 정확히 value_expr_evaluated_amounts site로 거부됐다. 기존 gameplay/queue/runtime facts/동일 Arc record/ordinal과 재평가 결과를 보존했다.
정확 배열 한도는 그 배열까지 허용하고 뒤의 평가 할당은 거부한다. 전체 성공에서 측정한 누적 한도와 동일한 새 operation으로 정상 재시도해 ordinal+2를 확인했다.
관측 off 및 source 없는 programmatic legacy는 새 배열 admission 없이 동일 피해를 소비하며 expression은 공통 helper를 사용한다.

check_final_001 통과 후 관측 on/off의 피해량 외 transaction snapshot/queue/runtime facts 전체 대조 assertion을 추가했다.
앞선 통과 결과는 pre-final이며 이후 최종 source에서 다시 검증한다.

## 최종 제품 검증

최종 digest e431c18332d1336ef4f0c0f7ef062ed7569c49875acf3d8a18e21ca32a456730에서 F02 3/3, authoring112/112, value_expr94/94, 전체1545/1545 통과. 전체 시험은 기본 병렬137.5초이며 단일 스레드로 제한하지 않았다.
check_final_002/clippy_final_001/format_final_001 exit0. 새 helper에도 기존 GameError 크기로 인한 result_large_err 경고가 보고되어 원문을 보존했다. 오류 타입 전면 변경은 범위 밖이며 자동 fix는 하지 않았다.
PC36 측정용 격리본은341개 current source byte와 동일하게 복사하고 기존 private probe 두 hook만 추가했다. 제품 source에 PC hook을 넣지 않았다.

## PC36/R05와 최종 인계

pc_build_final_001은 --no-run 빌드만이며 실행 PASS 증거로 세지 않는다. pc_final_001은 실제 ignored probe 1/1,
576행/36조건(성공31·typed 거부5)을 실행했다. 선행 D07-B와 gameplay/status/error code/error detail 차이0.
PC raw JSON을 작업 디스크에 저장하는 구간이 길었으나 probe 진행과 파일 증가를 확인했고 중단하지 않았다.
PC36과 R05 workload는 제품 검증과 빌드가 모두 끝난 뒤 순차 실행했다.
r05_build_final_001 빌드 후 r05_final_001 실행: raw exit101. concurrent peak9,049,037 bytes로 원 cap9,030,188보다18,849 초과.
원 로그의 유일한 main panic이 concurrent allocation guard assertion임을 확인했고 sample별 operations/retained,
event-heavy peak, repeatability와 parity는 모두 통과했다. 기존 정책에 따라 이 peak만 warning으로 분류했다.
R05 tool/baseline/입력 SHA·observer/schema·각 sample 수를 확인하고 원 실패를 그대로 보존했다.
최종 source341개 digest와 모든 fresh 실행 전후를 대조했다. 첫 D07-B 대비 누적 변경4개+추가1개이며 F01 수정은 유지한다.
인계 문서·세 기록·실행 도구·원자료를 정리한 뒤 새 target의 freeze 및 별도 verify에서 currentness/격리 정·역 복원을 확인한다.
F02 자체 검증만 완료 처리하며 F03 및 전체 D07-B/R06 완료는 선언하지 않는다.
