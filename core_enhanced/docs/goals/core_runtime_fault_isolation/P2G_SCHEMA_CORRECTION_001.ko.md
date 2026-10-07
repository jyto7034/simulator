# Canonical 형식 교정

P2G-40 표준 reader 진입 전 검사에서 raw exit1 `MEMBER_STATUS`가 발생했다.
원인은 `Intrinsic Status` 표기와 scalar acceptance였다. 표준은 `Intrinsic status`와 배열을 요구한다.
[원래 실패 로그](evidence/p2g_schema_precheck_001.log)를 보존했다. 도구·원래 artifact를 수정하지 않는다.

[Change/declaration](P2G_SCHEMA_CHANGE_001.json)은 기존 Freeze001과 파생 basis를 stale로 판정한다.
정책 의미의 영향 집합은 empty다. canonical representation 결함은 P2G-10 owner에서12개 successor
revision으로 교정하며 결정 Ref와 각 acceptance 원문은 그대로다. 원래12개 결정을 다시 사용자 답변으로
만들거나 새로운 정책 선택을 추가하지 않았다. full source 재실행 대신 정확한 unchanged bytes를 확인하고
P2G-20 current binding, P2G-30 successor Freeze를 새로 발행한다. O10→O20→O30를 모두 해결해야
후속 계획을 채택한다. 아직 Phase Plan/Goal/Handoff/제품 구현은 없으므로 그 단계의 repair obligation은 없다.
