# U03 단일 완료 단계 설계

Bound Freeze: `FPF-CRFI-003`. Exact basis와 selector는 [Phase Plan](P2G_40_PHASE_PLAN_002.json)이 소유한다.
Repair Basis: 형식 교정 O10/O20/O30 모두 해소; Change002 O40가 현재 수리 대상이며 produced resolution은 완전한 stage 결과에 결속한다. Supersession은 explicit empty.
Parallelization: none approved. 구현 에이전트를 추가하지 않고 주 작업자가 순차 구현한다.

| Work Package | 책임 | 소비/완료 경계 |
| --- | --- | --- |
| CORE | 최초 typed fault·root 외부 수명·공개 gate/조회 | HOST와 OBSERVE가 실제 API를 소비 |
| HOST | tick/명령/재접속·실제 writer·명시적 기존 복구 | 다른 player 진행과 원본 저장을 함께 검증 |
| OBSERVE | 오류 종료의 current/selected 정리·실패 통계 보존 | fake commit 없이 기존 collector로 관측 |
| VERIFY | V01–12·100 선행 조건·실제 release host·감사 target | final source의 수락 조건별 증거와 독립 감사 |

CORE/HOST/OBSERVE는 `AG-CRFI-RUNTIME` 한 최소 전환 단위다. Core만 먼저 완료라 하면 host timer/저장 우회가,
host만 막으면 공개 embedding 우회가, 관측만 남기면 종료 집계 누수가 생긴다. 내부 순서는 CORE→HOST 및
CORE→OBSERVE이고 verification은 세 구현을 함께 소비한다. 같은 Phase의 부분 Work Package 완료는
외부에 독립 release로 게시하지 않는다. 이 결합은 동일 transaction을 서버 전체에 확장한다는 뜻이 아니다.

`PHASE-CRFI-01`은12요소의 유일 primary owner며 AG와 VERIFY 두 final node를 포함한다.
단계 직후 단일 Core authority·정상 거절/조회/commit 경계·live transport·save bytes·기존 Quit/Continue·
세션 격리·bounded 관측이 모두 유효해야 한다. 새 저장 schema/migration/fallback은 없고 독립 감사 전
U04 종속 구현을 시작하지 않는다. P2G-50에 전달하는 구조 사실은 `single_phase`다.

12개 validation seam은 모두 VERIFY에 명시적으로 배정했다. 독립 감사자에게 아직 없는 fixture 구현을
넘기지 않는다. 실제 failure 입력/실행 측정은 GEX 내부 owner가 만들고 nonzero 결과와 한계를 남긴다.
표준 receipt는 selector/graph/cardinality만 검증하며 이 경계의 의미적 판단을 대체하지 않는다.
