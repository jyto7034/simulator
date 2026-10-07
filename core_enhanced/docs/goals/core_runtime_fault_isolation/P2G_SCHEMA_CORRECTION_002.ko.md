# Decision identity 교정

P2G-50 검증기 소스 확인에서 Covered Decision Ref parser가 `DR-` prefix를 사용함을 확인했다.
초기 `DEC-` identity로는 정확한 mapping을 표현할 수 없어 P2G-10에서 동일 의미의 Decision successor12개를
발행한다. 표준 도구와 과거 artifacts는 바꾸지 않는다. 기존 Choice/acceptance/Phase 구조의 의미는 같다.

[Change002/declaration](P2G_SCHEMA_CHANGE_002.json)이 Freeze002와 Plan001을 stale로 판정하며
O10→O20→O30→O40 교정을 모두 요구한다. Change001은 해소된 상태로 계속 적용한다.
Goal/Handoff/제품 구현은 아직 없어 해당 단계 교정 의무는 없다. 새 질문·사용자 답변은 생성하지 않았다.
