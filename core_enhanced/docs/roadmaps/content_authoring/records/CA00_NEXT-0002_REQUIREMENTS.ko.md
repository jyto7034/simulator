# CA00 — NEXT-0002 원문별 요구·정책 추적 기준선

작성일: 2026-09-09 · 작성자: Codex · 상태: **NEXT-0002 원문 요구·정책 기준선 조사 종료 / 구현·정식 검토 완료 아님**

## 요청과 경계

이번 사용자 요청은 “ROADMAP_STATUS.md를 읽고 다음 단계를 진행… 문서 배치와 P2G/GEX의 적용 버전도 확인… 작업 결과와 근거를 기록한 뒤 현황 문서 갱신”이다. 요청 시작 시 기록 0003의 NEXT-0002를 선택했다. 이전 [NEXT-0001 조사](CA00_NEXT-0001_INTAKE.ko.md)를 이어받는다.

허용 범위는 세 콘텐츠 원문 전체의 요구·정책 출처·상태·검증 계획 조사와 CA00 기록 및 현황 갱신이다. production 코드·RON·store·검증기·AGENTS·P2G/GEX 규칙은 수정하지 않는다. 정식 P2G 단계·Goal 작성·구현·독립 감사는 이번 작업이 아니다. 새로 작성할 다음 카드는 자동 실행하지 않는다.

