# U08 집중20 회수 및 D 첫 수명주기 — 2026-10-06/004

공개13/조회owner3/Eventowner4가 각 실제 raw와 receipt에서 통과했다. 최종 공개bc002와 Eventowner003의 source424/2cfcdfe…는 불변이다. queryowner003의 f23454f… 이후 전체 source delta는 u08_event_owner.rs 단일 fixture 교정이며 query 시험/제품에 비영향으로 재사용한다. EVIDENCE.json은 성공뿐 아니라 선행 query002/Event001/Event002 실패도 원본 SHA로 연결한다.

제품 실패는 Sol, 시험 경계/fixture 실패는 Astra가 각각 교정했다. root는 code-grounded 설계·원인 분류·file lock·실제 diff/SHA·receipt 확인을 수행했다. 중첩 native controller 안의 임시 diagnostic을 공개 fault로 오인하지 않도록 정상 공개 closed scope 시험을 별도로 추가했고, Event 완료 이력을 현재 층에만 보존하는 수명 결정을 별도 publication으로 남겼다. 사용자의 기대를 낮추거나 한도를 올리지 않았다.

마지막 runner가 끝난 뒤 첫 D 시험 작성을 시작했다. 허용 파일은 신규 tests/u08_relic_public/lifecycle.rs, integration root, 필요한 최소 support helper와 독립 tests/fixtures/u08/lifecycle001/{facility_templates.ron,node_assignment_profiles.ron}다. 기존 public001 generation/catalog는 변경하지 않는다. 독립 authored map에서 정상 Event 진입과 public Continue를 검증하며 transfer 및 normal StartNewGame reset을 더한다. 제품·나머지 시험은 HOLD이고 새 실행은 아직 승인하지 않았다.

DOCUMENTS_BEFORE는 쓰기 전 PLAN/NOTES 실제 원본이다. 과거 source/failure/audit/frozen P2G/HANDOFF는 수정하지 않는다. 전체 U08의 실제 carrier/lifecycle/trial/CLI·필수 기능/성능·증거선정/impact/adaptation/coverage·동결/복원검증·독립감사는 남아 있다. Core-first 연속 작업을 유지하며 이 국소 경계에서 종료하지 않는다.
