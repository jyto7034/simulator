# F03 C04/C05 현재 판단

템플릿 두 개는 직전 사용자 질문 때 원문과 실제 산출물을 대조했으며 이번 재개에도 같은 기준을 적용한다.
스칼라 공격력은 직접 stats 변경, typed resource는 기존 operations, 완료 result는 실제 선행 effect 실행으로 변화시킨다.
Snapshot carrier는 release 때 만들어 consumption에 전달한다. 소비 전에 다시 capture해서 정상처럼 보이는 시험을 만들지 않는다.
record/frame/trace는 실제 executor가 만들고 시험이 조립하지 않는다. query 후 state와 반복 출력·usage도 대조한다.
private record encoder 검증은 selector/권한/공개 dynamic 지원이나 terminal 수명까지 입증하지 않는다.
생산 코드가 그대로라면 PC36/R05는 직전 동결 원자료와 source 적용성을 대조한다. 새 비용 정책/상한은 만들지 않는다.

기존 test helper의 build 후 resource_data 주입은 동적 Resource 식/Resources effect의 검증보다 늦었다.
with_resources를 build 전에 연결해 정상 compiler 검증으로 확장한다. 검증 우회나 production 공개 차단 해제 대신 실제 fixture 준비 순서를 교정한다.

release carrier와 runtime_fact_source는 둘 다 같은 delivery에 결속되어야 한다. actual projectile caller는 scoped replace/restore로 이 계약을 보장한다. 새 시험도 이 caller 계약을 따른다. 서로 다른 source를 projection에서 합쳐주는 production 완화는 하지 않는다.

현재 다섯 시험은 실제 소비된 amount와 기록 replay의 동적 입력 불변성을 입증한다. completed result의 나중 변경은 private fixture에서 수치 관측만 일시 정지하고 실제 damage/settlement를 실행했다. selector·기록 교체·관측 전환 계약을 검증했다는 의미로 확대하지 않는다.
제품 밖 시험만 바뀌었으므로 선행 PC36/R05 재사용이 적용된다. F03 나머지에서 실제 소유권/admission 교정이 생기면 그 target에서는 재측정해야 한다.