worktree `/mnt/f/work/simulator`, branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`의 기존 dirty 변경을 보존한다. 첫 인수에서 확인한 원본을 초기화하지 않고 관련 해시와 새 변경을 확인한다.

## 1. 결과와 기록 배치

세 콘텐츠 원문 **9,168줄 전체**와 공통 정책 원문 **644줄 전체**를 읽었다. [요구 추적표](CA00_NEXT-0002_COVERAGE.json)에 **354개 COV**를 기록했다. 이 중 콘텐츠 행은 332개, 공통 정책 행은 22개다. 보스의 개별 능력·전환·실패·제작 포트는 **127개 BOSS-COV 하위 행**으로 연결했다. 하위 행은 원래 요구의 분해이며 독립 완료 건수에 더하지 않는다.

| 원본 | source key / 줄 수 | COV | 이번 관찰 |
| --- | --- | --- | --- |
| [유물](../../../skills/run_relic_content_design_notes.ko.md) | R / 1,402 | COV-R-001–062 | 기본·개화·전용/범용·단일 전환·15축·수치 제안 분리 |
| [파편·장비](../../../skills/abnormality_skill_fragments_design_notes.md) | F / 4,500 | COV-F-001–147 | 기본/개화·탱/딜·무기/방어구/액세서리·침식 분리 |
| [보스·도구](../../../skills/abnormality_design_notes.ko.md) | B / 3,266 | COV-B-001–123 | 보스/보상 분리, phase/행동/전장/작업/승패 및 도구 9종 기록 |
| [유물 공통 정책](../../../tmp/run_relic_system_design_notes_2026-08-21.ko.md) | P / 467 | COV-P-001–016 | 사용자 결정·획득 보류·과거 참고 구별 |
| [침식 공통 정책](../../../tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md) | E / 177 | COV-E-001–006 | 실제 시전 대체·철수/처형/대가·clock·기존안 폐기 구별 |

문서의 역할은 다음과 같다.

- 이 파일: 조사 결론, 정책 출처 수준, 현재/목표 차이, 다음 Topic 범위 제안과 미정의 owner.
- `CA00_NEXT-0002_COVERAGE.json`: 원문 경로·SHA-256·heading·줄, 합의/제안/미정, 효과·형태·단계/분기/병행, 해당 15축 하위 요소, 담당 CA/PG, 조합 후보, 긍정·부정 시험 recipe, 실제 receipt 부재, 남은 위험의 원본.
- [부속 증거](CA00_NEXT-0002_EVIDENCE.json): revision·dirty 기준, 인수한 입력의 변화, 읽기/문서 검사·보존 결과. 정식 P2G/GEX receipt가 아니다.
- [ROADMAP_STATUS](../ROADMAP_STATUS.md): 위 결과의 요약과 다음 작업 하나만 소유한다. 계획 원본 COVERAGE/CA00, 기존 정책·Goal·감사 원본은 변경하지 않았다.

`effect_members`는 연결된 효과 문장의 분해이며 새 opcode나 별도 승인 요소가 아니다. 원문 반복/위임 행은 삭제하지 않고 상세 owner와 연결했다. 9종 수치 유물은 각 등급이 독립 정의인 제안임을 `definition_variants`에 표시했다. `section_index`는 세 콘텐츠의 334개 heading 구간을 요구 또는 서사·빈 양식·구획·상태 설명으로 연결한다. heading 교차와 줄 범위 검사는 의미 완전성의 자동 증명이 아니다. 의미 분류는 원문 전체를 읽어 작성한 이번 조사 결과다.

## 2. 작업공간·기존 진행·적용 지침

요청 시작 관찰은 2026-09-09 04:34 UTC다. `git status --porcelain=v1 -uall -z`의 기존 항목은 16,485개(수정 167, 삭제 24, 미추적 16,294)였다. 비교용 텍스트 경로 1,665개의 SHA-256/부재를 수집했다. 이전 조사에서 selected input으로 연결한 38개 원본은 모두 같았다. 전체 자산·개인 저장·ignored 파일까지 보존을 검증한 것은 아니다. 종료 비교의 실제 수량과 변경 경로는 부속 증거를 따른다.

[NEXT-0001 §3–5](CA00_NEXT-0001_INTAKE.ko.md#3-기존-topicgoal실행감사-인수)의 CRAR Working/Review/Freeze/Plan/Goal Set/Handoff, P02 successor 감사, CWBS·Boss·지도 기록을 인수했다. 기존 `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-MASTER)`의 `Complete / High`와 감사 target 해시 일치 관찰은 유지하며 이번 재감사로 표시하지 않는다. 이전 1,159개 테스트 PASS를 재실행하지 않았다. CA01을 위한 정식 Topic·Phase Plan·Goal은 이번에도 생성하지 않았다.

| 지침/배치 | 적용 기준 |
| --- | --- |
| 운영 문서 | 이 폴더의 ROADMAP_STATUS v1.2 기록 0003 / OPERATING_RULES v1.1에서 NEXT-0002 선택. 폴더에는 실제 roadmap/·guides/·records/가 존재한다. 과거 배포 설명의 “생성하지 않았다”는 당시 기록이다. |
| P2G | [저장소 원문](../../../policy_to_goal_pipeline_revised_v2.md), **2026-09-09-r2**, SHA-256 `e439da72c0b2c792064f6f003940da0dda5fcd87f0b30c0e8ad835457244079b`. [배포 사본](../guides/policy_to_goal_pipeline_revised_v2.md)과 바이트 동일. Reader·§8.5/8.6/8.9에 따라 inspect와 조사 기록만 수행. |
| GEX 시작 기준 | 시작 당시 `docs/goal_execution_pipeline.md`, SHA-256 `5c19b62ae16943ef58ea2d02446f2fa188fa7bf8c18ad42d8ee79d0b18676c72`. 종료 검사에서는 [archive의 동일 본문](../../../archive/goal_execution_pipeline.md)으로 확인. 기존 Goal·감사는 이 과거 basis로 인수하며 새 revision 결과로 소급하지 않음. |
| GEX 종료 관찰 | [새 배치 문서](../../../goal_execution_pipeline%281%29.md), **2026-09-09-gex-r1**, SHA-256 `0ec53e9f68a82790c33a61c3b87d326d0af545c8fdbb47a6e97986c65b6a75c6`. 이번 작업 도중 외부 변경으로 나타났으며 Reader 1–165줄을 추가 확인. 본 작업은 계속 inspect; 새 GEX 실행/감사·정식 적용 검증 없음. |
| 옛 P2G 경로 | `docs/policy_to_goal_pipeline.md`는 부재. [archive](../../../archive/policy_to_goal_pipeline.md)는 과거 CRAR Plan의 bound 지침과 일치한다. 과거 artifact 해석용이며 r2 진행 방식과 섞지 않는다. |
| 남은 적용 한계 | 옛 P2G/GEX 표준 경로 링크와 실제 파일명이 다르다. 파일 이동·rename·runner routing 교정은 수행하지 않았다. 새 GEX도 accepted Handoff의 exact basis를 확인하도록 명시한다. 후속 정식 작업은 실제 경로/revision과 필수 runner를 대조하고 다른 버전 강제가 있으면 해당 owner로 반환한다. |

GEX 파일의 이동·추가는 Codex가 이번에 수행한 변경이 아니다. 기존 경로를 복원하거나 새 문서를 덮어쓰지 않고 해시와 실제 배치만 기록했다. 이 변경은 GEX 실행을 포함하지 않는 현재 조사 결론을 차단하지 않는다. 다만 시작 시점의 “지침 무변경”이나 모든 baseline 파일 동일성을 종료 결과로 주장하지 않으며, 보존 검사에서 외부 GEX 변경을 별도로 표시한다.

## 3. 정책 출처와 상태

출처 수준을 세 가지로 구별한다. **문서에 기록된 사용자 결정**은 원문을 읽은 근거다. **패키지가 이전 대화의 결정으로 인수한 항목**은 그 로컬 기록을 확보했지만 과거 대화 원문은 새로 확보하지 못한 근거다. **기획 합의·초안·미정**은 해당 콘텐츠의 문서 상태다. 어느 것도 이번 조사만으로 Final Freeze나 current gameplay canonical에 승격되지 않는다.

| 결정 | 확보한 원본/추적 | 현재 계약과의 관계·후속 owner |
| --- | --- | --- |
| U01 공통 조합·전용 실행 코드 금지 | [POLICY_GATES U01](../roadmap/POLICY_GATES.ko.md#확정된-사용자-결정), SOURCES S11의 이전 대화/패키지 기록 | 승인 방향을 유지. 현재 모든 콘텐츠의 data-only 지원을 주장하지 않음. CA01/21 |
| U02 조합기 제외·작성/검증/시험 기반 | 같은 표 U02 | UI 조합기 대신 Core 공개 계약 검토. CA01/03 |
| U03 대표 실제 이식·전체 표현력 | 같은 표 U03 및 [C01–C07](../roadmap/COVERAGE.ko.md#대표-공식-콘텐츠-실제-이식-목록) | 이번은 전체 요구 추적. 실제 RON 이식·실행 없음. CA20/21 |
| U04 최신 규격·no legacy/no auto migration·개인 파일 보존 | 같은 표 U04, SOURCES S09의 이전 대화 결정 | 패키지의 승인 기록 인수; 원 대화 미확보. current V4/V5 reader와 전환 목표를 구분. CA02/10/19 |
| U05 외부 다중 팩·새 Run 세대 pin·시험 초기화 | 같은 표 U05 | 원 대화 미확보. embedded publication은 존재하지만 외부 generation 구현 완료 아님. CA02/03/19 |
| U06 정의·등록/획득 경로 override 허용 | 같은 표 U06 | 원 대화 미확보. 충돌 해석·삭제 참조 검증은 PG07. CA02/17 |
| U07 외부 코드 실행 금지·권한 분리 | 같은 표 U07 | 원 대화 미확보. 공통 명령 데이터는 OS/파일/네트워크 권한이 아님. CA02/22 |
| U08 개별 파편 사본의 개화·형태·자원 | 같은 표 U08, F의 copy-local 자원 세부 | 원 대화 미확보. current 종류별 progress를 우선하지 않음. CA04/10/19 |
| U09 유물3슬롯·사본·휴면/생성효과·경로 | P147–181, P192–269; COV-P-005/007–012 | 공통 정책의 사용자 결정 기록 확보. 활성 closure/반복/비호환 알고리즘은 PG01/02. CA11–13 |
| U10 전투 초기화·재배치 분리·동종 가산 | P183–191, P272–289; COV-P-006/013 | 사본 Run 자원과 유물 임시 누적을 혼동하지 않음. CA04/05/11/19 |
| U11 무기 성장·파편 강화 제거·장비 자체 효과 | P49–104; COV-P-002/003 및 F 공통 header | 문서의 사용자 결정 인수; current 성장/보상/저장 동반 전환 필요. CA05/10 |
| U12 파편 침식 효과 제외·정상 진행·직원 대가 | P291–305, E26–137; COV-P-014/E001–005 | 침식 유물 variant와 다른 정책. 일반 damage/kill trigger 재사용으로 의미를 바꾸지 않음. CA06/18/19 |
| U13 Core/headless 범위, server/Unity 제외 | POLICY_GATES U13 | 보스/도구 원문의 UI·서버 문장은 소비 요구의 재료. 실제 클라이언트/네트워크 구현·성공은 이번 범위 밖. CA03/23 |

U04–U08을 미승인으로 되돌리거나 동일 결정을 다시 질문하지 않는다. 후속 P2G가 exact approval binding을 요구하면 현 로컬 결정 기록과 후속 사용자 요청의 승인 범위를 연결한다. 그것만으로 필수 provenance를 충족할 수 없다면 **그 요소의 부족한 근거만** 질문하며 추정 Ref를 만들지 않는다.

| 구분 | 유지한 판단 |
| --- | --- |
| 현재 코드 사실 | NEXT-0001 B01–B07의 실제 소스와 기존 audit input 동일성에 한정. 해당 효과의 실행 지원 분류는 아직 미판정. |
| P 목표 정책 | 위 U 결정과 원문에서 명시한 구조·숫자만 인수. 태아 +20/40/80%, 속도 +15/20/30은 교체 값; 포큐버스 일반 2S는 합의. |
| R 권장 구조 | typed AST/EffectSite/ActionSite, descriptor, result slot, branch ancestry, boss milestone 등은 로드맵의 제안. 실제 타입/API라고 표기하지 않음. |
| 미정·예시 | 쐐기 10%p, 9종 수치 유물, 포큐버스 침식4S, 태아 8초/HP6% 등은 공식 수치 승인 아님. 효과 의미가 미정이면 시험 숫자로 덮지 않음. |

## 4. 콘텐츠 추적의 주요 경계

| 경계 | 개별 COV 근거와 결론 |
| --- | --- |
| 효과 선택과 carrier | R016–028/061, F135–142: 피해 범위 변경과 자기 Heal·폭격 탐색·cadence·source를 분리. 같은 효과의 태그 AND와 property 지원은 별개. |
| 기본/개화·탱/딜 | F005–010, F015–018, F027–042, F045–060, F070–071, F082–093, F100–101: 상태·비용·수명·병행 차이를 유지. 녹사랑 탱 개화 없음과 아무 것도 없는 개화 보류는 빈 효과를 창작하지 않음. |
| 사본/직원/Run 상태 | F034/042/074–075/079/083/088/119, P013: 욕망·양의 탈·시체 질량과 직원 부활 횟수·울대 수신기 대가는 서로 다른 owner. 재배치/이전/Retreat/commit은 개별 수명대로 검토. |
| 피해·치유·방어 결과 | F012–014/020–025/053–063/072–081/120–131: 피해 감면→흡수→HP손실, 실제 Heal, 이전 source, HP비용, 처형 segment의 인정 단위를 별도 추적. |
| 연쇄·공유·전파 | R034–046, F046/050/057–060/092/100–101: strike와 segment, 공유 총량과 수혜자 수, tile 집계와 resource 소비, 원천별 감염망, 장비 허용 재귀와 유물 경로 재방문 차단을 분리. |
| 조건·시간·취소 | F070–071/114–115/134/139/143–147, B080/088/109: timeout/reinput, channel/basic 병행, finite opportunity, 공격 release 시점, 취소 뒤 effect 부재와 같은 timestamp 우선순위. |
| 보스와 보상 | B006–012/020/112–120은 플레이어 보상이며 boss 구현으로 세지 않음. 파편 파일의 최종보스/백야 절은 비어 있어 B 원문의 미래 보상으로 연결. 원본 장례 무보상과 아종 고정세트 보상 구분. |
| 보스 제작 전체 | `boss_authoring_coverage`에서 phase/selector/task/parameter/query/movement/skill/telegraph/state effect/blackboard/signal/summon/wave/multi-actor 요구를 연결. 현재 Boss task dispatcher와 새 skill invocation·milestone 공백을 분리. |
| 미설계 보스 | B021 흉탄은 보스 능력·전장 미정, B039 천국은 exact 임무·승패 미정. 플레이어 파편으로 보스 능력을 채우지 않음. 백야의 정책 재승인 표시 유지. |
| 도구형 9종 | B090–111: 수액 ledger, 초상 전이, 팔찌 결과 HP계약, 우상 ritual, 조립 갑옷, 피의 욕조 원자희생, 무기1instance1시도, 열차 작업경합, 웰치어스5Floor귀환을 기록. 앞쪽 도구 요약의 폭발/반납사망/랜덤즉사보다 최신 상세 절을 우선. |

도구형의 공통 state/transaction/대상/clock/작업/피해 기능은 표현력 요구로 남긴다. 다만 **새 active-battle 도구 command, 특수 장착 슬롯, Tool node 공급·5Floor 보상 이식**을 네 제작 도메인의 현재 승인 범위에 자동 추가하지 않는다. 범위 후보는 “도구·노드 효과의 공통 작성 소비자”이며 CA17/19와 해당 primitive owner가 기존 item entry/검증/상태 투영·result를 검토한 뒤 조정한다. 완료 기준 후보는 동일 parser/검증/원자적 적용·저장/복원·거부 무변경 시험이다. 기존 CA 번호 밖의 실제 작업이 필요하면 후속 Topic의 명시적 scope 선택으로 기록한다. 이번에는 그 선택을 수행하지 않았다.

## 5. 시험 계획과 현재 지원 근거

모든 COV에 `actual_validation_receipt: null`을 기록했다. `test_recipe_catalog`의 19개 긍정·부정 recipe를 각 행의 대상·mechanics·미정과 함께 사용한다. 이는 실행 파일이나 통과 결과가 아니다. 다음은 핵심 기대를 고정할 최소 fixture 후보이며 나머지 행도 자기 recipe/PG를 가진다.

| 대상 | 긍정 시험 후보 | 부정/경계 시험 후보 |
| --- | --- | --- |
| CA01 / R016·025, F135 | 서로 다른 effect의 피해와 자기 Heal, 각자 type/property/source | 피해 radius 변경이 Heal에 전파; 서로 다른 효과의 태그를 AND로 오인; 미지원 opcode 성공 |
| 깊은 쐐기 / R034–037 | 같은 cast·target의 첫/둘째/셋째 strike에서 n=1/2/3 | miss·segment·다른 target·새 cast로 횟수 오염; 0피해 인정은 PG03 전 미확정 |
| 포큐버스 / F126–131 | 공유600에서 A최종피해400→잔량200, B350→HP150 손실 | 각 대상600 복제, 방어 이중적용, reentry충전, 생성 증폭 수혜자별 재적용 |
| 단일 전환 / R044–051 | 태아 상태별 피해20/40/80·속도15/20/30 교체; 포큐버스1명2S | 세 상태 가산, 포큐버스 channel 강제, 제거된 원형 radius 활성,4S 공식 승인 |
| 고기벽 / F053–063 | 손상된 맨위 구간만 벽으로 분리; 남은 pool·저지 갱신 | 부족 HP로 치명 비용·부분소비, 벽흡수로 ownerHP피격 trigger, 벽에 Heal/회수 반환 |
| 포자 / F057–060 | 원천별 감염·전파·살아있는 수/시간 성장·일제소비 | lethal hit/death 두 번 전파, 새 세대 즉시 재추첨, 빈기폭이 자원/revision 변경 |
| Hello / F070–071 | 기본공격 교체 중 기본 source 유지; 재입력 Goodbye 뒤 복귀 | 추가 공명, 시전즉시 기본공격, timeout에도 Goodbye, 초기화로 공격 timer 이득 |
| 흑조 / F120–122 | 자신의 파편 감쇄 기여만 축적→성공 방출 | parry/무적/다른 방어를 기여에 합산, 실패시 축적 삭제, 반사 재귀 |
| 사본과 직원 / F074–075·083 | 별도 사본 욕망 독립; 직원 부활 Run횟수 이전해도 직원에 남음 | 장비 이전으로 부활3번째 회피, 같은종 사본 전체개화, Retreat 이중 commit |
| 나비·루돌타 / F046·139 | tile별 집계; 유한 투하 기회·개별 impact snapshot | 비어 있던 투하 적립, 공속이 폭격 cadence 변경, 큰 몸체 중복 적중 |
| 눈여왕 / R043, F109–110 | 자신의 유효 냉기 중 다른 직원이 kill→잔영1 | 만료/다른 원천 냉기·면역 적 생성, 원파편 직접kill과 잔영2회 |
| 보스 C03 / B047–048 | 변신 중 피해·현재HP phase skip·HP0 즉시clear | 강제 무적/HPgate/초과피해 삭제/필수 모든 형태 경유 |
| 보스 C07 / B070–074 | 잠복→거리 조건 노출→작업 억제→노출 피해 | 작업미완료를 절대 피해불가로 변경, 원거리 조건을 플레이어 shield에 전이 |
| 도구 / B092·104·109 | commit된 수액위험·원자희생·동시간 열차 우선 | invalid RNG소모, 부분희생/복구, 재접속 reroll, 열차/작업 순서 역전 |

현재 지원의 직접 근거는 [NEXT-0001 B01–B07](CA00_NEXT-0001_INTAKE.ko.md#4-주요-의존-경계-1차-확인)이다. B01 opaque validated publication, B02 step 기반 SkillDef, B03 기존 Damage/Heal 실행·시험 소스, B04 종류별 파편 progress, B05 전투 item/proc identity, B06 snapshot/V5 writer, B07 boss task dispatcher를 해당 COV에 연결했다. 기존 코드가 일부 기능을 이미 지원하므로 “모두 누락”으로 분류하지 않았다. effect descriptor·사본 개화·generation pin·명시 boss skill invocation은 확인된 목표 delta이며, 그 밖의 정확한 구현 지원 분류는 해당 CA 코드 검토에서 닫는다.

## 6. 미정의 영향과 후속 owner

| 항목 | 영향과 질문/확인 대상 | owner와 시점 |
| --- | --- | --- |
| PG00 | 원문 구조와 prototype 숫자 구분. 미정 의미를 숫자로 채우지 않음 | CA00 기준선에 분류; CA20 fixture 선택 전 |
| PG01 | property 합성/Replace 충돌·해제·carrier 공유 범위. 동종 피해증폭 가산은 재질문 없음 | CA01 interface → CA05/08/12 동결 전 |
| PG02 | 생성 효과 활성의 순환/NOT·branch-local 방문·join/타이머 root 의미 | CA12/13/15. CA01은 필요한 주소/descriptor 제약만 검토 |
| PG03 | accepted hit/0피해/완전흡수, release/segment, timeout/reinput·lethal/death 우선순위 | CA05–09/13/16. 기존 의미를 보존하는 최소 CA01 slice에 필요한 부분만 먼저 확인 |
| PG04 | 개인/공유 보호 순서, HP비용·parry/부활/처형 중재 | CA06/09/18. 도구 예외 영구사망은 별도 scope·정책 확인 |
| PG05 | copy/employee 상태, 재시전·이전·재배치·결과/Retreat의 실제 보존 | 상태 도입 owner가 저장/복원을 동반. CA19 전체까지 지연 금지 |
| PG06 | 포큐버스 식/범위, 벌집 충전, 원천 시체 경쟁, 신규 boss 능력·Tool 공급 등 | 해당 콘텐츠 owner. 독립 primitive 포트를 막는 전역 선행으로 만들지 않음 |
| PG07 | override 충돌·삭제 참조·effective fingerprint·save mismatch | CA02/19; 형식은 기술 설계, 저장 손실 의미는 정책 |
| PG08 | 무한 gameplay 누적과 제한된 실행예산 구분, Certified 가중·Endless model | CA17/22. 계측·안전예산 없이 성능 완료 주장 금지 |
| PG09 | 침식 유물 해제/재적용/패널티, 공통 trauma임계/연출기간,4S비교안 | 해당 variant와 CA18/20. 예시를 현재 official profile로 채택하지 않음 |

이번 조사 종료에 필수인 원문은 확보했다. 위 항목은 기록해야 할 미정이지 지금 사용자가 선택해야 조사를 마칠 수 있는 질문이 아니다. 특히 내부 container, SourceMap, type 이름, loader 호출관계는 후속 검토자가 조사·결정할 기술 사항이다. 사용자에게 넘길 실제 선택은 그 범위가 해당 의미를 소비할 때 구체적인 대안·영향과 함께 제시한다.

## 7. 다음 Topic 범위 제안

다음 한 단위는 **NEXT-0003 — CA01 최소 작성·실행 지원 계약의 정책 준비와 P2G-20 코드 검토까지**를 제안한다. 정식 Topic/Working Revision Ref는 아직 없다. 다음 사용자가 이 카드를 선택할 때 r2 `bounded_pipeline`으로 필요한 P2G-00/10/20을 수행하고 **P2G-20 exit 또는 필수 정책/근거 blocker에서 정지**하도록 한다. Freeze·Phase·Goal·Handoff와 GEX는 별도 다음 작업이다.

제안 범위는 효과별 identity·태그와 실제 property descriptor, effect/action carrier 주소의 구별, source 위치와 미지원 진단, 기존 Damage/Heal/Instant로 내려가는 최소 typed 작성·검증·실행 지원 계약이다. 새 기능을 빈 executor나 항상 성공하는 enum으로 등록하지 않는다. 필요하면 공유 계약 K01/K02/K04–K10의 최소 port sketch를 작성하되 모든 후반 kernel을 미리 구현하지 않는다.

직접 요구 입력은 R016–028/061, F135–138과 CA01/T01이다. 후행 소비는 CA02 최신 schema, CA03 작성/검증 포트, CA12 effect selector/patch, CA16 boss action 연결이다. 후행 요구가 필요한 주소·결과·취소 계약을 바꾸는지 검토하되 전체 보스·상태·팩 기능을 CA01의 선행으로 만들지 않는다.

| 코드 검토 질문 | 현재 근거 | 결정 책임 |
| --- | --- | --- |
| RawSkillDatabase→SkillDef→명령 생성→Damage/Heal의 effect identity와 unit/type를 어디에 유지할까? | NEXT-0001 B01–B03 | P2G-20: 실제 parser/validator/executor/오류 소비자를 연결. L 설계 |
| target/delivery 공유 변경이 자기 Heal까지 번질 때 거부/분리 경계는? | R016/025, F135, 현재 step-level target | P2G-20: effect site와 action site 최소계약. 새 gameplay 의미면 P2G-10 |
| descriptor와 executor의 지원 목록을 어떻게 일치시킬까? | 현재 enum만으로 실행 지원을 증명 못함 | P2G-20: unsupported 조합·type/참조 부정 fixture 계획, 실제 지원만 노출 |
| 최소 schema 전환에 동반해야 할 current RON·tests·canonical·저장 소비자는? | publication 및 기존 test entry는 존재; 미래 외부팩 아님 | P2G-20에서 범위 조사, P2G-40에서 최소 동시 전환/중간 상태 확정. 이번 Plan 미작성 |
| 새 runtime 상태/ID를 실제로 도입하는가? | B04–B06, 저장 writer V5·reader V4/V5 | 도입한다면 snapshot/rollback/save owner 포함; 없다면 근거 있는 N/A. 전체 사본화 자동 포함 금지 |
| 검증 관찰은 public Core 포트인가 test-only builder인가? | B03와 test fixture 가시성 차이 | CA01 시험과 CA03 공개 포트의 정확한 소비 관계를 기술 조사 |

초기 scope는 위 계약 검토로 한정한다. 새로운 semantic operator 전체, 외부 pack 구현, 사본 inventory 전환, 유물/보스/도구 실데이터, server/Unity, 저장 파일 정리, AGENTS/P2G/GEX 규칙 수정은 포함하지 않는다. 다만 최소 계약 성립에 실제 필수인 소비자를 검토 밖으로 밀어내지는 않는다. 코드로 새 필수 범위가 드러나면 그 결과·선택을 owner에 반환한다.

## 8. 검증과 종료 판단

이번 실제 활동은 원문 전체 읽기, 354개 요구·127개 보스 하위 행 작성, 기존 근거 인수와 문서 정합성/보존 진단이다. 검사 명령·수량·실제 결과는 부속 증거의 `final_checks`에 둔다. JSON/ID/참조/줄 범위·읽기 구간·소스 해시·문서 링크를 검사하는 로컬 일회성 진단만 사용했으며 production 검증기 소스는 변경하지 않았다.

**게임 구현·RON 이식·Rust 빌드/테스트·probe·benchmark·정식 P2G validator·D2/DG·GEX 감사는 모두 미수행**이다. COV에는 모든 행의 최종 구현 분류를 미판정으로 남겼다. 자료 미확인을 `공통 기능 누락`, `명시적 비범위`, `해당 없음`이나 `지원·실행 확인`으로 임의 치환하지 않았다. T00의 원문 추적 입력을 작성했지만 정식 실행 검증·감사 완료로 주장하지 않는다.

NEXT-0002의 종료점인 전체 원문 요구·미정/비범위 이유·정책 출처 수준·CA01 범위 제안/미해결 질문 기록에 도달했다. 현재 요청에서 정책 선택을 대신해야 하거나 필수 자료가 없어 중단된 요소는 없다. NEXT-0003을 현황에 기록한 뒤 그 새 작업은 실행하지 않는다.
