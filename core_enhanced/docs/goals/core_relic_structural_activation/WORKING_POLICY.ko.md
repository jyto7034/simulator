# U09 유한 활성·속성 변형과 구조형 공식 사례

Document Identity: `WPD-CRSA-20261007` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_relic_structural_activation` · Current Working Revision: `WR-CRSA-006`.

**2026-10-07 현재 변경 인수 완료:** [CHG/COD-CRSA-001](POST_FREEZE_CHANGE_001.json)의 O10→60을 모두 수리했다. 현재 chain은 `WR-CRSA-006 → RB-CRSA-002 → FPF-CRSA-002 → PP-CRSA-002/PVR-CRSA-002 → GS-CRSA-003 → HA-CRSA-002`이며 [현재 인계 결과](P2G_60_RESULT_002.json)가 exact refs를 소유한다. PE04/08/09만 successor이고 나머지14정책은 동일하다. 아래 세 정책은 새 의미를 반영했으며 이후 나오는 옛 단계완료 설명은 predecessor 이력이다. 기존438 source/58 evidence와 유효 설계를 재사용해 GEX 인수를 이어간다. 제품·시험 source mutation/실행은 아직 없다.

[초기 입력·승인 범위](P2G_00_INITIAL_001.json), [현재 정책·결정](P2G_10_POLICY_001.json), [P2G-10 결과·위임 판단·한정 사실](P2G_10_RESULT_002.json)가 exact basis를 소유한다. 17정책/34문장·미결 질문0이며 정책 의미와 수용문장을 유지한 정식 코드 검토를 완료했다. [P2G-20 결과](P2G_20_RESULT_001.json)와 [코드 검토](CODE_REVIEW_001.ko.md)가 현재17개 code_reviewed revision 및 source 결속을 소유한다. [P2G-30 결과](P2G_30_RESULT_001.json)의 정책 동결 `FPF-CRSA-001`을 현재 planning basis로 선택했다. [P2G-40 결과](P2G_40_RESULT_001.json)의 `PP-CRSA-001`과 passed receipt를 현재 실행 계획으로 선택했다. [P2G-50 결과](P2G_50_RESULT_001.json)의 `GS-CRSA-001`과 [P2G-60 인계](P2G_60_HANDOFF_001.json) `HA-CRSA-001`을 현재 basis로 선택했다. 제품 구현·신규 시험은 아직 미수행이며 별도 연속 요청 권한으로 GEX-00 인수를 진행한다.

선행 U08은 [완료 인수](../../roadmaps/core_first/execution/u08_closure_20261007_001/RESULT.ko.md)의 전체 독립 Complete/High/open0와 두 현황문서 비영향 게시 delta로 인수한다. 동결·감사 원본은 보존한다. Root Astra/High가 계획·설계·조정, 기존 Sol/xhigh가 제품·통합, 별도 Astra/High가 시험을 맡는다. 현재 제품·시험 writer0/runner0이며 구체적 GEX 묶음 전 변경하지 않는다. 최종 독립 감사자는 U09 비참여자로 배정한다.

위임 판단: H04 §5의 명시적인 범위 내 gameplay 세부 결정 권한과 최신 연속 요청을 적용했다. typed arithmetic·단일 대상 고정/거리·승려 HpLost 기준은 AI의 위임 결정이며 사용자 직접 답변이 아니다. 기존 구조·수치 합의를 유지하고 공식 미정 밸런스·침식 패널티를 시험값으로 확정하지 않는다. source/code 사실 확인의 범위와 exact refs는 결과의 finding_triage/policy_source_evidence를 따른다.

## PE-CRSA-01 — 효과 단위 조건과 정체성

태그와 명시 typed property는 같은 실제 효과에서 AND로 판정한다. 정확한 파편·장비·형태 조건은 별도 identity 조건으로 함께 적용하며 다른 효과의 태그 합집합이나 일반 태그로 대신하지 않는다. 조건이 없으면 사본은 슬롯에 휴면 상태로 남고 조건 회복 시 재평가한다.

근거: relics 조건과 비활성;H04§4.2/A15-01;U09 C05. 판단: 효과 주소와 개인 사본의 기존 정체성을 유지한다.

- 태그가 서로 나뉜 두 효과는 AND를 충족하지 않지만 같은 효과의 태그·속성·정확 ID/form이 모두 맞으면 활성이다.
- 생성 효과에도 같은 선택 규칙을 적용하고 ID/form mismatch·property 부재는 설명 가능한 휴면 또는 부적격 결과로 나타난다.

## PE-CRSA-02 — seeded 최소 활성과 재계산

현재 무기·파편의 실제 시작 효과와 무조건 부여 효과에서 출발해 더 활성화할 효과가 없을 때까지 최소 활성 집합을 구한다. 긍정 A/B 순환은 입력 오류가 아니라 seed 없을 때 휴면이다. 시작점 제거 뒤 과거 활성 효과만으로 스스로 유지하지 않는다. 슬롯 순서·사본 UUID 정렬이 활성 결과를 바꾸지 않는다.

근거: relics 조건과 비활성;H04§4.2/A15-02;T28. 판단: 현재 구성을 권위로 삼고 이전 plan은 현재 활성의 seed로 쓰지 않는다.

- 무seed A/B는 둘 다 휴면, 각 실제 seed의 추가로 도달 가능한 효과만 활성이고 seed 제거 뒤 다시 휴면이다.
- 같은 사본 집합의 모든 장착 순서에서 최종 효과·값·휴면 이유가 같으며 자기 부트스트랩과 임의 반복횟수에 의한 부분 성공이 없다.

## PE-CRSA-03 — 유한 grant와 부정 의존 거절

새 grant는 공개된 실행 가능한 비반응 primitive/유한 Flow로 구성한다. 한 구성에서 생성 가능한 효과 identity 집합과 의존 평가가 유한함을 게시 전에 입증한다. 부정 순환·끝없이 새 identity를 만드는 재귀·자기 변경 때문에 안정된 해석을 입증할 수 없는 authoring은 구체적 typed 진단으로 거절한다. 단순 긍정 상호조건을 일괄 거절하지 않는다. U10 반응 subscriber를 이 단계에서 지원한다고 표시하지 않는다.

근거: H04§4.2/A15-01,02,03;R15§3,7;T28. 판단: 유한성은 안전한 게시 계약이며 gameplay 공통 발동 상한과 구분한다.

- 새 폭발 효과를 유물 A가 부여하고 B가 그 실제 생성 효과를 선택·강화해 정상 시전 피해로 나타난다.
- NOT cycle·재귀 grant·입증 불가능한 구조 의존은 부분 게시 없이 거절하고, 유효한 긍정 순환은 seed 의미대로 종료한다.

## PE-CRSA-04 — 공개 typed property와 개인 방어막 지원 일치

작성자가 선택·변경할 속성은 실제 executor와 연결된 명시 typed capability로 공개한다. 이번 범위는 기존 damage amount와 생성 효과, 효과별 피해 footprint, 지원 대상 구조, 포큐버스 개인 방어막 생성량, regular basic 피해·속도 값 및 필요한 구조 recipe다. 기존 R12 공용 보호 기능은 보존하고 개인 방어막과 의미를 혼동하지 않는다. effect와 carrier/action의 범위를 구분하고 본편 전용 임의 필드나 문자열 경로를 public write 권한으로 노출하지 않는다. 변형 뒤 없어진 속성을 여전히 지원한다고 표시하지 않는다.

근거: PIS-CRSA-002/CH-IN01–08; DR-CRSA-04-002. 새 개인 보호 구조의 공개 capability를 실제 consumer와 일치시킨다. 기존 공용 API를 의미 변경하지 않는다.

- 공개 descriptor·validation·explain·실제 실행이 같은 속성과 타입을 사용하며 없는/비공개/단위 불일치 property는 명시 거절된다.
- 단일 지원에서 제거된 area 속성에 반경 modifier가 적용되거나 대상 선택 사거리·별도 회복이 함께 늘어나지 않는다.

## PE-CRSA-05 — 수치 연산과 Replace 충돌

같은 typed 수치 property의 지원된 Add는 같은 단위로 합산하고 Scale의 같은 종류 증분은 합산하여 한 번 적용한다. Replace가 허용된 property는 대체 기준값에 Add 후 Scale을 적용한다. 서로 다른 Replace 또는 구조 write set 충돌은 원자 거절하며 동일 값·동일 의미의 중복 Replace는 한 결과로 합치고 기여 출처는 모두 남긴다. 비수치 구조는 허용 recipe만 변경한다. checked 계산과 해당 정수 단위의 한 번 절삭을 사용하고 overflow·음수 금지 범위·단위 오류를 조용히 clamp하지 않는다. 기존 damage flat→percent 의미와 다른 전투 공식은 보존한다.

근거: H04§4.2/A15-03;R15 typed Add/Scale/Replace;U08 numeric. 판단: H04의 비피해 산술 위임에 따라 property별 동일 단위 연산을 명시한다. 전투 저항·감쇄 전체의 산술을 교체하지 않는다.

- 고정 독립 oracle와 순서 permutation으로 Add/Scale/동일 Replace의 결과가 같고 서로 다른 Replace는 어떤 UUID/장착 순서에서도 typed 거절된다.
- 타입·단위·범위/overflow 실패에서 이전 loadout·plan·저장·자원이 보존되고 유물 없는 기존 수치 결과는 같다.

## PE-CRSA-06 — 개인 불변 plan과 실제 소비

활성 집합과 변형 결과는 원형 정의와 구분되는 개인 불변 effective plan이다. 정상 지원 소비자들의 readiness/preview/명령·기본 시전, paused 접수와 queued/released projectile/area/지연 실행이 같은 선택된 plan과 기존 Snapshot/Live 의미를 사용한다. 같은 SkillId의 다른 직원·사본·형태·콘텐츠 세대가 캐시를 공유해 서로 영향을 주지 않는다. 현재 지원되지 않는 직원 active Auto API를 새 수용 전제로 만들지 않는다.

근거: H04 A14-04/A15-01/X03;U08 audit013 applicability;T13/T27. 판단: 새 구조 plan도 U08에서 완료한 supported-consumer 및 released-owner 계약을 소비한다.

- 같은 SkillId를 서로 다른 유물/형태로 쓰는 직원의 실제 즉시·지연 결과와 query가 각각의 독립 oracle와 일치하고 공유 원형은 불변이다.
- Pause 접수→Resume와 plan 교체·source 종료 뒤 남은 허용 payload가 선택된 소유/평가 시점을 보존하며 stale generation/provider는 거절된다.

## PE-CRSA-07 — 구조 recipe의 write와 preserve

구조 recipe는 변경할 효과·속성과 보존할 carrier/대상 자격/원 출처/채널·action/비용·시간/권한을 명시한다. 여러 관련 효과를 함께 바꿔야 유효하면 원자 변환하고 최종 graph·참조·typed 값·실행 가능성을 다시 검증한다. footprint 변경이 공유 carrier의 다른 효과를 묵시 변경하지 않는다. source는 원 무기/파편/장비와 유물 기여를 구분한다.

근거: H04 A15-03/X03;T27;R15. 판단: 수정 대상을 효과와 공유 실행 owner로 구분해 우연한 광역 패치를 막는다.

- 손상된 참조·충돌 bundle·부적격 구조는 게시와 장착/전환을 부분 적용하지 않고 거절한다.
- 피해 범위만 바꾼 합성 스킬에서 자기 회복·다른 effect·carrier·채널·HP 비용·출처는 보존되고 최종 query와 실제 실행이 일치한다.

## PE-CRSA-08 — 단일 지원의 입력과 효과별 거리·수명

범위 지원을 단일 지원으로 바꾸는 recipe는 원래 자격과 authored 시전 선택 거리 안의 아군 한 명을 정상 수동 입력으로 선택한다. 태아는 시전자 제외, 포큐버스는 시전자 포함 자격을 보존한다. 선택 대상은 해당 activation 동안 고정하며 자동 대체·중간 재지정·무제한 선택 사거리를 추가하지 않는다. 태아는 거리 밖에서 지원 membership을 제거하고 같은 activation/UnitInstanceId가 유효 거리 안으로 돌아오면 같은 owner에 재가입한다. 포큐버스는 시전 시 생성한 개인 방어막을 단순 범위 이탈 후에도 유지하고 재진입·시전자 이동으로 추가 부여하거나 충전하지 않는다. 대상 철수·전투불능·사망으로 identity가 무효해지면 해당 대상 연결/방어막만 종료하며 재배치된 새 identity에 이전하지 않는다. 원 source의 비용·channel 여부·만료를 바꾸지 않는다.

근거: PIS-CRSA-002/CH-IN01–08; DR-CRSA-08-002. 새 직접 합의인 포큐버스 이탈 유지와 기존 태아 channel membership을 분리한다. 선택 방식은 기존 위임된 정상 수동 선택을 유지한다.

- 정상 선택·무자격/거리 밖 거절, 태아 자기 제외/거리 이탈·복귀와 포큐버스 자기 허용/부여 뒤 이탈 유지·재진입 무충전을 실제 명령과 효과로 검증하며 자동 retarget이 없다.
- 대상 철수/사망/재배치 및 source 종료에서 각 효과의 연결/방어막 수명이 맞고 새 identity나 다른 아군으로 이전하지 않는다. 한 대상 무효화가 다른 포큐버스 대상의 유효 방어막을 종료하지 않는다.

## PE-CRSA-09 — 포큐버스 시전시 개인 방어막과 단일 강화

포큐버스는 시전 release 순간 authored 범위 안의 active player 직원(시전자 포함, Ground/Platform, Guest·소환물·오브젝트 제외)에게 각 한 개의 유한 개인 방어막을 원자 부여한다. 각 대상의 초기량·잔량·만료와 UnitInstanceId는 독립이며 제공자는 시전자다. 늦게 진입한 대상에게 부여하지 않고 단순 범위 이탈 뒤 유지하며 재진입·시전자 이동으로 재부여/복제/충전하지 않는다. 단일 recipe는 원 자격 안에서 선택한 한 명에게만 authored 강화 개인 방어막 하나를 생성하며 원 대상 수를 보호량에 곱하지 않는다. 일반/침식 강화량은 작성된 typed 값으로 검증하고 옛 공용2S/4S를 기본값으로 승계하지 않는다. 유물의 typed 초기량 연산 후 작성된 source 생성 modifier를 생성 시 한 번 적용하고 수혜자 장비로 다시 증폭하지 않는다. 피격 대상의 방어·감면 후 개인 보호 단계에서 흡수하고 초과분은 기존 후속 보호/HP 규칙으로 처리한다. 다른 개인 방어막과는 기존 priority 내림차순·동률 생성순, 개인→기존 공유 보호 순서를 유지하며 다른 제공자의 잔량은 합치지 않는다. HP 비용·희생은 흡수하지 않고 회복/Rest/시간은 충전하지 않는다. 각 방어막은 소진/작성된 만료/해당 대상 identity 종료 시 끝난다. 시전자 철수·전투불능·사망 또는 전투 종료는 그 제공자의 부여분을 종료한다. 같은 제공자 전투 identity와 같은 원 파편/장비 사본·효과의 재시전은 새 부여 집합과 함께 이전 부여분 전체(새 범위 밖 대상 포함)를 원자 교체하며 잔량을 합산하지 않는다. 다른 제공자·원 사본·효과의 독립 보호는 유지한다. 실패/취소는 이전 부여분·공명·RNG/ID를 보존한다. release 후 Hard CC/Silence만으로 이미 생성한 비집중 방어막을 취소하지 않고 일반 공격·저지는 병행한다. 공식 개인량식/계수·최종 범위·기간·일반/침식 밸런스·패널티는 이 기능 결정의 대표 authored 값으로 확정하지 않는다.

근거: PIS-CRSA-002/CH-IN01–08; DR-CRSA-09-002. 직접 합의인 독립 잔량/부여 시점/이탈 유지/2S·4S 보류를 반영한다. H04§5의 넓은 gameplay 위임으로 eligibility·provider/recipient 종료·원자 재시전·priority·modifier 합성 세부를 명시한다. 기존 finite owner/개인 보호 순서를 재사용하며 새 재산/상속·저장 전환을 만들지 않는다. 공식 사용자 보류 값은 건드리지 않는다.

- 정상 공개 시전에서 대상별1회 개인 부여·독립 흡수·늦은 진입 미부여·이탈 유지·재진입/시전자 이동 무충전과 선택1명 강화 결과를 독립 숫자로 검증한다. 대상 수나 수혜자 modifier가 개인량을 증식하지 않고 source modifier는 한 번이며 기존 R12 공유 보호 동작을 보존한다.
- HP 비용 우회·방어 후 흡수/초과분·기존 보호 priority·회복/Rest 무충전·개별 소진/만료/대상종료·source 종료·release 후 CC·같은 출처 재시전 전체교체/다른 출처 독립 및 늦은 실패 원자성을 검증한다. 시험용 보호량/기간/강화량을 공식 밸런스로 채택하지 않는다.

## PE-CRSA-10 — 태아 상태값과 집중·비용 보존

태아의 regular 기본 공격 피해 증가와 속도 가산은 기본20%/15, 일반 단일40%/20, 침식 단일80%/30이라는 각 상태의 최종값으로 교체한다. 이전 상태와 더하거나 모두 같은 배율을 곱하지 않는다. 범용 Attack stat·active 파편·장비 proc·공명·작업을 가속하지 않는다. 제공자는 시전자, 실제 공격/처치는 수혜자다. 원래 channel/lease·행동 제한·중단·Pause·자원/HP 비용 계약을 보존하고 이미 release된 공격을 소급 변경하지 않는다. 태아 전용 유물·미정 침식 패널티와 무기한 집중을 공식 채택하지 않는다.

근거: D03 태아3346–3488;D04 1074–1180;H09 C05;R11/R12. 판단: 세 상태 수치는 직접 합의이며 시험용 기간/비용/패널티를 공식 밸런스로 승격하지 않는다.

- 실제 regular basic의20/40/80과 기준속도100에서115/120/130, 비대상 active/proc 불변 및 중복 source 비중첩을 독립 oracle로 검증한다.
- 집중 종료·거리 이탈·Hard CC·직접 중지·Pause와 작성된 HP 비용에서 원 소유/원자성/부족 실패가 유지된다. 침식 수치 fixture의 시험용 penalty와 공식 미정 penalty를 구분한다.

## PE-CRSA-11 — 풍운승려 피해 footprint와1회 자기회복

풍운승려 구조 사례는 Physical 범위 bite 뒤 적에게 실제 HP 손실을 하나 이상 확정한 경우 시전자에게 한 번 자기 회복한다. 피해 footprint만 변형하고 치유 대상·횟수·기준량·carrier·원 출처는 보존한다. Staff를 Magic으로 바꾸지 않고 처치·피해량 비례 흡혈을 요구하지 않는다. 빈 범위·전부 무효/전부 보호막 흡수로 HP 손실0이면 회복하지 않는다. 회복은 살아 있고 회복 가능한 원 시전자에 기존 규칙대로 적용하며 부활/초과 shield를 추가하지 않는다.

근거: D03 풍운승려4066–4193;H09 C02;H04 A15-03;current FlowMetric/HpLost. 판단: 유효 적중의 미정 경계는 기존 settled HpLost>0로 채택한다. resolved 대상 수/단순 접촉/보호막만 적중을 채택하지 않은 위임 결정이며 사용자 직접 답변이 아니다.

- 피해 범위를 넓혀 적 여러 명을 맞혀도 회복은 시전자에게 한 번이고 다른 아군을 치유하거나 대상 수에 비례해 반복하지 않는다.
- 빈 범위·무적·완전 흡수·부분 흡수·처치/비처치에서 settled 실제 HP-loss 기준 분기가 맞고 Physical/source/최대HP clamp를 보존한다.

## PE-CRSA-12 — 생성·변형 출처와 권한

생성/변형 효과에는 기여한 유물 사본·효과와 원 실행 제공자·source/copy/form/action·catalog generation의 관계를 보존한다. 관측 출처를 추가했다고 새 action/resource/reinput/진행 권한을 부여하지 않는다. 실행 가능한 원 provider와 정규 등록을 검증한다. 에고 침식에 유물 증폭·추가 효과를 적용하거나 억제된 부가효과를 우회 재활성화하지 않는다.

근거: H04§5/X03;U08 source contract;relics 에고침식. 판단: 선행 권한 경계를 새 효과 identity가 우회하지 않도록 한다.

- 기본 효과와 생성 효과의 서로 다른 사본 기여를 실제 피해/보호/상태 결과에서 추적하고 위조·stale source/세대·권한은 거절한다.
- 유물 유무·새 구조 조합과 관계없이 에고 침식 및 기존 source/reinput/보스 진행 권한 회귀가 유지된다.

## PE-CRSA-13 — 공개 RON·조회·진단

새 ID와 새 조합의 정상 외부 RON을 loader→validation/compile→등록→정상 유물 획득·장착→실제 명령·결과→조회 경로로 연결한다. query/explain은 활성/휴면 이유, 적용 대상·전후 property·기여 source와 지원 제한을 설명하되 권위를 만들지 않는다. 세 대표 구조는 실제로 실행 가능해야 하며 schema만 추가하거나 같은 내부 객체 주입으로 대신하지 않는다. 공식 전체 조우/획득 빈도·UI 완성을 주장하지 않는다.

근거: H04 X01;U09 output/transition;R15§6,7;T13/T27. 판단: 기반 표현력을 실제 정상 경로로 닫고 미래 U10/U21/U26를 완료로 합치지 않는다.

- 독립 새 ID grant→다른 유물 강화와 세 대표 구조가 정상 공개 경로에서 실제 결과를 만들고 query와 일치한다.
- unknown property/잘못된 reference/type/충돌/미지원 reactive authoring은 정확한 source 위치·사유로 거절되고 부분 catalog 게시가 없다.

## PE-CRSA-14 — 사본·복원·세대와 수명

런 사본/슬롯/variant 저장은 U08의 소유를 유지하고 closure와 effective plan은 현재 검증된 콘텐츠·구성에서 재구성한다. current/checkpoint/entry/Continue/retreat의 비용·변형 동시 복원과 동일 런 generation pin을 보존한다. 아직 허용된 released owner는 끝날 때까지 유지하되 새 구성에 stale 효과를 재활성화하지 않는다. 생성효과/캐시/lease/pool의 생성·교체·취소·종료 참조를 회수한다.

근거: relics 상태보존;H04 X02/X03;U08/U07 contract. 판단: 저장된 유물 정체성을 보존하면서 plan과 battle-local 상태의 수명을 분리한다.

- 동일한 사본·variant를 fresh Core/Continue/checkpoint/retreat로 복원하면 같은 활성·휴면·실제 효과가 나오며 missing/changed content와 forged graph는 원본 보존 거절이다.
- 반복 구성 전환·전투·release 종료 뒤 과거 owner가 불필요하게 남지 않고 seed 제거·직원 이전/사망 회수·clone은 독립된 올바른 현재 구성을 갖는다.

## PE-CRSA-15 — 예산·원자성·결정론

closure 후보/plan·generated effect·payload·source/cache의 생성·보유·clone·rollback은 기존 resource admission과 retained 계량에 포함한다. 검증/게시·장착·변형·명령 중 한도·값·참조·늦은 실패가 생기면 기존 typed fault와 원자적 상태 보존을 따른다. 계산 종료를 위해 gameplay 총량/공통 proc cap을 신설하거나 기존한도·workload를 완화하지 않는다.

근거: H04§5,7/X04;U07M/U08 budgets;T23. 판단: 기존 안전 한도와 실제 비용을 보존하고 새로운 비용은 원시 전후로 설명한다.

- one-short/exact 경계와 누적 후보·plan 교체·late fault에서 실제 requested/accepted·retained가 정확하고 상태·RNG·ID·저장에 부분 commit이 없다.
- 같은 seed/구성·관측 off/on/saturated 및 다양한 장착 순서에서 권위 결과가 같고 source/lease/cache 해제 뒤 불필요한 retained 증가가 없다.

## PE-CRSA-16 — 선행 회귀와 성능 증거

U08의 전체32 수용문장과 이전191 조건, 별도 R06 31행을 직접 소비되는 계약과 실제 변경 영향으로 인수한다. exact/입증된 비영향 증거는 재사용하며 바뀐 제품·입력·성능 경계만 필요한 검증으로 갱신한다. PC36/R05 원 workload/cap/hard guard와 shared-host 측정 한계를 보존한다. R05 peak-only 기존 경고를 숨기거나 모든 성능 PASS로 부르지 않는다. 20분 stress는 제외한다.

근거: H04§7;current U08 whole audit013;U07M baseline. 판단: 기존 테스트 작성자를 재표기하거나 역할 전환만으로 전체 재실행하지 않는다.

- 선행223 조건과 별도31행의 affected/non-impact 근거가 현재 source 및 실제 nonzero 결과에 연결되고 영향받은 공개·fault·저장·수명 회귀를 검증한다.
- PC 비시간 지표와 R05 원시 전후 비용·결정론·retained/allocator 범위를 비교하며 실패 원본·peak-only 경고·환경 한계를 보존한다.

## PE-CRSA-17 — 전체 독립 완료와 후속 인수

정책/설계/제품/시험의 실제 전체 수용 조건과 원시 증거를 연결하고 evidence selection→predecessor impact→adaptation→coverage→자체검증→문서→exact target/격리 정역복원→비참여 전체 독립 감사를 순서대로 완료한다. 국소 컴파일·준비·자체시험 성공을 전체 완료로 취급하지 않는다. 독립 Complete/High/open0 및 출구 현재성 인수 후 U10 등 다음 승인 작업을 진행한다.

근거: current user request finalization/roles;GEX;U09 scope. 판단: 메인은 계획과 인수만 소유하고 제품/시험 작성이나 독립 완료 판정을 대행하지 않는다.

- 구현·테스트 역할이 분리되고 모든 수용문장/필수차원/통합 경계의 실제 증거가 고정된 현재 target에 추적되며 실패·미실행을 숨기지 않는다.
- 격리 정·역 복원과 비참여 Astra/High의 전체 Goal 감사가 Complete/High/open0이고 출구 currentness 및 후속 문서 delta를 인수한다.
