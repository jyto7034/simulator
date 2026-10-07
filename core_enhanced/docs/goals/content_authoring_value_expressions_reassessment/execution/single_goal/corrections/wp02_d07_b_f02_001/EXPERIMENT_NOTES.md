# D07-B F02 현재 판단

공통 함수는 RuntimeAmount 배열에만 한정하고 기존 site value_expr_evaluated_amounts를 유지한다.
관측 off/catalog 미등록 legacy는 기존 분기 밖에서 새 배열/평가를 만들지 않는다.
정확 배열 admission 한도는 전체 effect 완료 한도가 아니다. 뒤의 평가/기록 할당을 별도 구별한다.
실제 성공 operation의 누적 admission을 cfg(test) 관찰로 기록하며 production 정책/한도는 바꾸지 않는다.
F02/C02 이외 동적 replay·terminal 수명 등 F03 검증을 이번 결과로 완료 처리하지 않는다.


최종 판단: C02의 두 검증 층이 모두 닫혔다. helper의 allocation 전 거부와 actual transaction의 정확 site/기존 record 보존을 확인했다.
관측 on/off 동일성은 HP뿐 아니라 transaction snapshot·queue·runtime facts를 함께 비교했다.
PC36 관측 off 보존과 신규 관측 on 시험의 책임은 구별한다. R05 peak 경고는 변경된 정책 적용이며 원 guard가 PASS였다는 뜻이 아니다.
Clippy exit0이며 새 helper의 GameError result_large_err를 포함한 경고는 원문에 남겼다. 오류 타입 전체 변경은 범위를 벗어나므로 하지 않았다.
다음은 사본 F02/C02 단계검토, 그 뒤 사용자 재개 요청이 있으면 F03의 첫 의미 있는 구간을 진행한다.
