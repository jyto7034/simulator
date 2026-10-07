# U08 실제 전투 보강 기록 020

## 2026-10-06/020 — G3 실제 재구성·순열·저항 검증 회수

Astra 테스트 담당자의 G3는 최종 source438/b3f0af7e0243d70b9ce972ee0770c6798d2224c917ae8744f2f59f31ccb2e38e에서 carrier_module00111/11(19.447123s), lifecycle_module0013/3(6.050834s) PASS로 닫혔다. root는 최종438 actual source bytes/membership, 이전G2 대비 u08_carriers.rs/u08_lifecycle.rs 정확2파일 delta,10개 실행의 raw/hash와 실행 중 source 불변을 대조했다. 공통 carrier 준비/진입 helper 변경의 영향을 같은 최종 source에서 모두 재확인했다. 제품 변경은 없다.

동일 세 사본의6슬롯순열은 각각 실제 requested/HP174, 동일 -15/-10 패널티, base/certified 및 불변 정의 보존을 확인했다. 원형 AuthoredSkillEffect의 원본/추가/제거 세 generation에서는 actual skill HP vector가 [144]/[144,84]/[144], added_step release는0/1/0이었다. 추가 Instant step은73ms 지연이며 모두 첫 hit+200ms 뒤까지 scheduler를 진행해 제거의 실제 음성을 관찰했다. catalog 원본과 보유한 이전 fact는 그대로다. 원형은 별도 시제품 item tier, OnBattleStart 또는 U09 created closure가 아니다.

정상 EquipItem 및 Maintenance의 AwakenSkillFragment/SelectSkillFragmentForm을 거쳐 새 실제 battle에서 스킬/기본공격 HP는 기본144/38→형태교체100/38→무기교체144/12였다. 같은 SkillId라도 정확 form 조건만 휴면이 되고 실제 무기 조건은 기본공격에만 영향을 준다. 기본공격의 원 trigger/action 출처는 RuntimeFactSource에 유지되고 실제 무기 selector는 RelicNumericBinding.equipment에 있다. 기여 없는 경우 overlay 자체가 없으므로 배치된 실제 장비 slot UUID/정의 및 raw HP를 대조했다. 내부 catalog/admin ownership 준비를 정상 외부 획득 증거로 재표기하지 않으며 E009는 별도다.

Attack 태세의 raw144/210에 물리저항150 또는 마법저항100이 먼저 적용되고 기존 skill ModifyDamage+50%가 뒤에 적용되어 실제HP Physical85/126, Magic108/157이었다. 이후 정상 SetEngagementStance로 Defense로 바꿔 새 incoming hit를 관찰했다. 적 raw100에 일반 Defense10/MR5는90/95, Corrupt -5/-5는105였다. 실제 장비 ground-focus10% 감소는 각각81/85/94를 만들었다. 장비 modifier는 별도 equipment modifier_sources receipt가 없고 incoming_damage_modifiers로 합산되는 기존 계약이므로 실제 roster weapon 정의→Ground/Defense/DefenseFocus tactical snapshot→DamageFact의 unattributed 감소율→HP를 연결했다. skill modifier와 equipment modifier를 같은 출처라고 주장하지 않는다.

실패는 모두 보존했다. provider001은 실제 basic 무기 provider를 외부 source.equipment에서 찾은 테스트 관찰경로 오류로 terminal까지 기다린 뒤 unwrap했다. mitigation001은 기본 Attack 태세를 Defense로 가정했다. mitigation002는 Defense에서 두 번째 skill readiness를 기다려 terminal unwrap을 관찰했다. 실제 코드의 skill_start_disabled_reason는 DefenseStance를 반환하고 manual skill resolve가 target/range 이전에 거부한다. 이 원인은 코드로 확인했으며002 raw에서 직접 readiness를 관찰했다고 주장하지 않는다. outgoing Attack skill 이후 incoming Defense 순서로 fixture를 교정했다. 별도 진단 assertion 준비는 적용하지 않아 추가 재컴파일을 만들지 않았다. 기존 oracle/cap/제품의미는 바뀌지 않았다.

현재 Astra는 G5 읽기 사전 확인 중이다. 실제 nonempty host DetachSession/AttachSession과 current battle progress 보존(Continue나 entry rollback과 구분), RetreatBattle의 entry copy/slot/variant/material 보존과 실제 phase/counter 복귀, checksum을 재계산한 forged slot wire의 실제 Continue 거부 및 원본/전체 roots 보존을 다음 범위로 맞춘다. host Run snapshot에 존재하지 않는 relic field를 요구하지 않고 별도 정상 RelicState query를 사용한다. Retreat는 unfinished와 node별 counter로 허용되며 missing counter도 기본값을 쓰므로 함수 이름만 보고 abnormality 전용이라고 가정하지 않는다.

보조 Astra finalizer 작성은 public019의 별도 신규 도구 영역에서 계속된다. 준비 도구는 기능실행/완료 증거가 아니며 기존45harness와 별개다. G7 이후 새 실행용 manifest는 실제 전이 의존 파일도 포함하고 Sol의 실행전후 동일 입력 확인과 결속한다. G3/G5/G6/G7 후 current 기능통합, functional writer/runner0 후 PC/R05, evidence→impact→adaptation→coverage→자체검증→PLAN/NOTES→exact freeze/정역복원→비참여 Astra 전체 U08 감사 순서를 유지한다. Sol 제품/통합/성능은 HOLD이고 EXPS/STATE 문서만 갱신한다. 전체 U08/16.1/16.2 완료가 아니다.
