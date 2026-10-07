# R06 사전 검증 교정 설계 — U08 내부 회귀 교정

원래 R06 row0/IVA-CAVER-01의 unit 거부 계약을 구현에 다시 맞춘다. P2G 정책·수용 문장·선행 감사 원본을 수정하는 작업이 아니다. 이번 실제 결함은 외부 참조를 포함한 식에서 이미 알려진 authored payload unit이 사전 검증되지 않는 것이다. 능력치·HP·cadence 관련 발견은 별도의 관측 공백이며 제품 결함으로 단정하지 않는다.

실제 경로는 value_expr/compile.rs의 Compiler::exit_aggregate → catalog.rs의 validate_result_contract 및 can_evaluate_without_external_context → eval.rs의 read_value/require_unit이다. AuthoredMember.value는 TypedInteger다. 지금 compiler는 member 식의 결과 unit으로 reduction 결과를 정하고, catalog는 외부 참조 하나만 있어도 식 전체의 정적 evaluate를 건너뛴다. 이때 authored payload와 직접 Read(AggregateMember { unit })의 불일치가 runtime까지 남는다.

수정 책임은 compile.rs 한 파일이다. 각 Aggregate의 member_segment에서 직접 CompiledOp::Read(ValueRef::AggregateMember { unit })만 확인하고 자신의 AuthoredMembers와 결속한다. nested child의 별도 segment를 부모 payload에 적용하지 않는다. member 최종 결과 unit을 payload unit으로 오인하지 않는다. Scalar payload를 Ratio의 분모로 읽어 Magnitude 결과를 만드는 유효 식을 보존한다. Const/SourceAttack/child-only처럼 payload를 읽지 않는 member는 사용하지 않는 payload unit 때문에 거부하지 않는다. empty source의 기존 reduction 의미와 runtime resolved-member/Stat/HP context 진단을 유지한다. Resource unit은 ResourceId 전체 값의 일치가 필요하다.

권장 구현은 witness operation index를 이용한 bounded O(member ops + authored members) 검사다. 성공 경로에 새 heap collection, ResourceId clone, AST 재귀 순회, plan metadata/layout을 추가하지 않는다. 실제 새 scan 및 ResourceId text 비교 비용은 수행 전에 checked 연산으로 기존 max_compile_work에 계상한다. runtime max_work/static preprocessing/retained storage 공식과 한도는 유지한다. valid 입력의 compile_work 증가 및 한계 근처 BudgetExceeded 가능성은 숨기지 않고 exact/one-below 경계로 검사한다. 기존 필수 회귀가 cap이나 기대값 차이로 실패하면 원시 차이와 원인을 반환하며 한도 상향·성공 기대 완화로 처리하지 않는다. 단순 타입/borrow/scoped fmt 교정은 이 범위에서 Sol이 자율 수행한다.

공개 실패는 기존 Compile/UnitMismatch 및 pack/catalog/file/object/action/effect/field/expected/actual 진단 경계를 유지한다. catalog의 전체 정적 평가 gate를 강제로 실행하거나 외부 값에 임의 기본값을 넣지 않는다. 정상 plan op/source/member/semantic fingerprint는 바꾸지 않는 검증 교정이며, 실제 positive 재실행으로 이를 확인한다. 잘못된 후보가 token으로 게시되지 않고 기존 published/application 상태가 보존되는 관측을 Astra가 작성한다.

시험 담당은 독립 기대를 가진 최소 묶음을 설계한다. public operator unit 및 Points/Milliseconds negative, static/dynamic Sum/Max와 정상·nested·unused·Scalar Ratio·Resource/empty/budget controls, SpellPower만 변경한 실제 식 결과와 Physical/Magic 정체성, Attack100을 만든 각 실제 기여의 제거, Max HP read, zero points/base interval의 공개 거부, AttackSpeedPoints만 바꾼 basic cadence와 spell/area/projectile 시점 비교를 다룬다. 기존 유효 검증을 재사용하고 같은 이름의 시험이 모든 조건을 검사했다고 추정하지 않는다. 테스트는 별도 파일 인계 후 작성하며 Sol은 fixture/oracle을 수정하지 않는다.

순서는 실제006 abort/source unchanged 확인 → Sol 단일 제품 파일 교정·작은 compile → 제품 writer0 → Astra 승인 시험 묶음·scoped format/집중 실행·일반 fixture 교정 → source/입력 writer0 및 새 source 인수 → harness005/통합007 → 최종 evidence/impact/adaptation/coverage/자체검증/동결/복원/독립 감사다. 같은 파일 동시 작성 및 실행 중 입력 편집은 금지한다. 현재 문서는 제품 구현 결과나 시험 통과를 의미하지 않는다.
