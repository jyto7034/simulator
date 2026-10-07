# U08 G4 침식 실패 원자성 회수 — 2026-10-06/017

연속 실행을 유지한다. G4 Astra writer/runner는 닫혔고 현재 G2 읽기 사전 확인 중이다. Sol 제품/통합/성능과 도구 입력은 HOLD다. root는 계획·문서만 담당한다.

정상 immutable host RuntimeSafetySettings로 구축한 Core에 준비한 시험 상태를 넣고, 외부 native::run 주입 없이 실제 core.execute(CorruptRelic)를 수행했다. 양성 대조는 장착한 Normal 사본+촉매1이다. retained seed30,689의 한도-1, 비용/variant 변경 이후 relic_query_snapshot32,587 및 relic_not_sellable_projection32,603의 각 한도-1에서 정확 fault code/site/requested/limit와 승인된 prefix를 확인했다. 각 거절을 두 번 반복하고 전체 world roots·UUID·사본/slot/variant·촉매·SavePoint/Continue 원본이 불변임을 확인했다. full Corrupt+Continue 누적 exact1,468,942에서는 동일 사본/slot을 유지한 Corrupt와 촉매0으로 성공했다. 관측 boundary는 enforcement 위치 검증이며 독립 heap 크기 증명이 아니다.

촉매0+실제 기타자산(enkephalin/equipment_dust/legacyfragment_dust) 충분 조건은 InvalidAction·무소비이며 촉매1 양성에서 정확히1만 소비했다. 이미Corrupt/없는copy/침식variant없음은 PreviewRelicCorruption과 CorruptRelic 모두 InvalidAction으로 거절하고 전체 roots가 보존됐다. preview의 CoreCommon query와 실행 CoreDomain route를 구별했다. 부활촉매는 현재 구현에 존재하지 않으므로 실제 두촉매 교차시험은 U17 seam 미실행으로 남고, frozen11.2 혼용금지 정책과 별도 침식필드 계약을 유지한다.

실제 결과는 atomic0011/1(113.746762s,source7edc5d8c), paired preview 보강 뒤 assets0011/1(85.576982s), 최종 affected module00113/13(74.783371s) PASS다. 최종source438/4f289cee78777f175b2046b052c0f37170d2bda4834560b3700b80e90b27423b에서 새2개를 포함한13개가 모두 실행됐다. G4 전체 변경은 src/game/world/tests/u08_relics.rs 한 파일뿐이며 기존 roots helper를 더 넓게 비교했다. root는3raw 및 최종438 actual source bytes를 대조했다. 실패나 제품 변경은 없었다.

다음 G2는 외부 RON full loader의 schema/주소/참조/variant/unsupported recipe 진단과 실제 변경/삭제 catalog를 재validate한 generation의 nonempty 저장 복원 거절이다. 과거 checksum 문자열 변조와 구분하고 E009 export·이전 실패·oracle는 보존한다. G3의 시작 효과는 원형에 있는 실제 효과 추가/제거의 재평가이며 U09 created-effect closure를 선구현하지 않는다. G2/G3/G5/G6/G7 뒤 현재 기능통합→functional0 뒤 PC/R05→증거최종화/동결/독립전체감사를 수행한다. 현재 U08 완료나16.1/16.2 PASS가 아니다.
