# U17 실행 계획

현재: GEX-10 구현·자체검증 종료 준비, 최종 target 고정/정역복원 후 감사 요청 대기. GS-CMER-001 / GOAL-CMER-01 / PHASE-CMER-01. 사용자 요청: `CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md 문서를 지침 삼아 u17 작업에 착수해줄래?` (2026-10-15). 추가 요청으로 갱신 지시서를 다시 읽었으며 §1 관련 기반 리팩토링을 적용한다. [고정한 지시서](EXECUTION_INSTRUCTIONS_20261015.ko.md), [변경 기록](EXECUTION_GUIDANCE_UPDATE_001.json).

P2G-60 exact basis는 [인계](../../P2G_60_HANDOFF_001.json), 수용 조건22개와 차원10개는 [Goal](../../P2G_50_GOAL_SET.ko.md), 구조와 실제 호출은 [코드 검토](../../CODE_REVIEW.ko.md)를 따른다. 단독 수행하며 서브에이전트·자동 감사를 하지 않는다. 구현/자체검증/증거 결속/최종 target·정역복원 뒤 감사요청 인계가 종료 경계다. U18 이후는 승인 범위 밖이다.

U16 현재 Complete/High와 XP 비차단 기록을 인수한다. 과거 NotComplete 원본을 수정하거나 XP 문제를 U17 필수 교정으로 되살리지 않는다. 초기 근거는 Core2228통과/6ignore, probes11통과, 교정 결과를 합친 server 고유96통과/22ignore다. 새로운 단일 workspace 전체 통과라고 하지 않는다. [BASELINE_SOURCE_001.json](BASELINE_SOURCE_001.json)의515개 파일은 U17 제품 변경 전 실제 dirty 작업 트리이며 archive member 복원 해시 확인을 마쳤다. 무관 dirty·동결 원본은 보존한다.

## 구현 순서

1. 원정별 최초/반복 plan과 알·단계·부화노드 owner, source/recipe/stage 작성·검증을 마련한다. 구 모노리스 구매/숙달/교환과 별개 책임으로 분리한다.
2. 명시 촉매 지불/독립 hatch commit, 자연 반복, 단계 결과의 fixed/choice 보상을 기존 U15/U04 owner에 연결한다. 예약/봉인/전조와 전투·노드 비용 수명을 유지한다.
3. 공개 command/capability/query, 새 저장/clone/계량, Continue/SavePoint/후퇴를 함께 연결하고 구 활성 소비·제안 경로를 제거한다. 구명령/구저장은 typed reject로 원본 보존한다.
4. 대표 정상 작성 콘텐츠와 집중 검사, 공개 host/새 프로세스/IO 실패·재시도, 비용 측정 및 영향 통합을 수행한다. 자체 코드 검토 후 coverage·target·정역복원과 감사 인계를 정리한다.

## 검증 준비와 독립 기대값

규칙은 최초1회/분쇄후재지급0, 일반보스 알 수열1→부화0→2승1→부화0→3승0이다. 엘리트/최종보스/패배는 알0이다. 후퇴는 비용 불변/후퇴+1/노드 미해결, 최대2회 뒤 마지막 도전이 남는다. 패배는 성공 보상/격파0·노드소비1·부화비 환급0이다. 부화 자체는 전투/후퇴/입장비를 새로 실행하지 않는다.

정상 전투 시험은 지원 RON의 실제 starter·정상 Event/상점·노드 연결로 자원을 준비하고 배치/피해/종료 규칙을 확인한다. 최초 승리 후 다음 단계의 진입 자격과 알·촉매·선택 권리·노드 상태를 매번 확인한다. 짧은 사전 관측으로 실제 목표 전이 도달을 확인한 뒤 장시간 host 경로를 실행한다. 각 branch는 저장을 외부 편집하지 않고 정상 분기로 만든다.

복원 관측은 (a)입장 전 hatch 확정 후 종료, (b)노드 입장 뒤 hatch 후 미완료 종료, (c)hatch 후 실제 battle 후퇴, (d)승패 직후 미확인 결과, (e)장비 선택 대기, (f)부화 전/후 SavePoint다. 같은 프로세스 재접속과 fresh process를 별도 검사한다. 출처 지불은 소지/창고/혼합 및 부족을 대조하고 원정과 무관한 창고 잔액을 보존한다. 실제 IO 실패는 candidate 전체 거절 뒤 같은 명령 재시도로 검증한다.

기존 숙달/모노리스 시험 중 현재 규칙을 대체한 부분은 정상 최초/수동/알 경로로 목적을 교정한다. 과거 artifact 안의 시험은 보존한다. 지원 작성 오류와 정상 명령 오용은 검증하며 외부 임의 변조는 제외한다.

기본 실행 환경은 PATH=/home/blast/.cargo/bin:$PATH, CARGO_HOME=/tmp/wp02-cargo-home, CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, offline/locked다. 공유 target의 동시 Cargo 빌드는 하지 않는다. 최초 집중 빌드는 과거 근거상2–8분 예상,60초에 로그/진행을 확인하고10분 초과 시 준비/컴파일/실행 원인을 구분한다. 출력 부재만으로 중단하지 않는다. 긴 host/비용 trace는 /tmp에 수집하고 원문을 보존해 Windows mount IO가 제품 시간에 섞이지 않게 한다.

정상/확장 비용은 실제 시간·heap peak/종료후회수·RSS와 내부 work를 구별한다. 규모는 실제 대표 경로 pilot 후 고정하고 원시 실패/중단을 성공 측정에 섞지 않는다. 전체 통합은 최종 제품/시험 입력에서 한 번 수행하고 이후 새 변경/실패 영향만 재실행한다.
