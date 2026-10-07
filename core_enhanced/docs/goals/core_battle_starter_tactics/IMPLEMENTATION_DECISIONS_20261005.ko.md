# U06 GEX-10 구현 선택 — 기존 정책의 상세화

## BASIC overlay 소비 경계

- 사용자 직접 결정: 순수 단일/영역 기본 치유, 명시 무기 profile, 기존 공명/사용 수명 및 승인된 U06 범위.
- AI 위임 결정: 기존 PE-CBST-05의 Instant 치유만 지원하는 경계를 overlay에도 적용한다. Instant overlay는 획득 범위만 변경하며 Heal 양/footprint/아군 의미를 유지한다. 피해용 Projectile/TileArea overlay는 접수 자격과 효과 시작 양쪽에서 명시 거절한다. 1회 overlay는 성공한 기본 치유 방출에서도 기존 AttackReleased 수명으로 소비한다.
- 이유: 기본 정의의 strict 검증을 runtime overlay가 우회할 수 없게 하고 범위 교체와 적대 damage/proc 의미를 분리한다. 치유량을 overlay 피해로 변환하지 않는다.
- 영향: 기본 Heal의 보관 footprint 비용, sustained 시작/조회 자격, 1회 overlay의 실제 종료. 기존 Damage overlay 의미는 유지한다. 실패/clone/순수 HP·원본 복원과 비용 경계를 집중 검증한다.
- 위임 근거: 현행 연속 재개 요청의 기존 합의 내 구현 판단, WORKING_POLICY PE-CBST-05/18/19 및 CODE_REVIEW TD01/08. 정책 요소·수락 조건·Goal/Phase 변경 없음. P2G 동결 산출물 재발행 없이 GEX 상세화로 기록한다.

## STARTER authored 초기값과 지급

사용자 직접 결정은 PE-CBST-06의 여섯 역할·무기+active 세트/상호 이전과 PE-CBST-02의 보유10이다. 아래 배정/숫자는 현행 위임에 따른 AI 초기 콘텐츠 결정이며 사용자 확정 밸런스로 표시하지 않는다.
- field_medic/hq_field_medic: 단일 Staff(1800ms/준비200, 기본8+주문력100%, 주변2타일), active 단일60 회복.
- maintenance_tech/hq_maintenance_aide: 영역 Staff(2200ms/250, 기본4+주문력50%, 주변2타일의 선택대상 주변1타일), active 선택영역25 회복. 두 Staff는 주문력10을 제공한다.
- scout_runner/hq_recovery_runner: Gun(1500ms/200, 전방3타일, projectile6000), 아드레날린8초 공격력30%/주기25%단축.
- records_clerk/hq_records_observer: 기존 Sword(1500ms/200), 버서커10초 공격력20%/주기20%단축, 기본흡혈30%/한 방출 회복상한20.
- breach_guard/hq_security_guard: Shield(1800ms/250, 기본저지3), 최후의전선8초 받은피해50%감소.
- containment_trainee/hq_containment_aide: Sword계열 대검(2000ms/300), 돌파 전방3칸 Physical60 + 자기보호막40/3초.
- 모든 active는 공명최대100, 기존 U05 초기1/5·자동획득·사용수명을 사용한다. 지속 효과 전부 ActionAndEffects로 사용 종료까지 잠근다. 즉시 회복은 Action이다.
- 이유: 역할별 순수치유/공격/방어를 실제 RON으로 구분하며 기존 주기·수치 체계를 사용한다. 공격속도 증가는 가역 AttackIntervalMs 비율 감소로 표현하며 표시값은 정확한 주기 감소율이다. 새로운 weapon archetype/직원 고유 active는 없다.
- 흡혈은 각 유한 출처의 실제HP손실×비율 내림과 작성 상한, 받은피해감소는 기존 합산/clamp0..100 파이프라인이다. 정식 피해 snapshot의 기존 시점/독립 방출 수명을 보존한다.
- 영향: 새5무기/6파편/6스킬, 시작/채용12정의, 일반장비보상에서 훈련무기 제외, 정상 grant/equip와 strict 참조 검사. 기존 직원 identity/프로필/maintenance archive_scanner는 보존한다. legacy 시험용 baseline 파편은 명시 입력에 남기며 공식 세트에서는 제거한다.
- 위임 근거: 현행 재개 요청·PE-CBST-04/06·TD03/08. 정책요소/수락조건/Phase 변경 없이 초기 작성값을 GEX에서 검증하고 조정한다.

## 기본 Damage 저장 표현 보존 교정

GEX 자체 검증에서 새 action 필드의 기본 Damage 직렬화가 기존 strict V11 canonical checksum을 바꾸는 결함을 재현했다(save_damage_compat_repro_001). 기존 저장 owner와 지원 포맷을 유지하는 PE-CBST-18/TD08 안에서 AI가 기본값만 생략하는 serializer 교정을 선택했다. Heal 값은 그대로 명시 저장한다. 버전 변경·암묵 변환·checksum 무시는 추가하지 않는다. 이전 V7 비교에 Damage를 보충하던 시험 adapter도 제거해 원래 전체 값 비교를 복원했다. 사용자 직접 결정인 저장 계약을 보존하는 구현 교정이며 새 정책 선택은 아니다.

## F01 교정 결정 — 기존 PE-CBST-05 구현

독립 감사001은 zero-effective Heal의 자동자격과 V15 query 결함을 확인했다. 사용자 직접 신규 결정이 아니라 연속 요청의 in-scope 교정 위임으로, 기존 부족HP·회복가능 대상 의미를 현재 `flat + SP*percent/100`의 양수 결과에 연결한다. allzero 계수 자체는 동결 정책상 반드시 거절할 작성 오류가 아니므로 schema 수용을 유지하고 비유효 자동행동만 막는다. 시작·공개조회·방출은 동일 계산을 사용하며 정수반올림과 준비후스탯변화도 적용한다. 새로운 상태/유한권한/직렬화/성능상한/정책요소는 도입하지 않는다. 첫 감사·target은 원본 보존하고 재감사 전 U07을 시작하지 않는다.
