# U08 실제 carrier 실패와 교정 결정 — 2026-10-06/010

관측 off literal 연결은 relic_carrier_literal002에서 1/1 PASS다. 동일 SkillId의 두 직원이 서로 다른 fragment/relic copy로 실제 144/210 피해와 별도 immutable binding을 보존했다. 이 시험은 내부 Admin 준비 이후 정상 command/scheduler 소비이며 정상 획득의 증거는 앞선 E1과 구분한다.

이어 relic_carrier_stage2001은 실제 5개 중 literal 1 PASS / carrier 4 FAIL이었다. Snapshot projectile, GroundFixed, Follow의 실제 피해는 기대210 대신100, Live Source.Attack 생존 양성은 기대78 대신12였다. 원 실패와 기대값은 보존한다. root·Astra·Sol이 실제 코드를 읽어 ValueExprDeliveryCapture::rebind_consumption_source가 incoming source를 복사하고 authored만 복원해 원 capture의 private relic_numeric 슬롯을 잃는 공통 제품 누락을 확인했다. release 결속과 consumption 적용 모두 관측 gate 밖이므로 observation-off 자체를 원인으로 취급하지 않는다.

Sol에게 core/value_expr_runtime.rs 및 battle/runtime_facts.rs 두 제품 파일과 scoped formatting을 위임했다. incoming action/delivery/독립 생성 및 reinput 정보는 보존하고 numeric 슬롯만 원 capture의 정확한 Some/None로 합성한다. incoming Some을 capture None 행에 전파해서는 안 된다. 기존 action/delivery 및 generation/provider 권한 검사는 유지하며 binding/plan/frame Arc를 공유한다. None/None에는 추가 cold wrapper를 만들지 않는다. 새 wrapper가 필요하면 allocation 전에 admission하고 합성 source의 retained extent로 cache를 다시 계산한다. 전체 rebind 사전 bound는 기존 capture owner와 incoming source의 합으로 보수적이며 cumulative 한도·overflow·실패의 무환급 의미를 유지한다. 이는 기존 TD04 수명 계약의 구현 교정으로 정책·공개 API·한도 변경이 아니다.

이 기록 작성 시 Sol 제품 편집/scoped fmt만 진행 중이며 Astra writer/runner는 0이다. root는 문서와 읽기 검토만 수행한다. 제품 handoff 후 Astra가 기존 relic_amount.rs에 두 private rebind 검증을 추가한다. 혼합 selector match/miss의 capture Some/None, incoming Some/None와 다른 independent/reinput, 정확 Arc·authored·새 delivery·원본 불변을 확인한다. 전체 실제 rebind admission의 exact cap/one-short typed 거부와 성공 prefix, 입력 불변도 확인한다. 이 private 검증은 normal execution 또는 heap bound 증명으로 확대하지 않는다. 기존 world 5개 시험은 기대값과 입력을 유지해 재실행하고 관련 binding/selector 및 resource_lifetimes released projectile/fixed-area 회귀를 확인한다. 이후 승인된 실제 basic melee/ranged·auto/triggered·windup 취소를 이어간다.

마지막으로 공개 release CLI와 통합·성능 검증 도구는 Astra가 현재 U08 입력으로 준비하고 Sol이 실행한다. U07-M 도구의 과거 label/path/cache를 그대로 재실행하지 않는다. fresh release build는 모든 공개 소비보다 앞서야 한다. R05 준비3파일과 dependency binder 및 current U08 통합 도구의 누락을 실제 확인했다. 기능 실행과 성능 측정은 겹치지 않는다.

아래 receipt/source는 각 과거 실행 경계이며 열린 제품 교정 이후 live source를 대신하지 않는다. U08은 GEX10 partial이다. 전체 검증 및 evidence→impact→adaptation→coverage→자체검증→정확 동결/복원→비참여 Astra 전체 독립 감사가 남아 있다. HANDOFF와 과거 실패·완료 증거를 소급 수정하지 않는다.
